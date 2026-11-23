//! Result construction for the launcher search overlay.
//!
//! Applications come from the already-loaded launcher list, files/folders come
//! from a small, bounded index that is built once at startup, and the web row
//! is synthesized from the raw query. Nothing here touches the filesystem on a
//! keystroke, so typing never stalls the render loop.

use std::{
    fs,
    path::{Path, PathBuf},
};

use common::constants::launcher::SEARCH_ROWS_PER_CATEGORY;

use super::super::LauncherApp;
use super::SearchItem;

/// Total number of paths kept in the in-memory file index.
const MAX_ENTRIES: usize = 400;
/// How deep below `$HOME` the one-shot scan is allowed to descend.
const MAX_DEPTH: usize = 3;
/// Directories that are large, private or irrelevant to a launcher search.
const SKIP_DIRS: &[&str] = &[
    ".cache",
    ".config",
    ".local",
    ".git",
    ".cargo",
    ".rustup",
    ".npm",
    ".pnpm-store",
    "node_modules",
    "target",
    "Library",
    ".steam",
    ".wine",
    ".dbus",
];

/// A single indexed filesystem entry.
struct FileEntry {
    name: String,
    lower: String,
    path: PathBuf,
    is_dir: bool,
}

/// Bounded, immutable snapshot of the user's common folders.
pub(crate) struct FileIndex {
    entries: Vec<FileEntry>,
}

impl Default for FileIndex {
    fn default() -> Self {
        Self::new()
    }
}

impl FileIndex {
    /// Scans `$HOME` once. Hidden files and well-known heavy directories are
    /// skipped and the walk is capped so startup cost stays negligible.
    pub(crate) fn new() -> Self {
        let mut entries = Vec::with_capacity(MAX_ENTRIES);
        let mut budget = MAX_ENTRIES;
        if let Some(home) = dirs::home_dir() {
            scan_dir(&home, 0, &mut entries, &mut budget);
        }
        entries.sort_by(|a, b| a.lower.cmp(&b.lower));
        Self { entries }
    }

    /// Returns the best-matching paths for `query`, ordered by relevance.
    pub(crate) fn matches(&self, query: &str, limit: usize) -> Vec<SearchItem> {
        let mut scored: Vec<(usize, &FileEntry)> = self
            .entries
            .iter()
            .filter_map(|entry| score(&entry.lower, query).map(|score| (score, entry)))
            .collect();
        scored.sort_by(|(a, left), (b, right)| a.cmp(b).then_with(|| left.lower.cmp(&right.lower)));
        scored
            .into_iter()
            .take(limit)
            .map(|(_, entry)| {
                SearchItem::file(entry.path.clone(), entry.name.clone(), entry.is_dir)
            })
            .collect()
    }
}

fn scan_dir(path: &Path, depth: usize, out: &mut Vec<FileEntry>, budget: &mut usize) {
    if depth > MAX_DEPTH || *budget == 0 {
        return;
    }
    let Ok(entries) = fs::read_dir(path) else {
        return;
    };
    for entry in entries.flatten() {
        if *budget == 0 {
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
        if is_dir && SKIP_DIRS.contains(&name.as_str()) {
            continue;
        }
        let path = entry.path();
        out.push(FileEntry {
            lower: name.to_lowercase(),
            name,
            path: path.clone(),
            is_dir,
        });
        *budget -= 1;
        if is_dir {
            scan_dir(&path, depth + 1, out, budget);
        }
    }
}

/// Lower score means a better match. `None` means "does not match at all".
fn score(haystack: &str, needle: &str) -> Option<usize> {
    if needle.is_empty() {
        return Some(0);
    }
    if let Some(position) = haystack.find(needle) {
        return Some(if position == 0 { 0 } else { 10 + position });
    }
    // Subsequence fallback: "fdr" still finds "folder", but ranks below every
    // contiguous match. The score counts skipped characters so tighter runs win.
    subsequence_score(haystack, needle)
}

fn subsequence_score(haystack: &str, needle: &str) -> Option<usize> {
    let mut previous: Option<usize> = None;
    let mut gaps = 0usize;
    for wanted in needle.chars() {
        let start = previous.map_or(0, |p| p + 1);
        let found = haystack
            .chars()
            .enumerate()
            .find(|(index, ch)| *index >= start && *ch == wanted)
            .map(|(index, _)| index)?;
        if let Some(prev) = previous {
            gaps += found - prev - 1;
        }
        previous = Some(found);
    }
    Some(500 + gaps)
}

/// Builds the three ordered sections from the current query.
pub(crate) fn build_results(
    query: &str,
    apps: &[LauncherApp],
    files: &FileIndex,
) -> Vec<SearchItem> {
    if query.trim().is_empty() {
        return Vec::new();
    }
    let needle = query.trim().to_lowercase();

    let mut apps_out: Vec<(usize, SearchItem)> = apps
        .iter()
        .enumerate()
        .filter_map(|(index, app)| {
            score(&app.name.to_lowercase(), &needle).map(|value| {
                (
                    value,
                    SearchItem::app(index, app.name.clone(), app.icon.clone()),
                )
            })
        })
        .collect();
    apps_out.sort_by_key(|(score, _)| *score);

    let mut results: Vec<SearchItem> = apps_out
        .into_iter()
        .take(SEARCH_ROWS_PER_CATEGORY)
        .map(|(_, item)| item)
        .collect();
    results.extend(files.matches(&needle, SEARCH_ROWS_PER_CATEGORY));
    results.push(SearchItem::web(query.trim()));
    results
}

/// Minimal RFC-3986 escaping for a search URL. Keeps this crate dependency-free
/// and handles the characters users actually type into a launcher query.
pub(crate) fn url_encode(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for byte in input.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(byte as char)
            }
            _ => out.push_str(&format!("%{byte:02X}")),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn substring_matches_outrank_subsequence_matches() {
        let contiguous = score("firefox", "fire").unwrap();
        let subsequence = score("file manager", "fr").unwrap();
        assert!(contiguous < subsequence);
        assert!(score("notes", "xyz").is_none());
    }

    #[test]
    fn web_query_is_escaped() {
        assert_eq!(url_encode("a b&c"), "a%20b%26c");
    }
}
