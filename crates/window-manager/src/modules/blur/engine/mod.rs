//! GPU blur engine facade, rule matching and frame rendering.

mod render;
mod rules;

#[cfg(test)]
mod tests;

pub(crate) use super::models::BlurEngine;
