//! Active window tracking and window actions for BabyDraDE features.
//!
//! Focused-window metadata is published as a retained `window.active` event on
//! the in-memory protocol bus. Window actions are requests to the compositor's
//! `wm` peer. Peers exchange framed JSON over a uid-scoped abstract Unix socket;
//! a router in the compositor owns delivery and retention. No files or queue
//! directories are involved.

use std::io;
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, OnceLock};
use std::time::Duration;

use gio::prelude::*;

use crate::protocol::{Bus, Event, Incoming, Request, WindowAction, WindowTarget};

pub use common::protocol::ActiveWindowInfo;

/// Queries the focused window, falling back to `wlrctl` when the bus has no
/// cached snapshot yet.
pub fn get_active_window() -> Option<ActiveWindowInfo> {
    ensure_active_watcher();
    active_cache()
        .lock()
        .ok()
        .and_then(|cache| cache.clone())
        .or_else(query_wlrctl_active_window)
}

/// Reads the focused window from `wlrctl` as a one-shot fallback.
///
/// # Returns
/// The focused window's app id and title, or `None` when `wlrctl` is absent or
/// reports no focused window.
fn query_wlrctl_active_window() -> Option<ActiveWindowInfo> {
    let output = Command::new("wlrctl")
        .args(["window", "list", "state:focused"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let line = stdout.lines().next()?;
    let (app_id, title) = line.split_once(':')?;
    let app_id = app_id.trim().to_string();
    let title = title.trim().to_string();
    if app_id.is_empty() && title.is_empty() {
        return None;
    }
    Some(ActiveWindowInfo {
        app_id,
        title,
        process_id: None,
    })
}

/// Minimal, display-independent view of an installed desktop entry.
#[derive(Debug, Clone, PartialEq, Eq)]
struct AppEntry {
    id: String,
    name: String,
    executable: String,
}

/// Resolved window identity: an authoritative desktop entry when one could be
/// identified, plus ordered candidates for icon/theme lookup.
#[derive(Debug, Clone, PartialEq, Eq)]
struct WindowIdentity {
    name: String,
    candidates: Vec<String>,
}

const EXACT_MATCH: u32 = 100;
const WORD_MATCH: u32 = 70;
const PARTIAL_MATCH: u32 = 50;

fn clean_app_id(app_id: &str) -> &str {
    let trimmed = app_id.trim();
    trimmed.strip_suffix(".desktop").unwrap_or(trimmed)
}

fn short_app_id(app_id: &str) -> &str {
    clean_app_id(app_id).rsplit('.').next().unwrap_or("")
}

/// Case/punctuation-insensitive form used to compare names, ids and commands.
fn normalise(value: &str) -> String {
    value
        .trim()
        .to_ascii_lowercase()
        .chars()
        .filter(char::is_ascii_alphanumeric)
        .collect()
}

/// Turns a human-readable title part into a plausible icon-theme name.
fn icon_token(value: &str) -> String {
    let lower = value.trim().to_ascii_lowercase().replace([' ', '_'], "-");
    lower
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-')
        .collect()
}

fn basename(value: &str) -> &str {
    let trimmed = value.trim();
    trimmed.rsplit(['/', '\\']).next().unwrap_or(trimmed).trim()
}

fn words(value: &str) -> Vec<String> {
    value
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|word| !word.is_empty())
        .map(|word| word.to_ascii_lowercase())
        .collect()
}

fn executable_name(entry: &AppEntry) -> String {
    normalise(basename(&entry.executable))
}

/// Reads the installed application catalogue from desktop entries. No app data
/// is baked into this crate: `gio` is the single source of truth for names,
/// executables and icons.
fn installed_apps() -> Vec<AppEntry> {
    gio::AppInfo::all()
        .into_iter()
        .filter_map(|info| {
            let id = info.id()?.to_string();
            let name = info.name().to_string();
            let executable = info.executable().to_string_lossy().to_string();
            let executable = if executable.is_empty() {
                info.commandline()
                    .map(|command| command.to_string_lossy().to_string())
                    .unwrap_or_default()
            } else {
                executable
            };
            Some(AppEntry {
                id: clean_app_id(&id).to_string(),
                name,
                executable,
            })
        })
        .collect()
}

fn capitalise(value: &str) -> String {
    let mut chars = value.chars();
    match chars.next() {
        Some(first) => format!("{}{}", first.to_uppercase(), chars.as_str()),
        None => String::new(),
    }
}

fn push_unique(list: &mut Vec<String>, value: String) {
    if !value.is_empty() && !list.iter().any(|existing| existing.eq_ignore_ascii_case(&value)) {
        list.push(value);
    }
}

/// Extracts the trailing app part of a conventional window title.
///
/// Both ` - ` and em/en dash separators are supported; the last separator wins
/// because titles usually put the document first and the application last.
pub fn get_title_app_part(title: &str) -> Option<String> {
    let normalised = title.replace(['—', '–'], "-");
    normalised
        .rsplit_once(" - ")
        .map(|(_, part)| part.trim().to_string())
        .filter(|part| !part.is_empty())
}

/// Returns true when an app id cannot be tied to any installed desktop entry.
/// Such an id cannot identify an application by itself, so the title/process
/// evidence takes precedence in the resolution helpers below.
pub fn is_shared_app_id(app_id: &str) -> bool {
    let id = clean_app_id(app_id);
    id.is_empty() || find_by_id(&installed_apps(), id).is_none()
}

fn find_by_id(entries: &[AppEntry], id: &str) -> Option<AppEntry> {
    let needle = normalise(clean_app_id(id));
    if needle.is_empty() {
        return None;
    }
    entries
        .iter()
        .find(|entry| normalise(&entry.id) == needle)
        .cloned()
}

fn find_by_executable(entries: &[AppEntry], executable: &str) -> Option<AppEntry> {
    let needle = normalise(basename(executable));
    if needle.is_empty() {
        return None;
    }
    entries
        .iter()
        .find(|entry| executable_name(entry) == needle)
        .cloned()
}

/// Scores how strongly a title/app string identifies a desktop entry, using
/// only the entry's installed display name and executable.
fn match_score(candidate: &str, entry: &AppEntry) -> u32 {
    let needle = normalise(candidate);
    if needle.is_empty() {
        return 0;
    }
    let name = normalise(&entry.name);
    let executable = executable_name(entry);
    if needle == name {
        return EXACT_MATCH;
    }
    if !executable.is_empty() && needle == executable {
        return EXACT_MATCH - 5;
    }
    if !executable.is_empty() && words(candidate).iter().any(|word| normalise(word) == executable) {
        return WORD_MATCH;
    }
    if !executable.is_empty() && needle.contains(&executable) {
        return PARTIAL_MATCH;
    }
    if !name.is_empty() && (needle.contains(&name) || name.contains(&needle)) {
        return PARTIAL_MATCH;
    }
    0
}

fn find_best(entries: &[AppEntry], candidate: &str, minimum: u32) -> Option<(u32, AppEntry)> {
    let mut best: Option<(u32, AppEntry)> = None;
    for entry in entries {
        let score = match_score(candidate, entry);
        if score < minimum {
            continue;
        }
        if best.as_ref().is_none_or(|(current, _)| score > *current) {
            best = Some((score, entry.clone()));
        }
    }
    best
}

fn entry_identity(entry: &AppEntry) -> WindowIdentity {
    let mut candidates = Vec::new();
    push_unique(&mut candidates, clean_app_id(&entry.id).to_string());
    push_unique(&mut candidates, basename(&entry.executable).to_string());
    WindowIdentity {
        name: entry.name.clone(),
        candidates,
    }
}

/// Pure resolution core. Evidence, in priority order:
/// 1. the actual process executable matched against installed desktop entries;
/// 2. the window title's trailing app part matched against installed entries;
/// 3. the Wayland app id matched against installed entries;
/// 4. the raw title/app id when no entry is installed (legacy fallback).
///
/// A title that names a different installed app wins over a generic app id or
/// a shared shell executable, so a window is never mislabelled as the shell it
/// happens to run on. Every name and icon candidate comes from observed data
/// (desktop entries, `/proc`, or the window itself) — never a fixed list.
fn resolve_identity(
    entries: &[AppEntry],
    app_id: &str,
    title: &str,
    process_executable: Option<&str>,
) -> WindowIdentity {
    let id = clean_app_id(app_id);
    let by_id = find_by_id(entries, id);
    let by_process = process_executable.and_then(|exe| find_by_executable(entries, exe));

    let title_part = get_title_app_part(title);
    let by_title = match &title_part {
        Some(part) => find_best(entries, part, PARTIAL_MATCH),
        None => find_best(entries, title.trim(), EXACT_MATCH),
    };

    if let Some(process_entry) = by_process {
        if let Some((score, title_entry)) = by_title {
            if score >= EXACT_MATCH && title_entry.id != process_entry.id {
                return entry_identity(&title_entry);
            }
        }
        return entry_identity(&process_entry);
    }

    if let Some((score, title_entry)) = by_title {
        let different_from_id = by_id
            .as_ref()
            .is_none_or(|id_entry| id_entry.id != title_entry.id);
        if score >= EXACT_MATCH || different_from_id {
            return entry_identity(&title_entry);
        }
    }

    if let Some(id_entry) = by_id {
        // Even when the IDE has no desktop entry of its own, a process that is
        // clearly not the app named by the shared id plus a title naming a
        // different app is enough to avoid mislabelling the window.
        if let (Some(exe), Some(part)) = (process_executable, &title_part) {
            let process_name = normalise(basename(exe));
            let entry_name = executable_name(&id_entry);
            let part_name = normalise(part);
            if !process_name.is_empty()
                && !entry_name.is_empty()
                && process_name != entry_name
                && !part_name.is_empty()
                && part_name != normalise(&id_entry.name)
            {
                let mut candidates = Vec::new();
                push_unique(&mut candidates, basename(exe).to_ascii_lowercase());
                push_unique(&mut candidates, icon_token(part));
                return WindowIdentity {
                    name: part.clone(),
                    candidates,
                };
            }
        }
        return entry_identity(&id_entry);
    }

    if let Some(part) = title_part {
        return WindowIdentity {
            name: part.clone(),
            candidates: vec![icon_token(&part)],
        };
    }
    if !id.is_empty() {
        let short = short_app_id(id).to_string();
        return WindowIdentity {
            name: capitalise(&short),
            candidates: vec![short],
        };
    }
    let whole = title.trim();
    WindowIdentity {
        name: whole.to_string(),
        candidates: vec![icon_token(whole)],
    }
}

/// Extracts the executable basename from a `/proc/<pid>/cmdline` style buffer.
fn app_name_from_cmdline_bytes(cmdline: &[u8]) -> Option<String> {
    let first = cmdline
        .split(|byte| *byte == 0)
        .find(|part| !part.is_empty())?;
    let path = String::from_utf8_lossy(first);
    let name = basename(&path);
    if name.is_empty() {
        return None;
    }
    Some(name.to_ascii_lowercase())
}

/// Last-resort executable hint from `/proc/<pid>` (cmdline, then comm). Never
/// panics and returns `None` when the process is gone or unreadable.
pub fn get_process_app_name(process_id: u32) -> Option<String> {
    if let Ok(cmdline) = std::fs::read(format!("/proc/{process_id}/cmdline")) {
        if let Some(name) = app_name_from_cmdline_bytes(&cmdline) {
            return Some(name);
        }
    }
    let comm = std::fs::read_to_string(format!("/proc/{process_id}/comm")).ok()?;
    let name = comm.trim();
    (!name.is_empty()).then(|| name.to_ascii_lowercase())
}

/// Ordered app ids/names to try when resolving a window's display name or icon.
/// Derived from installed desktop entries plus the actual window/process data.
pub fn get_window_app_candidates(
    app_id: &str,
    title: &str,
    process_id: Option<u32>,
) -> Vec<String> {
    let process_executable = process_id.and_then(get_process_app_name);
    resolve_identity(
        &installed_apps(),
        app_id,
        title,
        process_executable.as_deref(),
    )
    .candidates
}

/// Picks the most likely desktop id for a window from the same evidence used
/// for its display name, so the label and the icon cannot disagree.
pub fn get_window_icon_id(app_id: &str, title: &str, process_id: Option<u32>) -> String {
    let candidates = get_window_app_candidates(app_id, title, process_id);
    candidates
        .first()
        .cloned()
        .unwrap_or_else(|| clean_app_id(app_id).to_string())
}

/// Formats a user-friendly application name from app id and title.
///
/// Installed desktop entries provide authoritative names; a title naming a
/// different app wins over the shared id. Without any installed entry the
/// historical app-id/title fallback is used.
pub fn get_app_display_name(app_id: &str, title: &str) -> String {
    get_app_display_name_with_pid(app_id, title, None)
}

/// Variant of [`get_app_display_name`] that also uses the process executable
/// when the compositor supplied a process id.
pub fn get_app_display_name_with_pid(
    app_id: &str,
    title: &str,
    process_id: Option<u32>,
) -> String {
    let process_executable = process_id.and_then(get_process_app_name);
    resolve_identity(
        &installed_apps(),
        app_id,
        title,
        process_executable.as_deref(),
    )
    .name
}
/// Connects (once) to the protocol bus as the services peer.
///
/// # Returns
/// The shared bus handle.
///
/// # Errors
/// Returns the socket error raised while connecting.
fn bus() -> io::Result<&'static Bus> {
    static BUS: OnceLock<Bus> = OnceLock::new();
    if BUS.get().is_none() {
        let _ = BUS.set(Bus::connect("org.babydrade.services")?);
    }
    BUS.get()
        .ok_or_else(|| io::Error::other("protocol bus unavailable"))
}

