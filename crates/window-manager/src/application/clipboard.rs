//! Use case: choose which selection MIME type to capture.
//!
//! Pure policy — no pipes, threads or Wayland selection state.

/// Text MIME types we prefer when capturing a selection, in priority order.
const PREFERRED_TEXT_MIMES: &[&str] = &[
    "text/plain;charset=utf-8",
    "UTF8_STRING",
    "text/plain",
    "text/plain;charset=utf16le",
    "STRING",
];

/// Selects the MIME type to capture from a client's offered types.
///
/// # Arguments
///
/// * `mimes` - MIME types offered by the selection source.
///
/// # Returns
///
/// The preferred text MIME type, any text type, or the first offered type;
/// `None` when no type was offered.
pub(crate) fn preferred_mime(mimes: &[String]) -> Option<String> {
    for wanted in PREFERRED_TEXT_MIMES {
        if let Some(mime) = mimes.iter().find(|mime| mime.as_str() == *wanted) {
            return Some(mime.clone());
        }
    }
    mimes
        .iter()
        .find(|mime| mime.starts_with("text/"))
        .cloned()
        .or_else(|| mimes.first().cloned())
}

#[cfg(test)]
#[path = "../tests/application/clipboard.rs"]
mod tests;
