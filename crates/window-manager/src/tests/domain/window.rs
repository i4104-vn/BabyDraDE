use super::*;

#[test]
fn caches_and_forgets_metadata() {
    store(
        "surface-1",
        WindowMetadata {
            app_id: "app".into(),
            title: "Title".into(),
            app_name: "App".into(),
        },
    );
    assert_eq!(cached("surface-1").map(|m| m.app_id), Some("app".into()));
    forget("surface-1");
    assert!(cached("surface-1").is_none());
}
