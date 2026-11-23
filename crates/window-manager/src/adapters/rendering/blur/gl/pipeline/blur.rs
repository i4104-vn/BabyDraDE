//! Dual Kawase blur orchestration: downsample then upsample.

use crate::adapters::rendering::blur::config::BlurParams;

use super::cache::BlurCache;
use super::{downsample, upsample, BlurPipeline};

/// Runs the dual kawase pass on the cached backdrop and stores the result in
/// the cache, so that it can be re-composited later on.
///
/// # Safety
///
/// Must be invoked while the OpenGL/EGL context is current.
pub(super) unsafe fn run(
    gl: &glow::Context,
    pipeline: &BlurPipeline,
    region_w: i32,
    region_h: i32,
    params: &BlurParams,
    cache: &mut BlurCache,
) {
    downsample::run(gl, pipeline, cache, region_w, region_h, params);
    upsample::run(gl, pipeline, cache, region_w, region_h, params);
}
