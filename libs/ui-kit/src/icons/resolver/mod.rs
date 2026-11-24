//! Pure string resolver for BabyDra icon names.

use std::collections::HashMap;
use std::sync::OnceLock;

mod pairs;

fn aliases() -> &'static HashMap<String, String> {
    static ALIASES: OnceLock<HashMap<String, String>> = OnceLock::new();
    ALIASES.get_or_init(|| {
        serde_json::from_str(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/icon_aliases.json"
        )))
        .unwrap_or_default()
    })
}

/// Resolves an icon name to its dark/light SVG asset pair.
pub fn resolve_icon_pair(name: &str) -> Option<(&'static str, &'static str)> {
    let canonical = aliases().get(name).map(String::as_str).unwrap_or(name);
    pairs::resolve(canonical)
}

/// Resolves an icon name to a single SVG asset for the requested theme.
pub fn resolve_icon(name: &str, dark_mode: bool) -> Option<&'static str> {
    let (dark, light) = resolve_icon_pair(name)?;
    Some(if dark_mode { dark } else { light })
}
