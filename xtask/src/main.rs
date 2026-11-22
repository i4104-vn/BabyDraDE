//! Architecture fitness functions for the BabyDraDE workspace.
//!
//! `cargo xtask check` verifies the dependency matrix, binary-crate visibility,
//! hot-path purity and unsafe-block documentation. `cargo xtask baseline`
//! records the current debt in `xtask/baseline.toml` so CI fails on new
//! violations only.

use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

#[derive(Debug, Deserialize)]
struct Architecture {
    matrix: std::collections::BTreeMap<String, Vec<String>>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
struct Baseline {
    #[serde(default)]
    layering: BTreeSet<String>,
    #[serde(default)]
    visibility: BTreeSet<String>,
    #[serde(default)]
    hot_path: BTreeSet<String>,
    #[serde(default)]
    unsafe_blocks: BTreeSet<String>,
}

fn main() -> ExitCode {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtask lives inside the workspace");
    let command = std::env::args().nth(1).unwrap_or_else(|| "check".into());

    match command.as_str() {
        "baseline" => match write_baseline(root) {
            Ok(()) => {
                println!("baseline written to xtask/baseline.toml");
                ExitCode::SUCCESS
            }
            Err(err) => {
                eprintln!("baseline failed: {err}");
                ExitCode::FAILURE
            }
        },
        "check" => match check(root) {
            Ok(()) => ExitCode::SUCCESS,
            Err(err) => {
                eprintln!("architecture check failed: {err}");
                ExitCode::FAILURE
            }
        },
        other => {
            eprintln!("unknown command: {other} (expected 'check' or 'baseline')");
            ExitCode::FAILURE
        }
    }
}

fn architecture(root: &Path) -> Result<Architecture, String> {
    let path = root.join("xtask").join("architecture.toml");
    let content = fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    toml::from_str(&content).map_err(|e| format!("{}: {e}", path.display()))
}

fn baseline(root: &Path) -> Result<Baseline, String> {
    let path = root.join("xtask").join("baseline.toml");
    if !path.exists() {
        return Ok(Baseline::default());
    }
    let content = fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    toml::from_str(&content).map_err(|e| format!("{}: {e}", path.display()))
}

fn rust_files(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let Ok(entries) = fs::read_dir(dir) else {
        return out;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            out.extend(rust_files(&path));
        } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
            out.push(path);
        }
    }
    out
}

fn source_dir(root: &Path) -> PathBuf {
    root.join("crates").join("window-manager").join("src")
}

