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

#[cfg(test)]
mod tests;
