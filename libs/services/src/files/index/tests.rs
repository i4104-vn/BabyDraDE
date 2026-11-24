//! Behavior tests for the bounded filesystem snapshot.

use std::fs;

use super::handler::scan_root;
use super::models::{FileEntry, SKIP_DIRS};

fn names(entries: &[FileEntry]) -> Vec<&str> {
    entries.iter().map(|entry| entry.name.as_str()).collect()
}

#[test]
fn should_collect_sorted_entries_using_hidden_and_depth_rules() {
    let root = tempfile::tempdir().expect("temporary home directory");
    fs::write(root.path().join("zeta.txt"), "z").expect("write zeta");
    fs::write(root.path().join("Alpha.txt"), "a").expect("write alpha");
    fs::write(root.path().join(".hidden_root"), "h").expect("write hidden root file");
    fs::write(root.path().join("target"), "t").expect("write skip-dir named file");
    fs::create_dir(root.path().join(".cache")).expect("create skipped dir");
    fs::write(root.path().join(".cache/cached.txt"), "c").expect("write cached file");

    fs::create_dir(root.path().join("sub")).expect("create sub dir");
    fs::write(root.path().join("sub/inside.txt"), "i").expect("write inside file");
    fs::write(root.path().join("sub/.secret"), "s").expect("write nested hidden file");
    fs::create_dir_all(root.path().join("sub/nested/too_deep/very_deep"))
        .expect("create nested dirs");
    fs::write(root.path().join("sub/nested/deep.txt"), "d").expect("write deep file");
    fs::write(root.path().join("sub/nested/too_deep/deeper.txt"), "dd")
        .expect("write deeper file");
    fs::write(root.path().join("sub/nested/too_deep/very_deep/too_far.txt"), "f")
        .expect("write out-of-depth file");

    let entries = scan_root(root.path(), 400, 3, SKIP_DIRS);

    assert_eq!(
        names(&entries),
        [
            ".hidden_root",
            "Alpha.txt",
            "deep.txt",
            "deeper.txt",
            "inside.txt",
            "nested",
            "sub",
            "target",
            "too_deep",
            "very_deep",
            "zeta.txt",
        ]
    );
    assert!(entries.iter().any(|entry| entry.name == "sub" && entry.is_dir));
    assert!(
        entries
            .iter()
            .any(|entry| entry.name == "Alpha.txt" && entry.lower == "alpha.txt")
    );
}

#[test]
fn should_stop_collecting_at_the_entry_budget() {
    let root = tempfile::tempdir().expect("temporary home directory");
    for name in ["a.txt", "b.txt", "c.txt"] {
        fs::write(root.path().join(name), name).expect("write file");
    }

    let entries = scan_root(root.path(), 2, 3, SKIP_DIRS);

    assert_eq!(names(&entries).len(), 2);
    assert!(names(&entries).iter().all(|name| ["a.txt", "b.txt", "c.txt"].contains(name)));

    let empty = scan_root(root.path(), 0, 3, SKIP_DIRS);
    assert!(empty.is_empty());
}