/// Returns the in-memory cache of the last retained active-window event.
fn active_cache() -> &'static Mutex<Option<ActiveWindowInfo>> {
    static ACTIVE: OnceLock<Mutex<Option<ActiveWindowInfo>>> = OnceLock::new();
    ACTIVE.get_or_init(|| Mutex::new(None))
}

/// Starts the single background reader that feeds the active-window cache.
///
/// Each call retries while the compositor is unreachable; only the first
/// successful subscription spawns the reader, so one watcher feeds the cache
/// for the lifetime of the process.
fn ensure_active_watcher() {
    static SPAWNED: AtomicBool = AtomicBool::new(false);
    if SPAWNED.load(Ordering::Relaxed) {
        return;
    }

    // Retry until the compositor is reachable: a failed first attempt must not
    // permanently disable the cache.
    let Ok(bus) = bus() else {
        return;
    };
    if bus.subscribe("window.active").is_err() {
        return;
    }
    if SPAWNED.swap(true, Ordering::Relaxed) {
        return;
    }

    std::thread::spawn(move || loop {
        match bus.recv_timeout(Duration::from_secs(1)) {
            Some(Incoming::Event(message)) if message.topic.as_deref() == Some("window.active") => {
                if let Ok(Event::ActiveWindowChanged { info }) = message.decode::<Event>() {
                    if let Ok(mut cache) = active_cache().lock() {
                        *cache = Some(info);
                    }
                }
            }
            Some(Incoming::Disconnected) => break,
            _ => {}
        }
    });
}

