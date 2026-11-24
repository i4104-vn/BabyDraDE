//! Bounded recursive walk that produces the home-directory snapshot.

use std::fs;
use std::path::Path;

use super::models::FileEntry;

/// Scans `$HOME` once and returns a sorted, bounded snapshot of its entries.
///
/// # Arguments
/// * `limit` - Maximum number of entries to collect.
/// * `depth` - Maximum recursion depth below the home directory.
/// * `skip_dirs` - Directory names to prune from the walk.
///
/// # Returns
/// The collected entries sorted by lower-cased name, or an empty vector when
/// the home directory cannot be determined.
pub fn scan_home(limit: usize, depth: usize, skip_dirs: &[&str]) -> Vec<FileEntry> {
    let Some(home) = dirs::home_dir() else {
        return Vec::new();
    };
    scan_root(&home, limit, depth, skip_dirs)
}

/// Scans an explicit root with the same rules as [`scan_home`].
///
/// # Arguments
/// * `root` - Directory to start the walk from.
/// * `limit` - Maximum number of entries to collect.
/// * `depth` - Maximum recursion depth below `root`.
/// * `skip_dirs` - Directory names to prune from the walk.
///
/// # Returns
/// The collected entries sorted by lower-cased name.
pub(super) fn scan_root(
    root: &Path,
    limit: usize,
    depth: usize,
    skip_dirs: &[&str],
) -> Vec<FileEntry> {
    let mut entries = Vec::with_capacity(limit);
    let mut budget = limit;
    let mut walk = Walk {
        max_depth: depth,
        skip_dirs,
        entries: &mut entries,
        budget: &mut budget,
    };
    walk.visit(root, 0);
    entries.sort_by(|left, right| left.lower.cmp(&right.lower));
    entries
}

/// Parameters and output of one bounded walk.
struct Walk<'a> {
    max_depth: usize,
    skip_dirs: &'a [&'a str],
    entries: &'a mut Vec<FileEntry>,
    budget: &'a mut usize,
}

impl Walk<'_> {
    /// Collects the entries of one directory and descends into its folders.
    fn visit(&mut self, path: &Path, depth: usize) {
        if depth > self.max_depth || *self.budget == 0 {
            return;
        }
        let Ok(entries) = fs::read_dir(path) else {
            return;
        };
        for entry in entries.flatten() {
            if *self.budget == 0 {
                break;
            }
            let name = entry.file_name().to_string_lossy().into_owned();
            if depth > 0 && name.starts_with('.') {
                continue;
            }
            let Ok(kind) = entry.file_type() else {
                continue;
            };
            let is_dir = kind.is_dir();
            if !is_dir && !kind.is_file() {
                continue;
            }
            if is_dir && self.skip_dirs.contains(&name.as_str()) {
                continue;
            }
            let path = entry.path();
            self.entries.push(FileEntry {
                lower: name.to_lowercase(),
                name,
                path: path.clone(),
                is_dir,
            });
            *self.budget -= 1;
            if is_dir {
                self.visit(&path, depth + 1);
            }
        }
    }
}
