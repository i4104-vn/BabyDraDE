//! GPU blur pipeline setup and frame rendering.

use smithay::{
    backend::renderer::element::Id,
    utils::{Physical, Rectangle},
};
use tracing::error;

use common::constants::{BLUR_CACHE_LIFETIME, MAX_BLUR_CACHE_BYTES};

use super::super::pipeline::{BlurCache, BlurPipeline};
use super::BlurEngine;

impl BlurEngine {
    pub(crate) fn init_pipeline(&mut self, gl: &glow::Context) {
        if self.pipeline.is_some() || self.pipeline_failed {
            return;
        }
        // SAFETY: GL context is current and object names come from this pipeline.
        unsafe {
            match BlurPipeline::new(gl) {
                Ok(p) => self.pipeline = Some(p),
                Err(err) => {
                    error!("Failed to initialize BabyDraDE BlurPipeline: {err}");
                    self.pipeline_failed = true;
                }
            }
        }
    }

    /// Render Dual Kawase blur over the destination physical rectangle.
    ///
    /// Captures the framebuffer background, blurs it, and composites the result
    /// on every call. GPU texture objects are cached per element Id so that the
    /// allocation cost is paid only once per blurred window.
    pub(crate) fn render_blur(
        &mut self,
        gl: &glow::Context,
        id: &Id,
        dst: Rectangle<i32, Physical>,
        corner_radius: f32,
        projection: [f32; 9],
    ) {
        if !self.params.enabled || self.pipeline_failed {
            return;
        }

        if self.pipeline.is_none() {
            self.init_pipeline(gl);
            if self.pipeline.is_none() {
                return;
            }
        }

        // Take the pipeline out of the struct so that the caches can be borrowed
        // at the same time.
        let Some(mut pipeline) = self.pipeline.take() else {
            return;
        };

        let frame = self.frame;
        self.frame = self.frame.wrapping_add(1);

        if !self.backdrops.contains_key(id) {
            // SAFETY: GL context is current and object names come from this pipeline.
            let cache = match unsafe { BlurCache::new(gl) } {
                Ok(cache) => Some(cache),
                Err(err) => {
                    error!("Failed to create the BabyDraDE blur cache: {err}");
                    None
                }
            };
            self.backdrops.insert(id.clone(), cache);
        }

        let Some(Some(cache)) = self.backdrops.get_mut(id) else {
            self.pipeline = Some(pipeline);
            return;
        };
        cache.last_used = frame;

        // SAFETY: GL context is current and object names come from this pipeline.
        unsafe {
            pipeline.render_blur(gl, dst, corner_radius, projection, &self.params, cache);
        }

        self.pipeline = Some(pipeline);

        self.prune_caches(gl, id, frame);
    }

    /// Drops the caches of elements that are gone and keeps the memory usage
    /// of the remaining ones bounded.
    fn prune_caches(&mut self, gl: &glow::Context, keep: &Id, frame: u64) {
        let mut expired = Vec::new();
        self.backdrops.retain(|id, cache| {
            let last_used = cache.as_ref().map(|cache| cache.last_used).unwrap_or(0);
            if frame.saturating_sub(last_used) >= BLUR_CACHE_LIFETIME {
                expired.push(id.clone());
                false
            } else {
                true
            }
        });

        for id in expired {
            if let Some(Some(mut cache)) = self.backdrops.remove(&id) {
                // SAFETY: GL context is current and object names come from this pipeline.
                unsafe { cache.release(gl) };
            }
        }

        let mut total: usize = self
            .backdrops
            .values()
            .filter_map(|cache| cache.as_ref())
            .map(|cache| cache.bytes)
            .sum();
        if total <= MAX_BLUR_CACHE_BYTES {
            return;
        }

        // Drop the least recently used caches until the memory usage is back
        // below the limit.
        let mut entries: Vec<(u64, Id)> = self
            .backdrops
            .iter()
            .filter_map(|(id, cache)| cache.as_ref().map(|cache| (cache.last_used, id.clone())))
            .collect();
        entries.sort_by_key(|(last_used, _)| *last_used);

        for (_, id) in entries {
            if total <= MAX_BLUR_CACHE_BYTES {
                break;
            }
            if id == *keep {
                continue;
            }
            if let Some(Some(mut cache)) = self.backdrops.remove(&id) {
                total = total.saturating_sub(cache.bytes);
                // SAFETY: GL context is current and object names come from this pipeline.
                unsafe { cache.release(gl) };
            }
        }
    }
}
