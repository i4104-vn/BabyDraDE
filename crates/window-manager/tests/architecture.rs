use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

fn src_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src")
}

fn collect_source_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.components().any(|component| component.as_os_str() == "tests") {
            continue;
        }
        if path.is_dir() {
            collect_source_files(&path, out);
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            out.push(path);
        }
    }
}

fn source_files() -> Vec<PathBuf> {
    let mut files = Vec::new();
    collect_source_files(&src_root(), &mut files);
    files.sort();
    files
}

fn relative_to_src(path: &Path) -> PathBuf {
    path.strip_prefix(src_root())
        .expect("source file under src")
        .to_path_buf()
}

fn top_level_area(path: &Path) -> Option<String> {
    relative_to_src(path)
        .components()
        .next()
        .and_then(|component| component.as_os_str().to_str())
        .map(str::to_owned)
}

const ADAPTER_AREAS: [&str; 4] = ["compositor", "input", "rendering", "shell"];

const BASELINE_EDGES: [(&str, &str); 8] = [
    ("compositor", "input"),
    ("compositor", "rendering"),
    ("compositor", "shell"),
    ("input", "compositor"),
    ("input", "shell"),
    ("rendering", "compositor"),
    ("shell", "compositor"),
    ("shell", "rendering"),
];

#[test]
fn domain_and_application_do_not_import_outer_layers() {
    let forbidden = ["crate::adapters", "crate::infrastructure", "crate::composition"];
    for path in source_files() {
        if !matches!(top_level_area(&path).as_deref(), Some("domain" | "application")) {
            continue;
        }
        let text = fs::read_to_string(&path).expect("read source file");
        for pattern in forbidden {
            assert!(
                !text.contains(pattern),
                "{} must not import {}",
                path.display(),
                pattern
            );
        }
    }
}

#[test]
fn adapter_import_edges_are_unchanged() {
    // Baseline must shrink as the refactor removes cycles; never add new edges.
    let mut edges = BTreeSet::new();
    for path in source_files() {
        let relative = relative_to_src(&path);
        let Ok(adapters) = relative.strip_prefix("adapters") else {
            continue;
        };
        let Some(from_name) = adapters
            .components()
            .next()
            .and_then(|component| component.as_os_str().to_str())
        else {
            continue;
        };
        let Some(from) = ADAPTER_AREAS.iter().copied().find(|area| *area == from_name) else {
            continue;
        };

        let text = fs::read_to_string(&path).expect("read source file");
        for &to in &ADAPTER_AREAS {
            if to == from {
                continue;
            }
            let needle = format!("crate::adapters::{to}");
            if text.contains(&needle) {
                edges.insert((from, to));
            }
        }
    }

    let baseline: BTreeSet<_> = BASELINE_EDGES.into_iter().collect();
    assert_eq!(edges, baseline, "adapter import graph changed");
}
