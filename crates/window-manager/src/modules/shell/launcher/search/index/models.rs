//! Bounded filesystem snapshot used for launcher result construction.

use services::files::index::{scan_home, FileEntry, MAX_DEPTH, MAX_ENTRIES, SKIP_DIRS};

use super::super::SearchItem;

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
    /// Requests a one-shot snapshot through the filesystem service.
    ///
    /// # Returns
    /// Entries sorted by lower-cased name, capped at [`MAX_ENTRIES`].
    pub(crate) fn new() -> Self {
        Self {
            entries: scan_home(MAX_ENTRIES, MAX_DEPTH, SKIP_DIRS),
        }
    }

    /// Returns the best-matching paths for `query`, ordered by relevance.
    ///
    /// # Arguments
    /// * `query` - Lower-cased search needle.
    /// * `limit` - Maximum number of rows to return.
    ///
    /// # Returns
    /// File and folder rows ordered by score, then by lower-cased name.
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

    /// Builds an index from prepared entries for tests.
    #[cfg(test)]
    fn from_entries(entries: Vec<FileEntry>) -> Self {
        Self { entries }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    fn entry(name: &str) -> FileEntry {
        FileEntry {
            lower: name.to_lowercase(),
            name: name.to_string(),
            path: PathBuf::from(name),
            is_dir: false,
        }
    }

    #[test]
    fn should_rank_contiguous_matches_then_alphabetical_ties_and_respect_limit() {
        let index = FileIndex::from_entries(vec![
            entry("zebra.txt"),
            entry("Alps"),
            entry("a-long-log.txt"),
            entry("album"),
        ]);

        let items = index.matches("al", 2);
        let labels: Vec<&str> = items.iter().map(|item| item.label.as_str()).collect();

        assert_eq!(labels, ["album", "Alps"]);
    }
}
