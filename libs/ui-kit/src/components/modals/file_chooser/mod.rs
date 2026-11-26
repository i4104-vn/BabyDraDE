//! Native file chooser widget used by the `xdg-open` replacement.

mod dialog;
mod format;
mod list;
mod sidebar;
mod state;
mod toolbar;

pub use dialog::show;