/// Sends one request to the compositor's `wm` peer, ignoring transport errors.
///
/// # Arguments
/// * `request` - The window-manager service request to deliver.
fn send_request(request: &Request) {
    if let Ok(bus) = bus() {
        let _ = bus.request("wm", request);
    }
}

/// Sends a window action through the protocol bus.
///
/// # Arguments
/// * `action` - Legacy action name (`close`, `minimize`, `maximize`, `fullscreen`).
pub fn send_window_action(action: &str) {
    let Some(action) = WindowAction::parse(action) else {
        return;
    };
    send_request(&Request::WindowAction { action });
}

/// Closes the window identified by `app_id` or `title`, or the focused one.
///
/// # Arguments
/// * `app_id` - Wayland app id to match, preferred when non-empty.
/// * `title` - Window title to match when `app_id` is empty.
pub fn close_window_by_app(app_id: &str, title: &str) {
    let target = if !app_id.is_empty() {
        WindowTarget::AppId(app_id.to_string())
    } else if !title.is_empty() {
        WindowTarget::Title(title.to_string())
    } else {
        WindowTarget::Focused
    };
    send_request(&Request::CloseTargeted { target });
}

/// Minimizes the currently focused window.
pub fn minimize_active_window() {
    send_request(&Request::WindowAction {
        action: WindowAction::Minimize,
    });
}

