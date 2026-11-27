//! Blur render element state.

use std::sync::{Arc, Mutex};

use smithay::backend::renderer::element::Id;
use smithay::utils::{Logical, Rectangle};

use super::super::engine::BlurEngine;

/// A render element that applies background Dual Kawase frosted glass blur
/// behind transparent/translucent windows and layer shell surfaces
#[derive(Debug, Clone)]
pub(crate) struct BlurRenderElement {
    pub(in crate::modules::blur) id: Id,
    pub(in crate::modules::blur) area: Rectangle<i32, Logical>,
    pub(in crate::modules::blur) corner_radius: f32,
    pub(in crate::modules::blur) blur_engine: Arc<Mutex<BlurEngine>>,
}

impl BlurRenderElement {
    pub(crate) fn new(
        id: Id,
        area: Rectangle<i32, Logical>,
        corner_radius: f32,
        blur_engine: Arc<Mutex<BlurEngine>>,
    ) -> Self {
        Self {
            id,
            area,
            corner_radius,
            blur_engine,
        }
    }
}
