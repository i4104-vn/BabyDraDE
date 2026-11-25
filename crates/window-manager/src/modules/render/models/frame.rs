//! Frame composition input model.

use std::sync::{Arc, Mutex};

use smithay::{
    desktop::{Space, Window},
    output::Output,
    utils::Scale,
};

use crate::modules::blur::BlurEngine;
use crate::shared::types::WindowVisualState;

use super::CustomRenderElements;

/// Inputs that are produced by the backend or shell before composition.
pub(crate) struct FrameContext<'a> {
    pub(crate) output: &'a Output,
    pub(crate) scale: Scale<f64>,
    pub(crate) cursor: Option<CustomRenderElements>,
    /// Shell elements rendered above layer-shell windows but below client windows.
    pub(crate) below_windows: Vec<CustomRenderElements>,
    /// Shell elements rendered above client windows (for example the switcher).
    pub(crate) above_windows: Vec<CustomRenderElements>,
    pub(crate) space: &'a Space<Window>,
    pub(crate) blur_engine: Option<&'a Arc<Mutex<BlurEngine>>>,
    pub(crate) pointer: smithay::utils::Point<f64, smithay::utils::Logical>,
    pub(crate) visuals: &'a dyn Fn(&Window) -> WindowVisualState,
}
