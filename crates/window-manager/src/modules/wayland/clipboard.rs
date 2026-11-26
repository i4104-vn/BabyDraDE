//! Compositor-side clipboard persistence.
//!
//! Wayland selections are owned by the client that copied the data, so the
//! clipboard is lost as soon as that client exits. We cache the selection in
//! `services` and then take ownership of it ourselves, which keeps copy/paste
//! working exactly like on other desktops.

pub(crate) use crate::modules::wayland::models::ClipboardState;

pub(crate) mod policy {
    //! Use case: choose which selection MIME type to capture.
    //!
    //! Pure policy — no pipes, threads or Wayland selection state.

    use common::constants::{MIME_TEXT_PREFIX, PREFERRED_TEXT_MIMES};

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
            .find(|mime| mime.starts_with(MIME_TEXT_PREFIX))
            .cloned()
            .or_else(|| mimes.first().cloned())
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn prefers_known_text_types() {
            let mimes = vec!["STRING".into(), "text/plain".into()];
            assert_eq!(preferred_mime(&mimes).as_deref(), Some("text/plain"));
        }

        #[test]
        fn falls_back_to_any_type() {
            let mimes = vec!["image/png".into()];
            assert_eq!(preferred_mime(&mimes).as_deref(), Some("image/png"));
            assert_eq!(preferred_mime(&[]), None);
        }
    }
}
