use animation::Tween;
use smithay::utils::{Logical, Rectangle};

/// Start and end rectangles (in space/logical coordinates) of a geometry
/// transition such as fullscreen or maximize enter/exit.
#[derive(Debug, Clone, Copy)]
pub(crate) struct GeometryTransition {
    pub(crate) from: Rectangle<f64, Logical>,
    pub(crate) to: Rectangle<f64, Logical>,
}

/// Tracks in-progress window transitions.
///
/// All of them are render-only: the real window geometry is changed right away
/// and the renderer draws a temporary, interpolated element until the tween
/// finishes. `Minimizing` is the exception in that it keeps the window mapped
/// until the shrink completes so it can actually be seen collapsing.
#[derive(Debug, Clone)]
pub(crate) enum WindowAnimation {
    Opening(Tween),
    Closing(Tween),
    Minimizing(Tween),
    Restoring(Tween),
    Transition {
        tween: Tween,
        geometry: GeometryTransition,
    },
}

/// Parameters specifying animation progress and blur properties for a transition frame.
#[derive(Clone, Copy, Debug)]
pub(crate) struct WindowAnimationRenderParams {
    pub(crate) scale_factor: f64,
    pub(crate) alpha_factor: f32,
    /// Absolute interpolated rectangle for geometry transitions. When set, the
    /// window is rendered into this rect instead of being scaled around its
    /// current center.
    pub(crate) target_rect: Option<Rectangle<f64, Logical>>,
    pub(crate) base_opacity: f32,
    pub(crate) should_blur: bool,
    pub(crate) corner_radius: f32,
}
