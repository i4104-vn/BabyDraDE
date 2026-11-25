//! Input-method environment defaults for fcitx5.
//!
//! Exported before toolkit initialization so GTK, Qt and SDL applications
//! started by the compositor inherit the same input method. Values already set
//! by the user are never overwritten.

/// `(variable, fcitx5 value)` pairs applied when the variable is unset.
const DEFAULTS: &[(&str, &str)] = &[
    ("GTK_IM_MODULE", "fcitx"),
    ("QT_IM_MODULE", "fcitx"),
    ("SDL_IM_MODULE", "fcitx"),
    ("INPUT_METHOD", "fcitx"),
    ("XMODIFIERS", "@im=fcitx"),
];

/// Exports the fcitx5 input-method defaults without overriding user settings.
pub(crate) fn export_defaults() {
    for &(key, value) in DEFAULTS {
        if std::env::var_os(key).is_some() {
            continue;
        }
        // SAFETY: called from the composition root before GTK init or threads.
        std::env::set_var(key, value);
    }
}
