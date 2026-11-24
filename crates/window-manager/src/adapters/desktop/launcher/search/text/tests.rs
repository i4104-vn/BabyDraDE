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
