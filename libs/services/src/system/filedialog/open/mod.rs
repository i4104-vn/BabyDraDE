//! `xdg-open` compatible launcher for files, folders and URIs.
//!
//! GIO handles most URIs; BabyDra applications and desktop entries provide the
//! fallback chain so the compositor can own the default-open experience.

mod desktop_entry;
mod launch;
mod target;

#[cfg(test)]
mod tests;

pub use target::open_target;
