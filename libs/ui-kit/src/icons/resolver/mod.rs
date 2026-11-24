//! Pure string resolver for BabyDra icon names.
use std::collections::HashMap;
use std::sync::OnceLock;

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

pub fn resolve_icon_pair(name: &str) -> Option<(&'static str, &'static str)> {
    let canonical = aliases().get(name).map(String::as_str).unwrap_or(name);

    pair_a_c::resolve(canonical)
        .or_else(|| pair_d_f::resolve(canonical))
        .or_else(|| pair_g_m::resolve(canonical))
        .or_else(|| pair_n_s::resolve(canonical))
        .or_else(|| pair_t_z::resolve(canonical))
}

pub fn resolve_icon(name: &str, dark_mode: bool) -> Option<&'static str> {
    let (dark, light) = resolve_icon_pair(name)?;
    Some(if dark_mode { dark } else { light })
}
mod pair_a_c;
mod pair_d_f;
mod pair_g_m;
mod pair_n_s;
mod pair_t_z;
