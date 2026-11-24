//! Data contracts for the bounded home-directory snapshot.

use std::path::PathBuf;

/// Total number of paths kept in an in-memory index.
pub const MAX_ENTRIES: usize = 400;
/// How deep below `$HOME` the one-shot scan may descend.
pub const MAX_DEPTH: usize = 3;
/// Directories that are large, private or irrelevant to a launcher search.
pub const SKIP_DIRS: &[&str] = &[
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
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FileEntry {
    /// File name as reported by the filesystem.
    pub name: String,
    /// Lower-cased name used for case-insensitive matching.
    pub lower: String,
    /// Full path of the entry.
    pub path: PathBuf,
    /// Whether the entry is a directory rather than a regular file.
    pub is_dir: bool,
}
