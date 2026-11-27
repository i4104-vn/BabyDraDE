use std::path::Path;

use super::desktop_entry::parse_desktop_entry;

#[test]
fn substitutes_desktop_entry_fields() {
    assert_eq!(
        parse_desktop_entry(
            "Exec=note --wait %f",
            Path::new("/tmp/note.txt"),
            "file:///tmp/note.txt"
        ),
        Some((
            "note".to_string(),
            vec!["--wait".into(), "/tmp/note.txt".into()],
            false,
        ))
    );
}

#[test]
fn appends_path_when_no_field_present() {
    assert_eq!(
        parse_desktop_entry(
            "Exec=babydra-preview",
            Path::new("/tmp/a.png"),
            "file:///tmp/a.png"
        ),
        Some((
            "babydra-preview".to_string(),
            vec!["/tmp/a.png".into()],
            false,
        ))
    );
}

#[test]
fn keeps_quoted_arguments_with_spaces() {
    assert_eq!(
        parse_desktop_entry(
            "Exec=note --title \"My Notes\" %f",
            Path::new("/tmp/note.txt"),
            "file:///tmp/note.txt"
        ),
        Some((
            "note".to_string(),
            vec!["--title".into(), "My Notes".into(), "/tmp/note.txt".into()],
            false,
        ))
    );
}

#[test]
fn substitutes_field_inside_a_quoted_argument() {
    assert_eq!(
        parse_desktop_entry(
            "Exec=note \"%f\"",
            Path::new("/tmp/note.txt"),
            "file:///tmp/note.txt"
        ),
        Some(("note".to_string(), vec!["/tmp/note.txt".into()], false,))
    );
}
