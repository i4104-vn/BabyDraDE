//! Atomic file-queue transport.
//!
//! Each message is one file named by its monotonically increasing sequence.
//! Writers create a temporary file and atomically rename it, so a reader never
//! sees a half-written message. Readers sort by filename and delete what they
//! consumed, giving FIFO ordering and at-most-once delivery.

use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use super::{paths, Transport};

/// File-backed transport for one queue directory.
#[derive(Debug, Clone)]
pub struct FileTransport {
    dir: PathBuf,
}

impl FileTransport {
    /// Opens the named queue, creating it when needed.
    ///
    /// # Arguments
    /// * `queue` - Queue name from [`paths`] (for example `requests`).
    pub fn new(queue: &str) -> Self {
        let dir = paths::queue_dir(queue);
        let _ = fs::create_dir_all(&dir);
        Self { dir }
    }

    /// Returns the queue directory.
    pub fn dir(&self) -> &Path {
        &self.dir
    }

    fn next_name(&self) -> String {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        format!("{nanos:020}-{}.msg", std::process::id())
    }
}

impl Transport for FileTransport {
    fn send(&self, bytes: &[u8]) -> Result<(), std::io::Error> {
        fs::create_dir_all(&self.dir)?;
        let final_path = self.dir.join(self.next_name());
        let temp_path = self.dir.join(format!(".{}", final_path.file_name().unwrap().to_string_lossy()));
        fs::write(&temp_path, bytes)?;
        fs::rename(temp_path, final_path)
    }

    fn drain(&self) -> Result<Vec<Vec<u8>>, std::io::Error> {
        let mut entries = Vec::new();
        for entry in fs::read_dir(&self.dir)? {
            let entry = entry?;
            let name = entry.file_name().to_string_lossy().into_owned();
            if name.ends_with(".msg") {
                entries.push((name, entry.path()));
            }
        }
        entries.sort_by(|a, b| a.0.cmp(&b.0));
        let mut messages = Vec::with_capacity(entries.len());
        for (_name, path) in entries {
            if let Ok(bytes) = fs::read(&path) {
                let _ = fs::remove_file(&path);
                messages.push(bytes);
            }
        }
        Ok(messages)
    }
}
