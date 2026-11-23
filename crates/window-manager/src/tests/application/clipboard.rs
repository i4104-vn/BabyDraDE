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
