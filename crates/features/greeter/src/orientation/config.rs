//! Locates and loads the display configuration for the greeter session.

use std::{
    path::{Path, PathBuf},
    time::SystemTime,
};

use services::config::{load_display_config_from, models::DisplayConfig};

/// Locates the most recently modified readable display config.
///
/// The greeter's home is usually `/`, so the last logged-in user's config is
/// preferred, with any user under `/home` as a fallback.
pub(super) fn find_config() -> Option<PathBuf> {
    let mut candidates = Vec::new();
    if let Some(user) = crate::widgets::get_last_user() {
        if valid_username(&user) {
            if let Some(home) = user_home(
                &user,
                &std::fs::read_to_string("/etc/passwd").unwrap_or_default(),
            ) {
                candidates.push(home);
            }
        }
    }
    candidates.push(services::config::get_home_dir());
    if let Ok(entries) = std::fs::read_dir("/home") {
        candidates.extend(
            entries
                .flatten()
                .map(|entry| entry.path())
                .filter(|path| path.is_dir()),
        );
    }

    let mut best: Option<(SystemTime, PathBuf)> = None;
    for home in candidates {
        let path = home.join(".babydra/configs/display.toml");
        let Ok(metadata) = std::fs::metadata(&path) else {
            continue;
        };
        let Ok(modified) = metadata.modified() else {
            continue;
        };
        if best.as_ref().is_none_or(|(time, _)| modified > *time) {
            best = Some((modified, path));
        }
    }
    best.map(|(_, path)| path).map(|path| {
        tracing::info!("greeter: using display config {path:?}");
        path
    })
}

/// Loads the display config from a resolved path.
pub(super) fn load_config(path: &Path) -> DisplayConfig {
    load_display_config_from(path)
}

/// Returns true for names that are safe to match in the passwd database.
fn valid_username(user: &str) -> bool {
    !user.is_empty()
        && user
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
}

/// Resolves a user's home directory from passwd content.
pub(super) fn user_home(user: &str, passwd: &str) -> Option<PathBuf> {
    for line in passwd.lines() {
        let fields: Vec<&str> = line.split(':').collect();
        if fields.first() == Some(&user) && fields.len() > 5 {
            return Some(PathBuf::from(fields[5]));
        }
    }
    None
}
