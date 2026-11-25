//! Product identity, command launching and background watchers.

#[allow(unused_imports)]
pub(crate) mod auto_rotate;
pub(crate) mod brand;
pub(crate) mod process;
pub(crate) mod shortcuts_watcher;

#[allow(unused_imports)]
pub(crate) use brand::{DISPLAY_NAME, ID};
#[allow(unused_imports)]
pub(crate) use process::{ProcessLauncher, ShortcutCommandLauncher};
#[allow(unused_imports)]
pub(crate) use shortcuts_watcher::spawn_watcher;
