//! Installed desktop-entry catalogue, read from `gio` and cached briefly.

use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};

use gio::prelude::*;

/// Installed entries change rarely; avoid rescanning `AppInfo::all()` on every
/// window/title event.
const CATALOGUE_TTL: Duration = Duration::from_secs(30);

/// Minimal, display-independent view of an installed desktop entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AppEntry {
    pub(crate) id: String,
    /// The entry's `Name` exactly as installed. Never rewritten.
    pub(crate) name: String,
    pub(crate) executable: String,
    /// `NoDisplay`/`TryExec` state, used only to rank equally good executable
    /// matches. It never changes the name that is returned.
    pub(crate) visible: bool,
}

pub(super) fn clean_app_id(app_id: &str) -> &str {
    let trimmed = app_id.trim();
    trimmed.strip_suffix(".desktop").unwrap_or(trimmed)
}

pub(super) fn short_app_id(app_id: &str) -> &str {
    clean_app_id(app_id).rsplit('.').next().unwrap_or("")
}

/// Case/punctuation-insensitive form used to compare ids and executables. It is
/// only ever used for *matching*; displayed names are copied verbatim.
pub(super) fn normalise(value: &str) -> String {
    value
        .trim()
        .to_ascii_lowercase()
        .chars()
        .filter(char::is_ascii_alphanumeric)
        .collect()
}

/// Turns an observed string into a plausible icon-theme name (fallback only).
pub(super) fn icon_token(value: &str) -> String {
    let lower = value.trim().to_ascii_lowercase().replace([' ', '_'], "-");
    lower
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-')
        .collect()
}

pub(super) fn basename(value: &str) -> &str {
    let trimmed = value.trim();
    trimmed.rsplit(['/', '\\']).next().unwrap_or(trimmed).trim()
}

pub(super) fn executable_name(entry: &AppEntry) -> String {
    normalise(basename(&entry.executable))
}

/// Reads the installed application catalogue from desktop entries. No app data
/// is baked into this crate: `gio` is the single source of truth for names,
/// executables and icons.
fn load_installed_apps() -> Vec<AppEntry> {
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
                visible: info.should_show(),
            })
        })
        .collect()
}

struct CachedCatalogue {
    entries: Arc<Vec<AppEntry>>,
    loaded_at: Instant,
}

static CATALOGUE: OnceLock<Mutex<Option<CachedCatalogue>>> = OnceLock::new();

/// Pure TTL check for the installed-entry catalogue.
///
/// `now` is injected so refresh timing can be tested without sleeping.
pub(crate) fn catalogue_due(loaded_at: Option<Instant>, now: Instant, ttl: Duration) -> bool {
    loaded_at.is_none_or(|at| now.saturating_duration_since(at) >= ttl)
}

/// Returns the installed-entry catalogue, refreshing it at most once per TTL.
///
/// A poisoned lock is recovered after a warning so a single panicking caller
/// cannot disable application-name resolution for the rest of the session.
pub(super) fn installed_apps() -> Arc<Vec<AppEntry>> {
    let slot = CATALOGUE.get_or_init(|| Mutex::new(None));
    let mut guard = slot.lock().unwrap_or_else(|err| {
        tracing::warn!("application catalogue lock poisoned; recovering");
        err.into_inner()
    });
    let now = Instant::now();
    if catalogue_due(
        guard.as_ref().map(|cached| cached.loaded_at),
        now,
        CATALOGUE_TTL,
    ) {
        *guard = Some(CachedCatalogue {
            entries: Arc::new(load_installed_apps()),
            loaded_at: now,
        });
    }
    guard
        .as_ref()
        .map(|cached| Arc::clone(&cached.entries))
        .unwrap_or_default()
}

pub(super) fn push_unique(list: &mut Vec<String>, value: String) {
    if !value.is_empty()
        && !list
            .iter()
            .any(|existing| existing.eq_ignore_ascii_case(&value))
    {
        list.push(value);
    }
}
