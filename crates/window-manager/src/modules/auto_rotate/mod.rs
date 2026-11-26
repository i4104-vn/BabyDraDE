//! Automatic panel rotation from device orientation sensors.
//!
//! The feature is split into a driver layer (pluggable sensor sources and a
//! selection/fallback factory), a pure rotation controller and the event-loop
//! service that connects both to compositor state. Machines without a usable
//! sensor are a no-op.

mod controller;
mod models;
mod service;

pub(crate) use service::spawn;
