//! Configuration adapters.
//!
//! All filesystem access, default export, watcher threads and parsing live
//! here. Adapters receive ready-made snapshots instead of reading config.

pub(crate) mod gesture;
pub(crate) mod keymap;

pub(crate) use gesture::gesture_policy;
