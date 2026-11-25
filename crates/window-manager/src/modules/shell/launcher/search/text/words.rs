//! Character-index and word-boundary helpers.

/// Converts a character index into a UTF-8 byte offset.
///
/// # Arguments
/// * `text` - Text to index into.
/// * `index` - Character index, or the length for the end position.
///
/// # Returns
/// The byte offset of the character.
/// Byte offset of a character index; `len()` when the index is at the end.
pub(super) fn byte_at(text: &str, index: usize) -> usize {
    text.char_indices()
        .nth(index)
        .map(|(byte, _)| byte)
        .unwrap_or(text.len())
}

/// Tests whether a character belongs to a word for caret movement.
///
/// # Arguments
/// * `char` - Character to test.
///
/// # Returns
/// `true` for alphanumeric characters and underscores.
pub(super) fn is_word(char: char) -> bool {
    char.is_alphanumeric() || char == '_'
}

/// Finds the start of the word at or before a character index.
///
/// # Arguments
/// * `text` - Text to search.
/// * `index` - Character index to start from.
///
/// # Returns
/// The character index of the word start.
/// Start of the word containing `index`, or of the previous word when
/// `index` is already at a word boundary (GTK-style word movement).
pub(super) fn word_start(text: &str, index: usize) -> usize {
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

/// Finds the end of the word at or after a character index.
///
/// # Arguments
/// * `text` - Text to search.
/// * `index` - Character index to start from.
///
/// # Returns
/// The character index of the word end.
/// End of the word at `index`, skipping any intervening separators.
pub(super) fn word_end(text: &str, index: usize) -> usize {
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
