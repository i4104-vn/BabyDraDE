use std::collections::HashSet;

use super::{connection_process_id, service_matches_app_id};

/// Standard locations used by GTK, Qt/KDE, Chromium and Electron app-menu
/// exporters. A service may use any one of them, so all are probed.
const CANDIDATE_PATHS: &[&str] = &[
    "/MenuBar",
    "/MenuBar/1",
    "/MenuBar/0",
    "/menu",
    "/menubar",
    "/dbusmenu",
    "/com/canonical/menu",
    "/com/canonical/menu/1",
    "/com/canonical/AppMenu/topbar",
    "/org/appmenu/menu",
    "/org/chromium/DbusMenu",
    "/org/chromium/DbusMenu/1",
    "/KMenuBar",
];
pub(super) const MAX_SERVICES: usize = 18;
pub(super) const MAX_PATHS: usize = 16;

#[derive(Debug, Clone)]
pub(super) struct Candidate {
    pub(super) service: String,
    /// Empty when the standard paths should be probed.
    pub(super) registered_path: String,
    /// Lower is more trustworthy (registered menu before name guesses).
    pub(super) priority: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct MenuResult {
    pub(super) score: usize,
    pub(super) priority: u8,
}

impl MenuResult {
    pub(super) fn is_better_than(self, other: Option<MenuResult>) -> bool {
        match other {
            Some(other) => {
                self.score > other.score
                    || (self.score == other.score && self.priority < other.priority)
            }
            None => true,
        }
    }
}

pub(super) fn build_candidates(
    names: &[String],
    registered: &[(String, String)],
    app_id: &str,
    process_id: Option<u32>,
    conn: &gio::DBusConnection,
) -> Vec<Candidate> {
    let mut candidates = Vec::new();
    let mut seen = HashSet::new();

    let mut push = |service: String, path: String, priority: u8| {
        if service.is_empty() {
            return;
        }
        let key = format!("{service}\0{path}");
        if seen.insert(key) {
            candidates.push(Candidate {
                service,
                registered_path: path,
                priority,
            });
        }
    };

    // Registrar entries are authoritative: they were published by the app.
    // Unrelated registrations are excluded so an empty/wrong menu cannot win.
    for (service, path) in registered {
        let belongs_to_app = service_matches_app_id(service, app_id)
            || process_id.is_some_and(|pid| connection_process_id(conn, service) == Some(pid));
        if belongs_to_app {
            push(service.clone(), path.clone(), 0);
        }
    }

    for name in names {
        if name.starts_with(':') {
            continue;
        }
        if service_matches_app_id(name, app_id) {
            push(name.clone(), String::new(), 1);
        }
    }

    // KDE's application-menu aggregator exports the focused window's menubar.
    // It is a provider, not an application name, and is used as a last resort.
    for name in names {
        let lower = name.to_ascii_lowercase();
        if lower == "org.kde.kappmenu"
            || lower.starts_with("org.kde.kappmenu.")
            || lower == "org.kde.appmenu"
            || lower.starts_with("org.kde.appmenu.")
        {
            push(name.clone(), String::new(), 3);
        }
    }

    if let Some(pid) = process_id {
        for name in names {
            if connection_process_id(conn, name) == Some(pid) {
                let priority = if name.starts_with(':') { 2 } else { 1 };
                push(name.clone(), String::new(), priority);
            }
        }
    }

    candidates
}

pub(super) fn candidate_paths(candidate: &Candidate) -> Vec<&str> {
    let mut paths: Vec<&str> = Vec::with_capacity(CANDIDATE_PATHS.len() + 1);
    if !candidate.registered_path.is_empty() {
        paths.push(&candidate.registered_path);
    }
    paths.extend_from_slice(CANDIDATE_PATHS);
    paths.truncate(MAX_PATHS);
    paths
}
