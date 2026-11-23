//! Search overlay owned by the launcher.
//!
//! Typing any printable key while the docked launcher is open expands it into a
//! centered, full-screen search surface. Results are grouped into three stable
//! categories — applications, files/folders and a web-search fallback — so the
//! panel layout never depends on which category happened to match first.
//!
//! The split mirrors the rest of the desktop adapters: [`state`] owns mutable
//! input/animation state, [`index`] builds the result set, [`layout`] computes
//! geometry shared by drawing and hit-testing, [`draw`] owns Cairo drawing and
//! [`buffer`] owns the off-screen pixel work.

pub(crate) mod buffer;
pub(crate) mod draw;
pub(crate) mod index;
pub(crate) mod layout;
pub(crate) mod state;

use std::path::PathBuf;

use gtk4::gdk_pixbuf::Pixbuf;

pub(crate) use state::SearchState;

/// A physical region covered by an off-screen buffer: `(x, y, width, height)`.
pub(crate) type SearchRegion = (i32, i32, i32, i32);
/// Panel cache key: result revision, animation tick and physical region.
pub(crate) type SearchPanelKey = (u64, u64, SearchRegion);
/// Full-screen dimmer cache key: physical size, animation tick and presence bits.
pub(crate) type SearchBackdropKey = ((i32, i32), u64, u64);

/// The three fixed sections of the search panel, in display order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SearchCategory {
    Apps,
    Files,
    Web,
}

impl SearchCategory {
    /// Localized heading drawn above the first row of this section.
    pub(crate) fn title(self) -> String {
        match self {
            Self::Apps => services::i18n::trans("launcher.apps"),
            Self::Files => services::i18n::trans("launcher.files_dirs"),
            Self::Web => services::i18n::trans("launcher.web_search"),
        }
    }
}

/// What happens when a result row is activated with Enter or a click.
#[derive(Clone, Debug)]
pub(crate) enum SearchAction {
    /// Index into the launcher's application list.
    App(usize),
    /// Open a matching file or folder with its default handler.
    Path(PathBuf),
    /// Open the query in the default web browser.
    Web(String),
}

/// One selectable row in the search results.
#[derive(Clone, Debug)]
pub(crate) struct SearchItem {
    pub(crate) category: SearchCategory,
    pub(crate) label: String,
    pub(crate) detail: String,
    /// Application icon; `None` makes the drawer paint a shape-based badge.
    pub(crate) icon: Option<Pixbuf>,
    /// Whether the badge should look like a folder rather than a file.
    pub(crate) is_dir: bool,
    pub(crate) action: SearchAction,
}

impl SearchItem {
    pub(crate) fn app(index: usize, name: String, icon: Option<Pixbuf>) -> Self {
        Self {
            category: SearchCategory::Apps,
            detail: services::i18n::trans("launcher.apps"),
            icon,
            is_dir: false,
            label: name,
            action: SearchAction::App(index),
        }
    }

    pub(crate) fn file(path: PathBuf, name: String, is_dir: bool) -> Self {
        let detail = path
            .parent()
            .map(|parent| parent.to_string_lossy().into_owned())
            .unwrap_or_default();
        Self {
            category: SearchCategory::Files,
            detail,
            icon: None,
            is_dir,
            label: name,
            action: SearchAction::Path(path),
        }
    }

    pub(crate) fn web(query: &str) -> Self {
        Self {
            category: SearchCategory::Web,
            detail: services::i18n::trans("launcher.web_desc"),
            icon: None,
            is_dir: false,
            label: services::i18n::trans("launcher.google_search").replace("{}", query),
            action: SearchAction::Web(query.to_string()),
        }
    }
}