fn relative(path: &Path, root: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

fn layer_for(path: &Path, src: &Path) -> Option<String> {
    let rel = path.strip_prefix(src).ok()?;
    let first = rel.components().next()?.as_os_str().to_str()?;
    match first {
        "main.rs" => Some("app".to_string()),
        "extensions" => Some("shell".to_string()),
        "backends" | "compositor" | "input" | "rendering" | "keymap" | "model" | "platform"
        | "shell" | "app" => Some(first.to_string()),
        _ => None,
    }
}

fn strip_comments(content: &str) -> String {
    content
        .lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n")
}

fn crate_targets(content: &str) -> BTreeSet<String> {
    let mut targets = BTreeSet::new();
    let bytes = content.as_bytes();
    let mut index = 0;
    while let Some(offset) = content[index..].find("crate::") {
        let start = index + offset + "crate::".len();
        let mut end = start;
        while end < bytes.len() && (bytes[end].is_ascii_lowercase() || bytes[end] == b'_') {
            end += 1;
        }
        if end > start {
            targets.insert(content[start..end].to_string());
        }
        index = end.max(index + 1);
    }
    targets
}

fn layering_violations(root: &Path, arch: &Architecture, base: &Baseline) -> Vec<String> {
    let src = source_dir(root);
    let mut violations = Vec::new();
    for file in rust_files(&src) {
        let Some(from) = layer_for(&file, &src) else {
            continue;
        };
        let content = match fs::read_to_string(&file) {
            Ok(content) => content,
            Err(_) => continue,
        };
        let allowed = arch.matrix.get(&from).cloned().unwrap_or_default();
        let path = relative(&file, root);
        let code = strip_comments(&content);
        for to in crate_targets(&code) {
            if to == from
                || (from == "shell" && to == "extensions")
                || allowed.iter().any(|item| item == &to)
            {
                continue;
            }
            let key = format!("{path}|{from}|{to}");
            if !base.layering.contains(&key) {
                violations.push(format!("layering: {path} ({from} -> {to})"));
            }
        }
    }
    violations
}

fn visibility_violations(root: &Path, base: &Baseline) -> Vec<String> {
    let src = source_dir(root);
    let mut violations = Vec::new();
    for file in rust_files(&src) {
        let Ok(content) = fs::read_to_string(&file) else {
            continue;
        };
        let has_pub = content.lines().any(|line| {
            let trimmed = line.trim_start();
            trimmed.starts_with("pub ") && !trimmed.starts_with("pub(crate)")
        });
        if !has_pub {
            continue;
        }
        let path = relative(&file, root);
        if !base.visibility.contains(&path) {
            violations.push(format!(
                "visibility: {path} exposes pub items in a binary crate"
            ));
        }
    }
    violations
}

const HOT_PATH_DIRS: &[&str] = &["compositor", "input", "rendering"];
const HOT_PATH_PATTERNS: &[&str] = &[
    "services::config::load_",
    "std::fs::",
    "std::env::var",
    "gtk4::",
    "std::process::Command",
];

fn hot_path_violations(root: &Path, base: &Baseline) -> Vec<String> {
    let src = source_dir(root);
    let mut violations = Vec::new();
    for dir in HOT_PATH_DIRS {
        for file in rust_files(&src.join(dir)) {
            let Ok(content) = fs::read_to_string(&file) else {
                continue;
            };
            let path = relative(&file, root);
            let code = strip_comments(&content);
            for pattern in HOT_PATH_PATTERNS {
                if code.contains(pattern) {
                    let key = format!("{path}|{pattern}");
                    if !base.hot_path.contains(&key) {
                        violations.push(format!("hot-path: {path} uses {pattern}"));
                    }
                }
            }
        }
    }
    violations
}

fn unsafe_violations(root: &Path, base: &Baseline) -> Vec<String> {
    let src = source_dir(root);
    let mut violations = Vec::new();
    for file in rust_files(&src) {
        let Ok(content) = fs::read_to_string(&file) else {
            continue;
        };
        let code = strip_comments(&content);
        let lines: Vec<&str> = code.lines().collect();
        let path = relative(&file, root);
        for (index, line) in lines.iter().enumerate() {
            let trimmed = line.trim_start();
            if !line.contains("unsafe")
                || trimmed.starts_with("// SAFETY")
                || trimmed.starts_with('#')
            {
                continue;
            }
            let documented = lines
                .iter()
                .rev()
                .skip(lines.len() - index)
                .take(3)
                .any(|candidate| candidate.contains("// SAFETY"));
            if documented {
                continue;
            }
            let key = format!("{path}|{}", index + 1);
            if !base.unsafe_blocks.contains(&key) {
                violations.push(format!("unsafe: {path}:{} lacks // SAFETY", index + 1));
            }
        }
    }
    violations
}

fn collect(root: &Path) -> Result<Baseline, String> {
    let arch = architecture(root)?;
    let mut baseline = Baseline::default();
    let src = source_dir(root);
    for file in rust_files(&src) {
        let Ok(content) = fs::read_to_string(&file) else {
            continue;
        };
        let path = relative(&file, root);
        if let Some(from) = layer_for(&file, &src) {
            let allowed = arch.matrix.get(&from).cloned().unwrap_or_default();
            let code = strip_comments(&content);
            for to in crate_targets(&code) {
                let is_self = to == from || (from == "shell" && to == "extensions");
                if !is_self && !allowed.iter().any(|item| item == &to) {
                    baseline.layering.insert(format!("{path}|{from}|{to}"));
                }
            }
        }
        if content.lines().any(|line| {
            let trimmed = line.trim_start();
            trimmed.starts_with("pub ") && !trimmed.starts_with("pub(crate)")
        }) {
            baseline.visibility.insert(path.clone());
        }
        let code = strip_comments(&content);
        for dir in HOT_PATH_DIRS {
            if path.contains(&format!("/{dir}/")) {
                for pattern in HOT_PATH_PATTERNS {
                    if code.contains(pattern) {
                        baseline.hot_path.insert(format!("{path}|{pattern}"));
                    }
                }
            }
        }
        let lines: Vec<&str> = code.lines().collect();
        for (index, line) in lines.iter().enumerate() {
            let trimmed = line.trim_start();
            if !line.contains("unsafe")
                || trimmed.starts_with("// SAFETY")
                || trimmed.starts_with('#')
            {
                continue;
            }
            let documented = lines
                .iter()
                .rev()
                .skip(lines.len() - index)
                .take(3)
                .any(|candidate| candidate.contains("// SAFETY"));
            if !documented {
                baseline
                    .unsafe_blocks
                    .insert(format!("{path}|{}", index + 1));
            }
        }
    }
    Ok(baseline)
}

fn write_baseline(root: &Path) -> Result<(), String> {
    let baseline = collect(root)?;
    let path = root.join("xtask").join("baseline.toml");
    let content = toml::to_string_pretty(&baseline).map_err(|e| e.to_string())?;
    fs::write(path, content).map_err(|e| e.to_string())
}

fn check(root: &Path) -> Result<(), String> {
    let arch = architecture(root)?;
    let base = baseline(root)?;
    let mut violations = Vec::new();
    violations.extend(layering_violations(root, &arch, &base));
    violations.extend(visibility_violations(root, &base));
    violations.extend(hot_path_violations(root, &base));
    violations.extend(unsafe_violations(root, &base));

    if violations.is_empty() {
        println!("architecture checks passed");
        Ok(())
    } else {
        for violation in &violations {
            eprintln!("{violation}");
        }
        Err(format!("{} new violation(s)", violations.len()))
    }
}
