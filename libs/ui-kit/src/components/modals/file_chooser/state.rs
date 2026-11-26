//! Mutable chooser state, directory loading, filtering and path confirmation.

use std::boxed::Box as StdBox;
use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::time::SystemTime;

use crate::trans;
use services::filedialog::filter::matches_filter;
use services::filedialog::{FileDialogMode, FileDialogOptions};

use super::format::file_icon;

/// One row shown in the file list.
#[derive(Clone)]
pub(super) struct FileRow {
    pub path: PathBuf,
    pub name: String,
    pub is_dir: bool,
    pub hidden: bool,
    pub size: u64,
    pub modified: Option<SystemTime>,
    pub icon: &'static str,
}

/// One-shot selection callback stored until the user confirms or cancels.
pub(super) type SelectionCallback = StdBox<dyn Fn(Option<Vec<PathBuf>>)>;
/// Shared slot holding the pending selection callback.
pub(super) type CallbackSlot = Rc<RefCell<Option<SelectionCallback>>>;
/// Directory navigation closure reused by breadcrumbs and toolbar buttons.
pub(super) type Navigator = Rc<dyn Fn(PathBuf, bool)>;
/// Slot that publishes the navigator after the widgets are built.
pub(super) type NavigateSlot = Rc<RefCell<Option<Navigator>>>;
/// Slot holding the list re-render callback.
pub(super) type RenderSlot = Rc<RefCell<Option<Rc<dyn Fn()>>>>;

/// Mutable state shared by the chooser's callbacks.
pub(super) struct DialogState {
    pub options: FileDialogOptions,
    pub current: PathBuf,
    pub back: Vec<PathBuf>,
    pub forward: Vec<PathBuf>,
    pub selected: Vec<PathBuf>,
    pub rows: Vec<FileRow>,
    pub displayed: Vec<FileRow>,
    pub filter_index: Option<usize>,
    pub query: String,
    pub show_hidden: bool,
}

/// Returns the localized label of the confirm button.
pub(super) fn confirm_label(options: &FileDialogOptions) -> String {
    options
        .accept_label
        .clone()
        .unwrap_or_else(|| match options.mode {
            FileDialogMode::OpenFile | FileDialogMode::OpenMultipleFiles => {
                trans("file_dialog.btn_open")
            }
            FileDialogMode::OpenFolder => trans("file_dialog.btn_select"),
            FileDialogMode::SaveFile => trans("file_dialog.btn_save"),
        })
}

/// Resolves the confirm action into concrete paths.
pub(super) fn confirm_paths(state: &Rc<RefCell<DialogState>>, typed: &str) -> Vec<PathBuf> {
    let borrowed = state.borrow();
    let current = borrowed.current.clone();
    match borrowed.options.mode {
        FileDialogMode::OpenFolder => {
            if !borrowed.selected.is_empty() && borrowed.selected[0].is_dir() {
                borrowed.selected.clone()
            } else if !typed.is_empty() {
                let candidate = resolve_typed(&current, typed);
                if candidate.is_dir() {
                    vec![candidate]
                } else {
                    vec![current]
                }
            } else {
                vec![current]
            }
        }
        FileDialogMode::SaveFile => {
            if typed.is_empty() {
                Vec::new()
            } else {
                vec![resolve_typed(&current, typed)]
            }
        }
        FileDialogMode::OpenFile | FileDialogMode::OpenMultipleFiles => {
            if !borrowed.selected.is_empty() {
                borrowed.selected.clone()
            } else if !typed.is_empty() {
                let candidate = resolve_typed(&current, typed);
                if candidate.exists() {
                    vec![candidate]
                } else {
                    Vec::new()
                }
            } else {
                Vec::new()
            }
        }
    }
}

fn resolve_typed(current: &Path, typed: &str) -> PathBuf {
    let path = PathBuf::from(typed);
    if path.is_absolute() {
        path
    } else {
        current.join(path)
    }
}

/// Returns the rows passing filter, search and hidden-file rules, sorted.
pub(super) fn visible_rows(state: &Rc<RefCell<DialogState>>) -> Vec<FileRow> {
    let mut borrowed = state.borrow_mut();
    let query = borrowed.query.to_ascii_lowercase();
    let mode = borrowed.options.mode;
    let filter = borrowed
        .filter_index
        .and_then(|index| borrowed.options.filters.get(index));

    let mut rows: Vec<FileRow> = borrowed
        .rows
        .iter()
        .filter(|row| {
            if row.hidden && !borrowed.show_hidden {
                return false;
            }
            if !query.is_empty() && !row.name.to_ascii_lowercase().contains(&query) {
                return false;
            }
            if !row.is_dir {
                if mode == FileDialogMode::OpenFolder {
                    return false;
                }
                if let Some(filter) = filter {
                    return matches_filter(&row.path, filter);
                }
            }
            true
        })
        .cloned()
        .collect();

    rows.sort_by(|a, b| match (a.is_dir, b.is_dir) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => a
            .name
            .to_ascii_lowercase()
            .cmp(&b.name.to_ascii_lowercase()),
    });
    borrowed.displayed.clone_from(&rows);
    rows
}

/// Reads a directory into chooseable rows.
pub(super) fn load_rows(directory: &Path) -> Vec<FileRow> {
    let Ok(entries) = std::fs::read_dir(directory) else {
        return Vec::new();
    };
    let mut rows = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().into_owned();
        let metadata = entry.metadata().ok();
        let is_dir = metadata.as_ref().is_some_and(|meta| meta.is_dir());
        rows.push(FileRow {
            hidden: name.starts_with('.'),
            icon: if is_dir { "folder" } else { file_icon(&name) },
            size: metadata.as_ref().map(|meta| meta.len()).unwrap_or(0),
            modified: metadata.and_then(|meta| meta.modified().ok()),
            path,
            name,
            is_dir,
        });
    }
    rows
}
