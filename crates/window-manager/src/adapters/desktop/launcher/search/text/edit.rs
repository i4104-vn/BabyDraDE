//! Query text editing operations.

use super::words::{byte_at, word_end, word_start};
use super::SearchText;

impl SearchText {
    /// Inserts text at the caret, replacing the selection if there is one.
    ///
    /// # Arguments
    /// * `text` - Text to insert.
    ///
    /// # Returns
    /// Nothing; the state is updated in place.
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
    ///
    /// # Returns
    /// Nothing; the state is updated in place.
    /// Deletes the selection, or the character before the caret.
    pub(crate) fn backspace(&mut self) {
        if let Some((start, end)) = self.selection() {
            self.remove_range(start, end);
        } else if self.cursor > 0 {
            self.remove_range(self.cursor - 1, self.cursor);
        }
    }

    /// Deletes the selection, or the character after the caret.
    ///
    /// # Returns
    /// Nothing; the state is updated in place.
    /// Deletes the selection, or the character after the caret.
    pub(crate) fn delete(&mut self) {
        if let Some((start, end)) = self.selection() {
            self.remove_range(start, end);
        } else if self.cursor < self.text.chars().count() {
            self.remove_range(self.cursor, self.cursor + 1);
        }
    }

    /// Deletes from the start of the word before the caret up to the caret.
    ///
    /// # Returns
    /// Nothing; the state is updated in place.
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

    /// Moves the caret by a character or word step.
    ///
    /// # Arguments
    /// * `direction` - Negative moves left, positive moves right.
    /// * `word` - Whether to step by words.
    /// * `extend` - Whether to extend the selection.
    ///
    /// # Returns
    /// Nothing; the state is updated in place.
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

    /// Moves the caret to the start of the query.
    ///
    /// # Arguments
    /// * `extend` - Whether to extend the current selection.
    ///
    /// # Returns
    /// Nothing; the state is updated in place.
    pub(crate) fn home(&mut self, extend: bool) {
        self.set_caret(0, extend);
    }

    /// Moves the caret to the end of the query.
    ///
    /// # Arguments
    /// * `extend` - Whether to extend the current selection.
    ///
    /// # Returns
    /// Nothing; the state is updated in place.
    pub(crate) fn end(&mut self, extend: bool) {
        let end = self.text.chars().count();
        self.set_caret(end, extend);
    }

    /// Selects the entire query.
    ///
    /// # Returns
    /// Nothing; the state is updated in place.
    pub(crate) fn select_all(&mut self) {
        self.anchor = Some(0);
        self.cursor = self.text.chars().count();
    }

    /// Places the caret at a character index (used by pointer hit-testing).
    ///
    /// # Arguments
    /// * `index` - Character index for the caret.
    ///
    /// # Returns
    /// Nothing; the state is updated in place.
    /// Places the caret at a character index (used by pointer hit-testing).
    pub(crate) fn set_caret_index(&mut self, index: usize) {
        self.anchor = None;
        self.cursor = index.min(self.text.chars().count());
    }

    /// Moves the caret, optionally keeping the selection anchor.
    ///
    /// # Arguments
    /// * `target` - Character index for the caret.
    /// * `extend` - Whether to extend the selection.
    ///
    /// # Returns
    /// Nothing; the state is updated in place.
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

    /// Deletes a character range and places the caret at its start.
    ///
    /// # Arguments
    /// * `start` - Start character index.
    /// * `end` - End character index.
    ///
    /// # Returns
    /// Nothing; the state is updated in place.
    fn remove_range(&mut self, start: usize, end: usize) {
        let from = byte_at(&self.text, start);
        let to = byte_at(&self.text, end);
        self.text.replace_range(from..to, "");
        self.cursor = start;
        self.anchor = None;
    }
}