/// Maximizes the currently focused window.
pub fn maximize_active_window() {
    send_request(&Request::WindowAction {
        action: WindowAction::Maximize,
    });
}

/// Toggles fullscreen on the currently focused window.
pub fn toggle_fullscreen_active_window() {
    send_request(&Request::WindowAction {
        action: WindowAction::Fullscreen,
    });
}
#[cfg(test)]
mod tests {
    use super::*;

    /// Synthetic desktop entries only: the production code has no app catalog.
    fn catalog() -> Vec<AppEntry> {
        vec![
            AppEntry {
                id: "alpha-shell".to_string(),
                name: "Alpha Shell".to_string(),
                executable: "/usr/bin/alpha-shell".to_string(),
            },
            AppEntry {
                id: "beta".to_string(),
                name: "Beta Editor".to_string(),
                executable: "/usr/bin/beta".to_string(),
            },
        ]
    }

    #[test]
    fn title_identifies_app_behind_shared_id() {
        let identity = resolve_identity(&catalog(), "alpha-shell", "main.rs — Beta Editor", None);
        assert_eq!(identity.name, "Beta Editor");
        assert_eq!(identity.candidates.first().map(String::as_str), Some("beta"));
    }

    #[test]
    fn process_executable_is_strong_evidence() {
        let identity = resolve_identity(&catalog(), "alpha-shell", "untitled document", Some("beta"));
        assert_eq!(identity.name, "Beta Editor");
    }

