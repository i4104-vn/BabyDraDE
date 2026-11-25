//! Blur engine state model.

use std::collections::HashMap;

use smithay::backend::renderer::element::Id;

use super::super::config::BlurParams;
use super::super::pipeline::{BlurCache, BlurPipeline};

// Blur engine state model.
pub(crate) struct BlurEngine {
    pub(crate) params: BlurParams,
    pub(in crate::modules::blur) rules_lower: Vec<String>,
    pub(in crate::modules::blur) pipeline: Option<BlurPipeline>,
    /// Set when the blur pipeline could not be created (shader compilation,
    /// missing GL features, ...). Blur is disabled in that case instead of
    /// retrying - and forcing a full redraw - on every frame.
    pub(in crate::modules::blur) pipeline_failed: bool,
    /// Per element caches holding the captured background and the blurred
    /// result. `None` marks an element whose offscreen target could not be
    /// created, so that it also stops forcing full redraws.
    pub(in crate::modules::blur) backdrops: HashMap<Id, Option<BlurCache>>,
    /// Counter used to expire caches of elements that are gone.
    pub(in crate::modules::blur) frame: u64,
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

