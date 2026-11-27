//! Render element types and helpers shared by the compositor and shell.

use std::sync::{Arc, Mutex, MutexGuard};

use crate::modules::blur::BlurEngine;
pub(crate) use crate::modules::render::models::elements::{
    ConstrainedSurface, CustomRenderElements, LayerBlurId, WindowBlurId,
};
pub(crate) use layer::render_layer_surfaces;
pub(crate) use window::render_window;

pub(crate) mod layer;
pub(crate) mod window;

/// Locks the shared blur engine, recovering a poisoned mutex with a warning
/// instead of silently dropping blur for the frame.
pub(crate) fn blur_engine(lock: &Arc<Mutex<BlurEngine>>) -> MutexGuard<'_, BlurEngine> {
    lock.lock().unwrap_or_else(|poisoned| {
        tracing::warn!("recovering poisoned blur engine lock");
        poisoned.into_inner()
    })
}
