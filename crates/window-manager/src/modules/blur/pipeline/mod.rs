//! Dual Kawase blur pipeline: capture, blur and composite a backdrop region.

mod projection;
mod state;

pub(crate) mod passes;

pub(super) use crate::modules::blur::models::BlurCache;
pub(super) use passes::{run_blur, run_composite, BlurPassTarget};
pub(super) use state::BlurPipeline;
