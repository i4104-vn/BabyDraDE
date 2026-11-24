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
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn text(&self) -> &str {
        &self.text
    }

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

    pub(crate) fn clear(&mut self) {
        self.text.clear();
        self.cursor = 0;
        self.anchor = None;
    }

    /// Inserts text at the caret, replacing the selection if there is one.
    pub(crate) fn insert(&mut self, text: &str) {
        if text.is_empty() {
            return;
        }
        if let Some((start, end)) = self.selection() {
            self.remove_range(start, end);
        }
        let at = byte_at(&self.text, self.cursor);
        self.text.insert_str(at, text);
        self.cursor += text.chars().count();
        self.anchor = None;
    }

    /// Deletes the selection, or the character before the caret.
    pub(crate) fn backspace(&mut self) {
        if let Some((start, end)) = self.selection() {
            self.remove_range(start, end);
        } else if self.cursor > 0 {
            self.remove_range(self.cursor - 1, self.cursor);
        }
    }

    /// Deletes the selection, or the character after the caret.
    pub(crate) fn delete(&mut self) {
        if let Some((start, end)) = self.selection() {
            self.remove_range(start, end);
        } else if self.cursor < self.text.chars().count() {
            self.remove_range(self.cursor, self.cursor + 1);
        }
    }

    /// Deletes from the start of the word before the caret up to the caret.
    pub(crate) fn delete_word(&mut self) {
        if let Some((start, end)) = self.selection() {
            self.remove_range(start, end);
            return;
        }
        let start = word_start(&self.text, self.cursor);
        if start < self.cursor {
            self.remove_range(start, self.cursor);
        }
    }

    /// Moves the caret. `word` switches to word steps, `extend` keeps the
    /// selection bound so Shift+Arrow extends instead of moving.
    pub(crate) fn move_caret(&mut self, direction: i32, word: bool, extend: bool) {
        let current = self.cursor;
        let target = if direction < 0 {
            if word {
                word_start(&self.text, current)
            } else {
                current.saturating_sub(1)
            }
        } else if word {
            word_end(&self.text, current)
        } else {
            (current + 1).min(self.text.chars().count())
        };
        self.set_caret(target, extend);
    }

    pub(crate) fn home(&mut self, extend: bool) {
        self.set_caret(0, extend);
    }

    pub(crate) fn end(&mut self, extend: bool) {
        let end = self.text.chars().count();
        self.set_caret(end, extend);
    }

    pub(crate) fn select_all(&mut self) {
        self.anchor = Some(0);
        self.cursor = self.text.chars().count();
    }

    /// Places the caret at a character index (used by pointer hit-testing).
    pub(crate) fn set_caret_index(&mut self, index: usize) {
        self.anchor = None;
        self.cursor = index.min(self.text.chars().count());
    }

    fn set_caret(&mut self, target: usize, extend: bool) {
        if extend {
            if self.anchor.is_none() {
                self.anchor = Some(self.cursor);
            }
        } else {
            self.anchor = None;
        }
        self.cursor = target;
        if self.anchor == Some(target) {
            self.anchor = None;
        }
    }

    fn remove_range(&mut self, start: usize, end: usize) {
        let from = byte_at(&self.text, start);
        let to = byte_at(&self.text, end);
        self.text.replace_range(from..to, "");
        self.cursor = start;
        self.anchor = None;
    }
}

/// Byte offset of a character index; `len()` when the index is at the end.
fn byte_at(text: &str, index: usize) -> usize {
    text.char_indices()
        .nth(index)
        .map(|(byte, _)| byte)
        .unwrap_or(text.len())
}

fn is_word(char: char) -> bool {
    char.is_alphanumeric() || char == '_'
}

/// Start of the word containing `index`, or of the previous word when
/// `index` is already at a word boundary (GTK-style word movement).
fn word_start(text: &str, index: usize) -> usize {
    let chars: Vec<char> = text.chars().collect();
    let mut index = index.min(chars.len());
    while index > 0 && !is_word(chars[index - 1]) {
        index -= 1;
    }
    while index > 0 && is_word(chars[index - 1]) {
        index -= 1;
    }
    index
}

/// End of the word at `index`, skipping any intervening separators.
fn word_end(text: &str, index: usize) -> usize {
    let chars: Vec<char> = text.chars().collect();
    let mut index = index.min(chars.len());
    while index < chars.len() && !is_word(chars[index]) {
        index += 1;
    }
    while index < chars.len() && is_word(chars[index]) {
        index += 1;
    }
    index
}

#[cfg(test)]
mod tests;
