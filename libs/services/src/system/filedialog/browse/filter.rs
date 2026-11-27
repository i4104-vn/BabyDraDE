//! Glob and MIME matching used by the file chooser filter dropdown.

use std::path::Path;

use gio::content_type_guess;

use super::super::FileFilter;

/// Checks a filename against a simple glob such as `*.png`, `foo.*` or `*`.
///
/// # Arguments
/// * `filename` - File name without its parent directory.
/// * `pattern` - Glob pattern to match.
///
/// # Returns
/// `true` when the name matches the pattern.
pub fn matches_pattern(filename: &str, pattern: &str) -> bool {
    let pat = pattern.trim().to_ascii_lowercase();
    let name = filename.to_ascii_lowercase();

    if pat == "*" || pat == "*.*" {
        return true;
    }
    if let Some(ext) = pat.strip_prefix("*.") {
        return name.ends_with(&format!(".{ext}"));
    }
    if let Some(prefix) = pat.strip_suffix(".*") {
        return name.starts_with(prefix);
    }
    name == pat
}

/// Checks whether a path satisfies a [`FileFilter`].
///
/// # Arguments
/// * `path` - Candidate file path.
/// * `filter` - Filter to apply.
///
/// # Returns
/// `true` when the path matches a glob pattern or one of the MIME types.
pub fn matches_filter(path: &Path, filter: &FileFilter) -> bool {
    if filter.patterns.is_empty() && filter.mime_types.is_empty() {
        return true;
    }

    let filename = path.file_name().and_then(|name| name.to_str());
    if filename.is_none() && filter.mime_types.is_empty() {
        return false;
    }

    if let Some(name) = filename {
        for pattern in &filter.patterns {
            if matches_pattern(name, pattern) {
                return true;
            }
        }
    }

    if !filter.mime_types.is_empty() {
        let (guessed, _) = content_type_guess(Some(path.as_os_str()), None::<&[u8]>);
        let guessed = guessed.to_ascii_lowercase();
        if filter
            .mime_types
            .iter()
            .any(|mime| mime.eq_ignore_ascii_case(&guessed))
        {
            return true;
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_match_case_insensitive_extension_patterns() {
        assert!(matches_pattern("photo.PNG", "*.png"));
        assert!(matches_pattern("photo.jpeg", "*.jpeg"));
        assert!(!matches_pattern("photo.jpg", "*.png"));
    }

    #[test]
    fn should_match_every_file_for_wildcards() {
        assert!(matches_pattern("note.txt", "*"));
        assert!(matches_pattern("note.txt", "*.*"));
    }

    #[test]
    fn should_accept_any_path_for_an_empty_filter() {
        let filter = FileFilter::new("All", &[]);
        assert!(matches_filter(Path::new("/tmp/note.txt"), &filter));
    }
}
