//! Model types shared by the input feature.

pub(crate) use config::{MouseConfig, TouchpadConfig};
pub(crate) use gesture::WindowIntent;
pub(crate) use state::{ButtonContext, KeyContext};

pub(crate) mod config;
pub(crate) mod gesture;
pub(crate) mod state;
