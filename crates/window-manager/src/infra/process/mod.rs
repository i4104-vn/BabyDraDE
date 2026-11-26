//! Product identity, command launching and background watchers.

#[allow(unused_imports)]
pub(crate) mod auto_rotate;
pub(crate) mod brand;
pub(crate) mod process;

#[allow(unused_imports)]
pub(crate) use brand::{DISPLAY_NAME, ID};
#[allow(unused_imports)]
pub(crate) use process::{ProcessLauncher, ShortcutCommandLauncher};
