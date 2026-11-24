//! Bounded filesystem index data types.

use std::path::PathBuf;

use super::super::SearchItem;

/// Total number of paths kept in the in-memory file index.
pub(super) const MAX_ENTRIES: usize = 400;
/// How deep below `$HOME` the one-shot scan is allowed to descend.
pub(super) const MAX_DEPTH: usize = 3;
/// Directories that are large, private or irrelevant to a launcher search.
pub(super) const SKIP_DIRS: &[&str] = &[
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
pub(super) struct FileEntry {
    pub(super) name: String,
    pub(super) lower: String,
    pub(super) path: PathBuf,
    pub(super) is_dir: bool,
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
            super::handler::scan_dir(&home, 0, &mut entries, &mut budget);
        }
        entries.sort_by(|a, b| a.lower.cmp(&b.lower));
        Self { entries }
    }

    /// Returns the best-matching paths for `query`, ordered by relevance.
    pub(crate) fn matches(&self, query: &str, limit: usize) -> Vec<SearchItem> {
        let mut scored: Vec<(usize, &FileEntry)> = self
            .entries
            .iter()
            .filter_map(|entry| {
                super::handler::score(&entry.lower, query).map(|score| (score, entry))
            })
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
