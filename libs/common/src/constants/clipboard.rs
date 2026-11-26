//! Clipboard MIME vocabulary shared by the compositor and clipboard service.
//!
//! GTK, Qt and XWayland use different names for the same textual selection, so
//! both processes must agree on which names count as text and which aliases to
//! advertise.

/// X11 UTF-8 text selection alias.
pub const MIME_UTF8_STRING: &str = "UTF8_STRING";
/// Legacy X11 text selection type.
pub const MIME_STRING: &str = "STRING";
/// Plain-text MIME type with an explicit UTF-8 charset.
pub const MIME_TEXT_PLAIN_UTF8: &str = "text/plain;charset=utf-8";
/// Plain-text MIME type.
pub const MIME_TEXT_PLAIN: &str = "text/plain";
/// UTF-16LE plain-text MIME type.
pub const MIME_TEXT_PLAIN_UTF16LE: &str = "text/plain;charset=utf16le";
/// Prefix shared by every `text/*` MIME type.
pub const MIME_TEXT_PREFIX: &str = "text/";

/// Text MIME types in capture preference order.
pub const PREFERRED_TEXT_MIMES: &[&str] = &[
    MIME_TEXT_PLAIN_UTF8,
    MIME_UTF8_STRING,
    MIME_TEXT_PLAIN,
    MIME_TEXT_PLAIN_UTF16LE,
    MIME_STRING,
];

/// Aliases advertised when the cached selection is textual.
pub const TEXT_MIME_ALIASES: &[&str] = &[
    MIME_UTF8_STRING,
    MIME_STRING,
    MIME_TEXT_PLAIN,
    MIME_TEXT_PLAIN_UTF8,
];

/// Returns `true` when a MIME name denotes a textual selection.
///
/// # Arguments
/// * `mime` - MIME name offered by a selection source or requested by a client.
///
/// # Returns
/// Whether the name is a `text/*` type or one of the legacy X11 text aliases.
pub fn is_text_mime(mime: &str) -> bool {
    mime.starts_with(MIME_TEXT_PREFIX) || matches!(mime, MIME_UTF8_STRING | MIME_STRING)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognizes_text_names_and_aliases() {
        for mime in ["text/plain", "text/html", MIME_UTF8_STRING, MIME_STRING] {
            assert!(is_text_mime(mime), "{mime} should be text");
        }
        for mime in ["image/png", "application/json", ""] {
            assert!(!is_text_mime(mime), "{mime} should not be text");
        }
    }
}
