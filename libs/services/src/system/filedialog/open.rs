//! `xdg-open` compatible launcher for files, folders and URIs.
//!
//! GIO handles most URIs; BabyDra applications and desktop entries provide the
//! fallback chain so the compositor can own the default-open experience.

use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::Command;

use gio::AppInfo;
use url::Url;

const TEXT_EXTENSIONS: &[&str] = &[
    "txt", "md", "markdown", "rs", "c", "h", "cpp", "hpp", "cxx", "py", "js", "ts", "jsx", "tsx",
    "json", "toml", "yaml", "yml", "sh", "bash", "zsh", "css", "scss", "html", "xml", "csv", "ini",
    "conf", "log", "diff", "patch", "desktop", "service", "sql", "lua", "go", "java",
];
const MEDIA_EXTENSIONS: &[&str] = &[
    "png", "jpg", "jpeg", "webp", "gif", "bmp", "svg", "ico", "mp4", "mkv", "webm", "avi", "mov",
    "flv",
];

/// Opens a path or URI with the desktop's default application; returns `true` on success.
pub fn open_target(target: &str) -> bool {
    if ["http://", "https://", "mailto:", "ftp://"]
        .into_iter()
        .any(|s| target.starts_with(s))
    {
        if launch_uri(target) {
            return true;
        }
        for browser in [
            "opera",
            "google-chrome-stable",
            "google-chrome",
            "firefox",
            "chromium",
        ] {
            if launch(browser, &[OsStr::new(target)]) {
                return true;
            }
        }
        tracing::warn!("no handler for URI {target}");
        return false;
    }
    let path = resolve_path(target);
    if path.is_dir() {
        return show_in_file_manager(&path);
    }
    let uri = file_uri(&path);
    launch_uri(&uri) || fallback_open_file(&path, &uri)
}

/// Opens a directory in BabyDra Explore, falling back to GIO.
fn show_in_file_manager(path: &Path) -> bool {
    launch("babydra-explore", &[path.as_os_str()]) || launch_uri(&file_uri(path))
}

/// Spawns a helper binary with arguments.
fn launch(binary: &str, args: &[&OsStr]) -> bool {
    Command::new(binary).args(args).spawn().is_ok()
}

/// Launches a URI through GIO's registered default application.
fn launch_uri(uri: &str) -> bool {
    let ok = AppInfo::launch_default_for_uri(uri, None::<&gio::AppLaunchContext>).is_ok();
    if ok {
        let ctx = glib::MainContext::default();
        for _ in 0..15 {
            if !ctx.iteration(false) {
                break;
            }
        }
    }
    ok
}

/// Resolves a possibly relative path or `file://` URI into an absolute path.
fn resolve_path(target: &str) -> PathBuf {
    let raw = Url::parse(target)
        .ok()
        .filter(|u| u.scheme() == "file")
        .and_then(|u| u.to_file_path().ok())
        .unwrap_or_else(|| PathBuf::from(target));
    let absolute = if raw.is_absolute() {
        raw
    } else {
        std::env::current_dir()
            .map(|cwd| cwd.join(&raw))
            .unwrap_or(raw)
    };
    std::fs::canonicalize(&absolute).unwrap_or(absolute)
}

/// Builds a `file://` URI for a local path.
fn file_uri(path: &Path) -> String {
    Url::from_file_path(path)
        .map(|u| u.to_string())
        .unwrap_or_else(|_| format!("file://{}", path.to_string_lossy()))
}

/// Falls back to BabyDra apps, MIME defaults and desktop entries.
fn fallback_open_file(path: &Path, uri: &str) -> bool {
    let ext = path
        .extension()
        .and_then(OsStr::to_str)
        .unwrap_or_default()
        .to_ascii_lowercase();
    let app = if TEXT_EXTENSIONS.contains(&ext.as_str()) {
        Some("babydra-notepad")
    } else if MEDIA_EXTENSIONS.contains(&ext.as_str()) {
        Some("babydra-preview")
    } else {
        None
    };
    if app.is_some_and(|app| launch(app, &[path.as_os_str()])) {
        return true;
    }
    if open_with_mime_default(path, uri) {
        return true;
    }
    // Last resort: small files are almost certainly text documents.
    let small = std::fs::metadata(path)
        .map(|m| m.len() < 10 * 1024 * 1024)
        .unwrap_or(false);
    small && launch("babydra-notepad", &[path.as_os_str()])
}

