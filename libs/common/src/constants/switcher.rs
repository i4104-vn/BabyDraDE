//! Stage Manager and Alt+Tab switcher layout constants.

/// Maximum number of visible thumbnails displayed in the dock carousel.
pub const DOCK_MAX_VISIBLE_ITEMS: usize = 5;

/// Left offset of the dock from the output's left edge.
pub const DOCK_X: f64 = 24.0;

/// Top margin reserved above the dock.
pub const DOCK_TOP_MARGIN: f64 = 32.0;

/// Bottom margin reserved below the dock.
pub const DOCK_BOTTOM_MARGIN: f64 = 32.0;

/// Fixed width of each dock thumbnail card.
pub const DOCK_THUMBNAIL_WIDTH: f64 = 175.0;

/// Fixed height of each dock thumbnail card.
pub const DOCK_THUMBNAIL_HEIGHT: f64 = 115.0;

/// Vertical spacing gap between consecutive dock cards.
pub const DOCK_GAP: f64 = 12.0;

/// Horizontal gap between the dock and the stage preview area.
pub const STAGE_SIDE_GAP: f64 = 28.0;

/// Right margin between the stage preview area and the output's right edge.
pub const STAGE_RIGHT_MARGIN: f64 = 24.0;

/// Standard aspect ratio (16:9) targeted by the stage preview.
pub const STAGE_ASPECT_RATIO: f64 = 16.0 / 9.0;

/// Horizontal slide animation offset distance during stage item transitions.
pub const STAGE_SLIDE_DISTANCE: f64 = 24.0;

/// How far the stage grows at the peak of a switch pulse (0..1).
/// Growing instead of shrinking keeps the preview covering its slot, so the
/// dark canvas can never show through as a black border around the app.
pub const STAGE_SCALE_GROW: f64 = 0.02;
