//! Process-tree evidence read from `/proc`.

use std::collections::HashSet;

use super::catalogue::{basename, push_unique, AppEntry};
use super::resolve::{entry_for_executable_hints, entry_identity, resolve_identity, WindowIdentity};

/// Do not follow an unbounded (or cyclic) `/proc/<pid>/status` PPid chain.
const MAX_PARENT_DEPTH: usize = 8;

/// Extracts the executable basename from a `/proc/<pid>/cmdline` style buffer.
pub(crate) fn app_name_from_cmdline_bytes(cmdline: &[u8]) -> Option<String> {
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

/// Executable hints for a pid, all read from `/proc`: argv[0] from `cmdline`,
/// the kernel's `/proc/<pid>/exe` target (which resolves `/proc/self/exe`
/// launchers), then `comm`. Never panics.
pub(super) fn process_executable_hints(process_id: u32) -> Vec<String> {
    let mut hints = Vec::new();
    if let Ok(cmdline) = std::fs::read(format!("/proc/{process_id}/cmdline")) {
        if let Some(name) = app_name_from_cmdline_bytes(&cmdline) {
            push_unique(&mut hints, name);
        }
    }
    if let Ok(path) = std::fs::read_link(format!("/proc/{process_id}/exe")) {
        let raw = path.to_string_lossy();
        let raw = raw.strip_suffix(" (deleted)").unwrap_or(&raw);
        let name = basename(raw);
        if !name.is_empty() {
            push_unique(&mut hints, name.to_ascii_lowercase());
        }
    }
    if let Ok(comm) = std::fs::read_to_string(format!("/proc/{process_id}/comm")) {
        let name = comm.trim();
        if !name.is_empty() {
            push_unique(&mut hints, name.to_ascii_lowercase());
        }
    }
    hints
}

/// Direct parent from `/proc/<pid>/status`, used only when the process itself
/// cannot be tied to an installed entry (for example a wrapper or zygote).
fn parent_pid(process_id: u32) -> Option<u32> {
    let status = std::fs::read_to_string(format!("/proc/{process_id}/status")).ok()?;
    status.lines().find_map(|line| {
        line.strip_prefix("PPid:")
            .and_then(|value| value.trim().parse::<u32>().ok())
            .filter(|pid| *pid > 0)
    })
}

/// Resolves the entry for a real pid. The parent chain is walked only when the
/// direct executable matches no installed entry, and stays inside the actual
/// process tree (bounded depth, no cycles).
fn resolve_entry_for_pid(entries: &[AppEntry], process_id: u32, app_id: &str) -> Option<AppEntry> {
    let mut current = (process_id > 0).then_some(process_id);
    let mut seen = HashSet::new();
    for _ in 0..MAX_PARENT_DEPTH {
        let pid = current?;
        if !seen.insert(pid) {
            return None;
        }
        let hints = process_executable_hints(pid);
        if let Some(entry) = entry_for_executable_hints(entries, &hints, app_id) {
            return Some(entry);
        }
        current = parent_pid(pid);
    }
    None
}

/// Resolves an identity with the process tree authoritative when a pid exists.
pub(super) fn resolve_identity_for_pid(
    entries: &[AppEntry],
    app_id: &str,
    title: &str,
    process_id: Option<u32>,
) -> WindowIdentity {
    process_id
        .and_then(|pid| resolve_entry_for_pid(entries, pid, app_id))
        .map(|entry| entry_identity(&entry))
        .unwrap_or_else(|| resolve_identity(entries, app_id, title, None))
}