/// Queries `xdg-mime` for the default desktop entry and launches it.
fn open_with_mime_default(path: &Path, uri: &str) -> bool {
    let query = |args: &[&str]| {
        let out = Command::new("xdg-mime").args(args).output().ok()?;
        let value = String::from_utf8_lossy(&out.stdout).trim().to_string();
        (!value.is_empty()).then_some(value)
    };
    let path_str = path.to_string_lossy();
    let Some(mime) = query(&["query", "filetype", path_str.as_ref()]) else {
        return false;
    };
    let Some(id) = query(&["query", "default", mime.as_str()]) else {
        return false;
    };
    let app = if id.contains("notepad") {
        "babydra-notepad"
    } else if id.contains("preview") {
        "babydra-preview"
    } else if id.contains("explore") {
        "babydra-explore"
    } else {
        return launch_desktop_entry(&id, path, uri);
    };
    launch(app, &[path.as_os_str()])
}

/// Launches the `Exec=` line of an installed desktop entry.
fn launch_desktop_entry(id: &str, path: &Path, uri: &str) -> bool {
    let dirs = [
        dirs::data_dir().map(|d| d.join("applications")),
        Some(PathBuf::from("/usr/local/share/applications")),
        Some(PathBuf::from("/usr/share/applications")),
    ];
    for dir in dirs.into_iter().flatten() {
        let Ok(content) = std::fs::read_to_string(dir.join(id)) else {
            continue;
        };
        let Some((binary, args, is_terminal)) = parse_desktop_entry(&content, path, uri) else {
            continue;
        };
        let bin_name = Path::new(&binary)
            .file_name()
            .and_then(OsStr::to_str)
            .unwrap_or(&binary);
        if matches!(
            bin_name,
            "xdg-open" | "BabyDra" | "babydra-filedialog"
        ) {
            continue;
        }
        if is_terminal {
            for term in ["kitty", "alacritty", "foot", "xterm"] {
                if Command::new(term)
                    .arg("-e")
                    .arg(&binary)
                    .args(&args)
                    .spawn()
                    .is_ok()
                {
                    return true;
                }
            }
        } else if Command::new(&binary).args(&args).spawn().is_ok() {
            return true;
        }
    }
    false
}

/// Parses `Exec=` and `Terminal=`, substituting `%f`/`%u` fields with `path`/`uri`.
fn parse_desktop_entry(
    content: &str,
    path: &Path,
    uri: &str,
) -> Option<(String, Vec<String>, bool)> {
    let exec = content
        .lines()
        .filter_map(|l| l.trim().strip_prefix("Exec="))
        .next_back()?
        .trim();
    let is_terminal = content
        .lines()
        .any(|l| l.trim().eq_ignore_ascii_case("Terminal=true"));
    let mut parts = exec.split_whitespace();
    let binary = parts.next()?.trim_matches('"').to_string();
    let path_str = path.to_string_lossy();
    let mut args = Vec::new();
    let mut substituted = false;
    for part in parts {
        let clean = part.trim_matches('"');
        let value = match clean {
            "%f" | "%F" => {
                substituted = true;
                path_str.as_ref()
            }
            "%u" | "%U" => {
                substituted = true;
                uri
            }
            other if !other.starts_with('%') => other,
            _ => continue,
        };
        args.push(value.to_string());
    }
    if !substituted {
        args.push(path_str.into_owned());
    }
    Some((binary, args, is_terminal))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn substitutes_desktop_entry_fields() {
        assert_eq!(
            parse_desktop_entry(
                "Exec=note --wait %f",
                Path::new("/tmp/note.txt"),
                "file:///tmp/note.txt"
            ),
            Some((
                "note".to_string(),
                vec!["--wait".into(), "/tmp/note.txt".into()],
                false,
            ))
        );
    }

    #[test]
    fn appends_path_when_no_field_present() {
        assert_eq!(
            parse_desktop_entry(
                "Exec=babydra-preview",
                Path::new("/tmp/a.png"),
                "file:///tmp/a.png"
            ),
            Some((
                "babydra-preview".to_string(),
                vec!["/tmp/a.png".into()],
                false,
            ))
        );
    }
}