    #[test]
    fn exact_title_name_beats_shared_shell_executable() {
        let identity = resolve_identity(
            &catalog(),
            "alpha-shell",
            "main.rs — Beta Editor",
            Some("alpha-shell"),
        );
        assert_eq!(identity.name, "Beta Editor");
    }

    #[test]
    fn app_id_still_identifies_its_app() {
        let identity = resolve_identity(&catalog(), "alpha-shell", "some document", None);
        assert_eq!(identity.name, "Alpha Shell");
    }

    #[test]
    fn unknown_app_keeps_legacy_id_name() {
        let identity = resolve_identity(&catalog(), "org.example.gamma", "user@host: ~", None);
        assert_eq!(identity.name, "Gamma");
    }

    #[test]
    fn empty_id_uses_title() {
        let identity = resolve_identity(&catalog(), "", "main.rs — Beta Editor", None);
        assert_eq!(identity.name, "Beta Editor");
    }

    #[test]
    fn process_and_title_override_shared_id_without_own_entry() {
        let identity = resolve_identity(
            &catalog(),
            "alpha-shell",
            "main.rs — Delta Viewer",
            Some("delta"),
        );
        assert_eq!(identity.name, "Delta Viewer");
        assert_eq!(identity.candidates.first().map(String::as_str), Some("delta"));
    }

    #[test]
    fn unknown_app_without_entry_uses_title_text() {
        let identity = resolve_identity(&[], "alpha-shell", "main.rs — Delta Viewer", None);
        assert_eq!(identity.name, "Delta Viewer");
        assert_eq!(identity.candidates.first().map(String::as_str), Some("delta-viewer"));
    }

    #[test]
    fn title_part_uses_last_separator() {
        assert_eq!(
            get_title_app_part("a — b - Beta Editor").as_deref(),
            Some("Beta Editor")
        );
        assert_eq!(get_title_app_part("no separator here"), None);
    }

    #[test]
    fn executable_match_uses_basename() {
        let apps = catalog();
        assert_eq!(
            find_by_executable(&apps, "/usr/bin/beta").map(|entry| entry.name),
            Some("Beta Editor".to_string())
        );
    }

    #[test]
    fn cmdline_parser_extracts_executable_basename() {
        assert_eq!(
            app_name_from_cmdline_bytes(b"/usr/bin/beta\0--new-window\0").as_deref(),
            Some("beta")
        );
        assert_eq!(app_name_from_cmdline_bytes(b"\0").as_deref(), None);
        assert_eq!(app_name_from_cmdline_bytes(b"").as_deref(), None);
    }
}
