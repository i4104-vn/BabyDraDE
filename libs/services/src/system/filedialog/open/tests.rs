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
