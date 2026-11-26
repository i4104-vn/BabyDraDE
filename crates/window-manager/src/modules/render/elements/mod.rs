//! Render element types and helpers shared by the compositor and shell.

pub(crate) use crate::modules::render::models::elements::{
    ConstrainedSurface, CustomRenderElements, LayerBlurId, WindowBlurId,
};
pub(crate) use layer::render_layer_surfaces;
pub(crate) use window::render_window;

pub(crate) mod layer;
pub(crate) mod window;
