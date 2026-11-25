//! Plain-text editing model for the launcher query.
//!
//! The compositor receives key events from Smithay, not from a GTK widget, so
//! a GTK `TextBuffer` cannot be the source of truth here: creating one before
//! `gtk::init` panics, and a headless session may never initialize GTK at all.
//! This model keeps the same editing primitives (caret, selection, word
//! movement) without any toolkit dependency. Offsets are character indices,
//! which is what the renderer records when it measures the query with Cairo.

/// Query text plus its caret and optional selection, in character indices.
#[derive(Clone, Debug, Default)]
pub(crate) struct SearchText {
    text: String,
    /// Insertion point, in `0..=char_count`.
    cursor: usize,
    /// Selection bound; `Some` while Shift (or a pointer drag) holds it.
    anchor: Option<usize>,
}

impl SearchText {
    /// Creates an empty query model with no caret selection.
    ///
    /// # Returns
    /// The new search text model.
    pub(crate) fn new() -> Self {
        Self::default()
    }
    /// Reads the current query.
    ///
    /// # Returns
    /// The query string.
    pub(crate) fn text(&self) -> &str {
        &self.text
    }
    /// Reports whether the query is empty.
    ///
    /// # Returns
    /// `true` when no query text is present.
    pub(crate) fn is_empty(&self) -> bool {
        self.text.is_empty()
    }

    /// Current caret position as a character index.
    pub(crate) fn caret(&self) -> usize {
        self.cursor
    }
    /// Selected character range, `(start, end)`, if there is a selection.
    pub(crate) fn selection(&self) -> Option<(usize, usize)> {
        self.anchor
            .filter(|&anchor| anchor != self.cursor)
            .map(|anchor| (anchor.min(self.cursor), anchor.max(self.cursor)))
    }
    /// Clears the query and resets the caret and selection.
    ///
    /// # Returns
    /// Nothing; the state is updated in place.
    pub(crate) fn clear(&mut self) {
        self.text.clear();
        self.cursor = 0;
        self.anchor = None;
    }
}


mod edit;

mod words;

// Inlined from text/tests.rs.
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inserts_at_caret_and_handles_utf8() {
        let mut text = SearchText::new();
        text.insert("héllo");
        text.set_caret_index(1);
        text.insert("ä");
        assert_eq!(text.text(), "häéllo");
        assert_eq!(text.caret(), 2);
    }

    #[test]
    fn selection_is_replaced_by_insertion() {
        let mut text = SearchText::new();
        text.insert("firefox");
        text.home(false);
        for _ in 0..3 {
            text.move_caret(1, false, true);
        }
        assert_eq!(text.selection(), Some((0, 3)));
        text.insert("chrome");
        assert_eq!(text.text(), "chromeefox");
        assert_eq!(text.caret(), 6);
        assert_eq!(text.selection(), None);
    }

    #[test]
    fn backspace_delete_and_word_delete() {
        let mut text = SearchText::new();
        text.insert("one two");
        text.backspace();
        assert_eq!(text.text(), "one tw");
        text.delete_word();
        assert_eq!(text.text(), "one ");
        // Caret is at the end; Delete has nothing after it.
        text.delete();
        assert_eq!(text.text(), "one ");
    }

    #[test]
    fn word_movement_and_endpoints() {
        let mut text = SearchText::new();
        text.insert("one two");
        text.end(false);
        text.move_caret(-1, true, false);
        assert_eq!(text.caret(), 4);
        text.home(true);
        assert_eq!(text.selection(), Some((0, 4)));
        text.select_all();
        assert_eq!(text.selection(), Some((0, 7)));
    }
}
