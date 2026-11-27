//! State for the launcher's search overlay.
//!
//! The `SearchText` model is the source of truth for the query. Everything
//! else is derived once per frame: results come from the query string, the
//! dimmer and stagger come from one timeline, and the cache key is built from
//! the values that are actually read while drawing.

pub(crate) use models::SearchState;

mod handler;

mod models;
