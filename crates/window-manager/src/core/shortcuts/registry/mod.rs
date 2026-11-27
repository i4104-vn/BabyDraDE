//! Runtime shortcut registry shared by the public API and the keyboard path.
//!
//! Config-driven shortcuts are replaced wholesale on reload; app-owned
//! registrations live here so they survive keymap reloads and can be
//! added, replaced or removed independently by their owner.

mod entry;
mod error;
mod store;

#[cfg(test)]
mod tests;

pub use error::RegistrationError;
pub use store::ShortcutRegistry;
