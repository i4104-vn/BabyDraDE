use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::Command;

use gio::AppInfo;

use super::desktop_entry::parse_desktop_entry;
use super::target::{MEDIA_EXTENSIONS, TEXT_EXTENSIONS};

/// How many GLib main-context iterations drain launch side effects; a higher
/// value only delays the launcher, it does not change which app is used.
const MAIN_CONTEXT_ITERATIONS: usize = 15;
/// Files at most this small are treated as text when no handler is configured;
/// raising it risks opening binary files in the notepad.
const SMALL_TEXT_FILE_LIMIT: u64 = 10 * 1024 * 1024;

/// Spawns a helper binary with arguments.
pub(super) fn launch(binary: &str, args: &[&OsStr]) -> bool {
    Command::new(binary).args(args).spawn().is_ok()
}

/// Launches a URI through GIO's registered default application.
pub(super) fn launch_uri(uri: &str) -> bool {
    let ok = AppInfo::launch_default_for_uri(uri, None::<&gio::AppLaunchContext>).is_ok();
    if ok {
        let ctx = glib::MainContext::default();
        for _ in 0..MAIN_CONTEXT_ITERATIONS {
            if !ctx.iteration(false) {
                break;
            }
        }
    }
    ok
}

/// Falls back to BabyDra apps, MIME defaults and desktop entries.
pub(super) fn fallback_open_file(path: &Path, uri: &str) -> bool {
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
        .map(|m| m.len() < SMALL_TEXT_FILE_LIMIT)
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
        if matches!(bin_name, "xdg-open" | "BabyDra" | "babydra-filedialog") {
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
