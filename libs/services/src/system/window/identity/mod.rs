//! Resolves a window's application identity from installed desktop entries and
//! process evidence, without rewriting installed display names.

pub(crate) mod cache;
pub(crate) mod catalogue;
pub(crate) mod process;
pub(crate) mod resolve;

use cache::resolve_window_identity;
use catalogue::{clean_app_id, installed_apps};
use process::process_executable_hints;
use resolve::find_by_id;

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
pub fn is_shared_app_id(app_id: &str) -> bool {
    let id = clean_app_id(app_id);
    id.is_empty() || find_by_id(installed_apps().as_slice(), id).is_none()
}

/// Last-resort executable hint from `/proc/<pid>` (cmdline, exe, then comm).
pub fn get_process_app_name(process_id: u32) -> Option<String> {
    process_executable_hints(process_id).into_iter().next()
}

/// Ordered app ids/names to try when resolving a window's display name or icon.
/// Derived from installed desktop entries plus the actual window/process data.
pub fn get_window_app_candidates(
    app_id: &str,
    title: &str,
    process_id: Option<u32>,
) -> Vec<String> {
    resolve_window_identity(app_id, title, process_id).candidates
}

/// Picks the desktop id for a window from the same evidence used for its display
/// name, so the label and the icon cannot disagree.
pub fn get_window_icon_id(app_id: &str, title: &str, process_id: Option<u32>) -> String {
    get_window_app_candidates(app_id, title, process_id)
        .first()
        .cloned()
        .unwrap_or_else(|| clean_app_id(app_id).to_string())
}

/// Returns the installed desktop id that supplied the window's name, if any.
/// UI helpers use this to take that entry's *exact* icon rather than searching
/// by title-derived candidates.
pub fn get_window_desktop_entry_id(
    app_id: &str,
    title: &str,
    process_id: Option<u32>,
) -> Option<String> {
    resolve_window_identity(app_id, title, process_id).desktop_entry_id
}

/// Formats an application name from app id and title.
///
/// Without a pid this falls back to installed-entry app id / exact title matches
/// and then to the raw window data; it never derives names from title suffixes.
pub fn get_app_display_name(app_id: &str, title: &str) -> String {
    get_app_display_name_with_pid(app_id, title, None)
}

/// Returns the exact installed `Name` of the desktop entry belonging to the
/// window's real process. Title and app id cannot override that entry.
pub fn get_app_display_name_with_pid(app_id: &str, title: &str, process_id: Option<u32>) -> String {
    resolve_window_identity(app_id, title, process_id).name
}
