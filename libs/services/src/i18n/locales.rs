//! Bundled translation catalogs for English and Vietnamese.

use std::collections::HashMap;

/// Merges a JSON translation object into `target`, ignoring malformed files.
fn parse_locale_map(json_str: &str, target: &mut HashMap<String, String>) {
    if let Ok(map) = serde_json::from_str::<HashMap<String, String>>(json_str) {
        target.extend(map);
    }
}

/// Builds the English translation map from all bundled catalogs.
pub(super) fn load_en_map() -> HashMap<String, String> {
    let mut map = HashMap::new();
    parse_locale_map(include_str!("locales/common/en.json"), &mut map);
    parse_locale_map(include_str!("locales/launcher/en.json"), &mut map);
    parse_locale_map(include_str!("locales/settings/en.json"), &mut map);
    parse_locale_map(include_str!("locales/explore/en.json"), &mut map);
    parse_locale_map(include_str!("locales/greeter/en.json"), &mut map);
    parse_locale_map(include_str!("locales/desktop/en.json"), &mut map);
    parse_locale_map(include_str!("locales/recorder/en.json"), &mut map);
    parse_locale_map(include_str!("locales/file_dialog/en.json"), &mut map);
    map
}

/// Builds the Vietnamese translation map from all bundled catalogs.
pub(super) fn load_vi_map() -> HashMap<String, String> {
    let mut map = HashMap::new();
    parse_locale_map(include_str!("locales/common/vi.json"), &mut map);
    parse_locale_map(include_str!("locales/launcher/vi.json"), &mut map);
    parse_locale_map(include_str!("locales/settings/vi.json"), &mut map);
    parse_locale_map(include_str!("locales/explore/vi.json"), &mut map);
    parse_locale_map(include_str!("locales/greeter/vi.json"), &mut map);
    parse_locale_map(include_str!("locales/desktop/vi.json"), &mut map);
    parse_locale_map(include_str!("locales/recorder/vi.json"), &mut map);
    parse_locale_map(include_str!("locales/file_dialog/vi.json"), &mut map);
    map
}
