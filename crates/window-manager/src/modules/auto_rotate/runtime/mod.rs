//! Event-loop wiring for automatic rotation.

mod service;
mod target;

pub(crate) use service::spawn;
