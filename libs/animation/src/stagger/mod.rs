//! Sequential reveal driven by a single master clock.
//!
//! Maps one linear `0..=1` timeline onto independent, delayed progress values
//! so a leading element settles first and every following element arrives one
//! after another.

mod handler;
mod render;

pub use handler::Stagger;
