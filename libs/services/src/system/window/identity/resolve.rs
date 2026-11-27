//! Matching window evidence against installed desktop entries.

use super::catalogue::{
    basename, clean_app_id, executable_name, icon_token, normalise, push_unique, short_app_id,
    AppEntry,
};

/// Resolved window identity: an authoritative desktop entry when one could be
/// identified, plus ordered candidates for icon/theme lookup.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct WindowIdentity {
    pub(crate) name: String,
    pub(crate) candidates: Vec<String>,
    /// Installed desktop entry that supplied `name`, when one was identified.
    pub(crate) desktop_entry_id: Option<String>,
}

pub(super) fn find_by_id(entries: &[AppEntry], id: &str) -> Option<AppEntry> {
    let needle = normalise(clean_app_id(id));
    if needle.is_empty() {
        return None;
    }
    entries
        .iter()
        .find(|entry| normalise(&entry.id) == needle)
        .cloned()
}

/// Fallback only: an exact match on the *whole observed string*, never on a
/// title suffix. A suffix such as " - URL Handler" can never select an entry.
fn find_by_observed_text(entries: &[AppEntry], candidate: &str) -> Option<AppEntry> {
    let needle = normalise(candidate);
    if needle.is_empty() {
        return None;
    }
    entries
        .iter()
        .find(|entry| {
            normalise(&entry.name) == needle
                || (!executable_name(entry).is_empty() && executable_name(entry) == needle)
        })
        .cloned()
}

/// Picks the installed entry whose `Exec` matches the real process executable.
///
/// Several entries may share one executable (for example an app and its hidden
/// URL handler). Observed window data is used to break the tie: an entry whose
/// desktop id matches the Wayland app id, then a visible entry, then an id that
/// matches the executable itself. These are data-driven rankings; the returned
/// entry's `Name` is always copied verbatim.
pub(crate) fn find_by_executable(
    entries: &[AppEntry],
    executable: &str,
    app_id: &str,
) -> Option<AppEntry> {
    let needle = normalise(basename(executable));
    if needle.is_empty() {
        return None;
    }
    let observed_id = normalise(clean_app_id(app_id));
    entries
        .iter()
        .filter(|entry| executable_name(entry) == needle)
        .min_by_key(|entry| {
            let id = normalise(&entry.id);
            let id_matches_window = if observed_id.is_empty() || id != observed_id {
                1
            } else {
                0
            };
            let id_matches_exe = if id == needle { 0 } else { 1 };
            (
                id_matches_window,
                !entry.visible,
                id_matches_exe,
                entry.id.len(),
            )
        })
        .cloned()
}

pub(super) fn entry_identity(entry: &AppEntry) -> WindowIdentity {
    let mut candidates = Vec::new();
    push_unique(&mut candidates, clean_app_id(&entry.id).to_string());
    push_unique(&mut candidates, basename(&entry.executable).to_string());
    WindowIdentity {
        name: entry.name.clone(),
        candidates,
        desktop_entry_id: Some(clean_app_id(&entry.id).to_string()),
    }
}

/// Pure resolution core, with no title heuristics in the authoritative path.
///
/// Evidence, in priority order:
/// 1. the real process executable matched against installed desktop entries;
/// 2. the Wayland app id matched against installed desktop entries;
/// 3. the whole window title matched *exactly* against an installed entry;
/// 4. the raw app id / title, unchanged, when no entry is installed.
///
/// An authoritative entry is never trimmed, re-cased, suffixed or otherwise
/// rewritten: its installed `Name` is returned exactly as found.
pub(crate) fn resolve_identity(
    entries: &[AppEntry],
    app_id: &str,
    title: &str,
    process_executable: Option<&str>,
) -> WindowIdentity {
    if let Some(exe) = process_executable {
        if let Some(entry) = find_by_executable(entries, exe, app_id) {
            return entry_identity(&entry);
        }
    }
    if let Some(entry) = find_by_id(entries, app_id) {
        return entry_identity(&entry);
    }
    if let Some(entry) = find_by_observed_text(entries, title.trim()) {
        return entry_identity(&entry);
    }
    let id = clean_app_id(app_id);
    if !id.is_empty() {
        return WindowIdentity {
            name: app_id.trim().to_string(),
            candidates: vec![short_app_id(id).to_string()],
            desktop_entry_id: None,
        };
    }
    let whole = title.trim();
    WindowIdentity {
        name: whole.to_string(),
        candidates: vec![icon_token(whole)],
        desktop_entry_id: None,
    }
}

/// First usable executable hint from a list observed on the real process tree.
pub(crate) fn entry_for_executable_hints(
    entries: &[AppEntry],
    hints: &[String],
    app_id: &str,
) -> Option<AppEntry> {
    hints
        .iter()
        .find_map(|hint| find_by_executable(entries, hint, app_id))
}
