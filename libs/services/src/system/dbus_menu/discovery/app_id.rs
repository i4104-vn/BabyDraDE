use gio::glib::Variant;

/// Timeout for a `GetConnectionUnixProcessID` call, in milliseconds.
const PROCESS_ID_TIMEOUT_MS: i32 = 100;

/// Flattens an app id or bus name to printable lowercase characters.
fn normalize_app_id(value: &str) -> String {
    let value = value
        .trim()
        .strip_suffix(".desktop")
        .unwrap_or(value.trim());
    value
        .to_ascii_lowercase()
        .chars()
        .filter(char::is_ascii_alphanumeric)
        .collect()
}

/// Dot/dash/underscore separated components of an app id or bus name.
fn app_tokens(value: &str) -> Vec<String> {
    let value = value.trim();
    let value = value.strip_suffix(".desktop").unwrap_or(value);
    value
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|token| token.len() >= 2)
        .map(str::to_ascii_lowercase)
        .collect()
}

/// Matches a bus name against an app id without hardcoding any application.
///
/// Handles desktop ids (`org.gnome.Nautilus.desktop`), short executable names
/// (`code` vs `com.visualstudio.code`), and per-instance suffixes
/// (`org.kde.kate_4578`) used by KDE and Electron helpers.
pub(in crate::system::dbus_menu) fn service_matches_app_id(service: &str, app_id: &str) -> bool {
    let service_normalized = normalize_app_id(service);
    let app_normalized = normalize_app_id(app_id);
    if service_normalized.is_empty() || app_normalized.is_empty() {
        return false;
    }
    if service_normalized == app_normalized {
        return true;
    }
    if app_normalized.len() >= 3
        && (service_normalized.starts_with(&app_normalized)
            || service_normalized.ends_with(&app_normalized))
    {
        return true;
    }

    let app_id_tokens = app_tokens(app_id);
    let service_tokens = app_tokens(service);
    // The last token is the application's own name; namespace tokens such as
    // `org` or `gnome` are shared by unrelated apps and must not count.
    let Some(distinctive) = app_id_tokens.iter().rev().find(|token| token.len() >= 3) else {
        return false;
    };
    service_tokens.iter().any(|candidate| {
        candidate.starts_with(distinctive)
            || (distinctive.starts_with(candidate) && candidate.len() >= 4)
    })
}

/// Returns the Unix process id registered for a bus name, when available.
pub(in crate::system::dbus_menu) fn connection_process_id(
    conn: &gio::DBusConnection,
    name: &str,
) -> Option<u32> {
    let params = Variant::from((name,));
    conn.call_sync(
        Some("org.freedesktop.DBus"),
        "/org/freedesktop/DBus",
        "org.freedesktop.DBus",
        "GetConnectionUnixProcessID",
        Some(&params),
        None,
        gio::DBusCallFlags::NONE,
        PROCESS_ID_TIMEOUT_MS,
        gio::Cancellable::NONE,
    )
    .ok()?
    .child_value(0)
    .get::<u32>()
}

#[cfg(test)]
mod tests {
    use super::{app_tokens, normalize_app_id, service_matches_app_id};

    #[test]
    fn app_id_normalization_keeps_useful_tokens() {
        assert_eq!(
            normalize_app_id("org.gnome.Nautilus.desktop"),
            "orggnomenautilus"
        );
        assert_eq!(
            app_tokens("firefox_firefox.desktop"),
            vec!["firefox", "firefox"]
        );
    }

    #[test]
    fn service_matching_accepts_desktop_and_helper_name_variants() {
        assert!(service_matches_app_id(
            "org.gnome.Nautilus",
            "org.gnome.Nautilus.desktop"
        ));
        assert!(service_matches_app_id("com.visualstudio.code", "code"));
        assert!(service_matches_app_id("org.kde.kate_4578", "org.kde.kate"));
        assert!(service_matches_app_id("firefox", "firefox_firefox.desktop"));
        assert!(!service_matches_app_id(
            "org.gnome.Terminal",
            "org.gnome.Nautilus"
        ));
    }
}
