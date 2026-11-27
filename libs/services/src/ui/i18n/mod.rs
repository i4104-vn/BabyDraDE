//! Internationalization (i18n) support library for the BabyDra workspace.
//! Provides locale management, translations, and string formatting utilities
//! for English ("en") and Vietnamese ("vi") locales using service-specific JSON configurations.

use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::HashMap;

mod locales;
use locales::{load_en_map, load_vi_map};
use std::sync::{mpsc, OnceLock, RwLock};

/// Resolves the locale config file path (~/.babydra/locale.conf or fallback ~/.config/locale.conf).
pub fn get_locale_config_path() -> std::path::PathBuf {
    let babydra_file = crate::config::get_babydra_dir().join("locale.conf");
    if babydra_file.exists() {
        return babydra_file;
    }
    if let Some(config_dir) = dirs::config_dir() {
        let legacy_file = config_dir.join("locale.conf");
        if legacy_file.exists() {
            return legacy_file;
        }
    }
    babydra_file
}

/// Returns the default system locale determined from ~/.babydra/locale.conf, ~/.config/locale.conf or `LANG` environment variable.
fn default_locale() -> String {
    let locale_file = get_locale_config_path();
    if let Ok(content) = std::fs::read_to_string(&locale_file) {
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("LOCALE=") {
                let val = trimmed.trim_start_matches("LOCALE=").trim();
                if val == "vi" || val == "en" {
                    return val.to_string();
                }
            }
            if trimmed.starts_with("LANG=") || trimmed.starts_with("LANGUAGE=") {
                let lowered = trimmed.to_lowercase();
                if lowered.contains("vi") {
                    return "vi".to_string();
                }
                if lowered.contains("en") {
                    return "en".to_string();
                }
            }
        }
    }
    if let Ok(lang) = std::env::var("LANG") {
        if lang.to_lowercase().starts_with("vi") {
            return "vi".to_string();
        }
    }
    "en".to_string()
}

static CURRENT_LOCALE: OnceLock<RwLock<String>> = OnceLock::new();
static EN_MAP: OnceLock<HashMap<String, String>> = OnceLock::new();
static VI_MAP: OnceLock<HashMap<String, String>> = OnceLock::new();

/// Retrieves the current active system locale ("vi" or "en").
///
/// A poisoned lock is treated as non-fatal: a warning is logged and the
/// default locale is returned so translation never crashes the UI.
///
/// # Returns
/// The active locale code (`"vi"` or `"en"`).
pub fn get_locale() -> String {
    let lock = CURRENT_LOCALE.get_or_init(|| RwLock::new(default_locale()));
    read_locale(lock)
}

/// Reads a locale lock, falling back to the default locale if it is poisoned.
fn read_locale(lock: &RwLock<String>) -> String {
    match lock.read() {
        Ok(guard) => guard.clone(),
        Err(_) => {
            tracing::warn!("locale lock poisoned; using default locale");
            default_locale()
        }
    }
}

/// Sets the current active system locale.
pub fn set_locale(locale: &str) {
    let normalized = if locale == "en" { "en" } else { "vi" };

    let lock = CURRENT_LOCALE.get_or_init(|| RwLock::new(normalized.to_string()));
    if let Ok(mut writer) = lock.write() {
        *writer = normalized.to_string();
    }
}

/// Persist locale to ~/.babydra/locale.conf and ~/.config/locale.conf and update env vars.
pub fn persist_locale(locale: &str) {
    let normalized = if locale == "en" { "en" } else { "vi" };
    let lang_str = if normalized == "vi" {
        "vi_VN.UTF-8"
    } else {
        "en_US.UTF-8"
    };

    std::env::set_var("LANG", lang_str);
    std::env::set_var("LANGUAGE", lang_str);
    set_locale(normalized);

    let content = format!("LANG={lang_str}\nLANGUAGE={lang_str}\nLOCALE={normalized}\n");

    let babydra_dir = crate::config::get_babydra_dir();
    let _ = std::fs::create_dir_all(&babydra_dir);
    let _ = std::fs::write(babydra_dir.join("locale.conf"), &content);

    if let Some(config_dir) = dirs::config_dir() {
        let _ = std::fs::create_dir_all(&config_dir);
        let locale_file = config_dir.join("locale.conf");
        let _ = std::fs::write(&locale_file, &content);
    }
}

/// Watch ~/.babydra/locale.conf (and fallback) and send changed locale values to the caller.
/// UI crates can bridge this receiver onto their own event loop.
pub fn watch_locale_changes() -> mpsc::Receiver<String> {
    let (tx, rx) = mpsc::channel();
    let path = get_locale_config_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if !path.exists() {
        let _ = std::fs::write(&path, "LANG=en_US.UTF-8\nLANGUAGE=en_US.UTF-8\nLOCALE=en\n");
    }

    std::thread::Builder::new()
        .name("babydra-locale-watcher".into())
        .spawn(move || {
            let (event_tx, event_rx) = mpsc::channel();
            let Ok(mut watcher): Result<RecommendedWatcher, _> =
                notify::recommended_watcher(move |_| {
                    let _ = event_tx.send(());
                })
            else {
                return;
            };
            if watcher.watch(&path, RecursiveMode::NonRecursive).is_err() {
                return;
            }
            let mut last_locale = get_locale();
            while event_rx.recv().is_ok() {
                let new_locale = default_locale();
                if new_locale != last_locale {
                    last_locale = new_locale.clone();
                    set_locale(&new_locale);
                    if tx.send(new_locale).is_err() {
                        break;
                    }
                }
            }
        })
        .ok();

    rx
}

/// Translates a given key into the current active locale's string.
/// If the key is not found, returns the key itself.
pub fn trans(key: &str) -> String {
    let locale = get_locale();
    let map = match locale.as_str() {
        "en" => EN_MAP.get_or_init(load_en_map),
        _ => VI_MAP.get_or_init(load_vi_map),
    };
    map.get(key).cloned().unwrap_or_else(|| key.to_string())
}

#[cfg(test)]
mod tests;
