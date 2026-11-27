//! Unit tests for window identity resolution, using only synthetic entries.

use std::time::{Duration, Instant};

use crate::system::window::get_title_app_part;
use crate::system::window::identity::cache::{IdentityCache, IdentityKey};
use crate::system::window::identity::catalogue::AppEntry;
use crate::system::window::identity::process::app_name_from_cmdline_bytes;
use crate::system::window::identity::resolve::{
    entry_for_executable_hints, find_by_executable, resolve_identity, WindowIdentity,
};

fn entry(id: &str, name: &str, executable: &str, visible: bool) -> AppEntry {
    AppEntry {
        id: id.to_string(),
        name: name.to_string(),
        executable: executable.to_string(),
        visible,
    }
}

/// Synthetic desktop entries only: the production code has no app catalog.
fn catalog() -> Vec<AppEntry> {
    vec![
        entry("alpha-shell", "Alpha Shell", "/usr/bin/alpha-shell", true),
        entry("beta", "Beta Editor", "/usr/bin/beta", true),
        entry("chromium", "Chromium", "/usr/bin/chromium", true),
        entry(
            "antigravity-ide",
            "Antigravity IDE",
            "/usr/bin/antigravity-ide",
            true,
        ),
        entry(
            "antigravity-ide-url-handler",
            "Antigravity IDE - URL Handler",
            "/usr/bin/antigravity-ide",
            false,
        ),
        entry("odd-casing", "bEtA   editor!", "/usr/bin/odd-casing", true),
    ]
}

#[test]
fn process_entry_beats_title_and_app_id() {
    let identity = resolve_identity(
        &catalog(),
        "chromium",
        "main.rs - Chromium Browser",
        Some("/opt/beta/beta"),
    );
    assert_eq!(identity.name, "Beta Editor");
    assert_eq!(
        identity.candidates.first().map(String::as_str),
        Some("beta")
    );
    assert_eq!(identity.desktop_entry_id.as_deref(), Some("beta"));
}

#[test]
fn title_cannot_override_a_matching_process_entry() {
    // Even an exact Chromium title must not replace the process's entry.
    let identity = resolve_identity(
        &catalog(),
        "chromium",
        "Some page - Chromium",
        Some("/usr/bin/antigravity-ide"),
    );
    assert_eq!(identity.name, "Antigravity IDE");
}

#[test]
fn url_handler_suffix_is_never_injected() {
    let identity = resolve_identity(
        &catalog(),
        "antigravity-ide",
        "notes.md - URL Handler",
        Some("/opt/antigravity-ide/antigravity-ide"),
    );
    assert_eq!(identity.name, "Antigravity IDE");
    assert!(!identity.name.contains("URL Handler"));
    assert_eq!(
        identity.desktop_entry_id.as_deref(),
        Some("antigravity-ide")
    );
}

#[test]
fn exact_installed_name_is_preserved_verbatim() {
    let identity = resolve_identity(&catalog(), "", "", Some("/usr/bin/odd-casing"));
    assert_eq!(identity.name, "bEtA   editor!");
}

#[test]
fn app_id_fallback_uses_exact_entry_name() {
    let identity = resolve_identity(&catalog(), "beta.desktop", "untitled", None);
    assert_eq!(identity.name, "Beta Editor");
}

#[test]
fn title_fallback_only_matches_the_whole_title() {
    let apps = catalog();
    assert_eq!(
        resolve_identity(&apps, "", "Beta Editor", None).name,
        "Beta Editor"
    );
    // A conventional "document - App" suffix is not used as evidence.
    assert_eq!(
        resolve_identity(&apps, "", "main.rs - Beta Editor", None).name,
        "main.rs - Beta Editor"
    );
}

#[test]
fn unknown_app_keeps_raw_id_without_capitalising() {
    let identity = resolve_identity(&catalog(), "org.example.gamma", "user@host: ~", None);
    assert_eq!(identity.name, "org.example.gamma");
}

#[test]
fn empty_id_uses_raw_title() {
    let identity = resolve_identity(&catalog(), "", "Some Raw Title", None);
    assert_eq!(identity.name, "Some Raw Title");
    assert_eq!(identity.desktop_entry_id, None);
}

#[test]
fn title_part_helper_remains_available() {
    assert_eq!(
        get_title_app_part("a — b - Beta Editor").as_deref(),
        Some("Beta Editor")
    );
    assert_eq!(get_title_app_part("no separator here"), None);
}

#[test]
fn executable_match_uses_basename() {
    assert_eq!(
        find_by_executable(&catalog(), "/usr/bin/beta", "").map(|entry| entry.name),
        Some("Beta Editor".to_string())
    );
}

#[test]
fn shared_executable_prefers_window_id_and_visible_entry() {
    let apps = catalog();
    let entry = find_by_executable(
        &apps,
        "/opt/antigravity-ide/antigravity-ide",
        "antigravity-ide",
    )
    .expect("entry");
    assert_eq!(entry.id, "antigravity-ide");
    assert_eq!(entry.name, "Antigravity IDE");
}

#[test]
fn executable_hints_skip_proc_self_exe_alias() {
    let hints = vec!["exe".to_string(), "antigravity-ide".to_string()];
    assert_eq!(
        entry_for_executable_hints(&catalog(), &hints, "antigravity-ide").map(|entry| entry.id),
        Some("antigravity-ide".to_string())
    );
}

#[test]
fn identity_cache_serves_fresh_hits_and_expires() {
    let now = Instant::now();
    let ttl = Duration::from_secs(2);
    let key = IdentityKey {
        app_id: "beta".to_string(),
        title: "doc".to_string(),
        process_id: Some(42),
    };
    let identity = resolve_identity(&catalog(), "beta", "doc", None);
    let mut cache = IdentityCache::default();
    assert_eq!(cache.get(&key, now, ttl), None);
    cache.insert(key.clone(), identity.clone(), now, ttl, 2);
    assert_eq!(cache.get(&key, now, ttl), Some(identity.clone()));
    assert_eq!(cache.get(&key, now + ttl, ttl), None);
}

#[test]
fn identity_cache_is_bounded() {
    let now = Instant::now();
    let ttl = Duration::from_secs(2);
    let mut cache = IdentityCache::default();
    for pid in 1..=4u32 {
        let key = IdentityKey {
            app_id: String::new(),
            title: String::new(),
            process_id: Some(pid),
        };
        cache.insert(
            key,
            WindowIdentity {
                name: String::new(),
                candidates: Vec::new(),
                desktop_entry_id: None,
            },
            now,
            ttl,
            2,
        );
    }
    assert_eq!(cache.entries.len(), 2);
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
