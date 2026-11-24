use std::collections::HashMap;

use smithay::{
    backend::renderer::element::Id,
    desktop::Window,
    utils::{Physical, Rectangle},
};
use tracing::error;
#[cfg(test)]
use tracing::info;

use super::{config, config::BlurParams, gl::BlurCache, gl::BlurPipeline};

use common::constants::{BLUR_CACHE_LIFETIME, MAX_BLUR_CACHE_BYTES};

pub(crate) struct BlurEngine {
    pub(crate) params: BlurParams,
    rules_lower: Vec<String>,
    pipeline: Option<BlurPipeline>,
    /// Set when the blur pipeline could not be created (shader compilation,
    /// missing GL features, ...). Blur is disabled in that case instead of
    /// retrying - and forcing a full redraw - on every frame.
    pipeline_failed: bool,
    /// Per element caches holding the captured background and the blurred
    /// result. `None` marks an element whose offscreen target could not be
    /// created, so that it also stops forcing full redraws.
    backdrops: HashMap<Id, Option<BlurCache>>,
    /// Counter used to expire caches of elements that are gone.
    frame: u64,
}

impl std::fmt::Debug for BlurEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BlurEngine")
            .field("params", &self.params)
            .field("has_pipeline", &self.pipeline.is_some())
            .field("pipeline_failed", &self.pipeline_failed)
            .field("cached_backdrops", &self.backdrops.len())
            .finish()
    }
}

impl Default for BlurEngine {
    fn default() -> Self {
        Self::new()
    }
}


mod matching;
mod render;
mod rules;
