//! Geometry and surface tokens for the compositor-owned application launcher.
//!
//! Kept in `common` so shell features and the compositor can share the exact
//! dock metrics without redefining them in drawing code.

/// Left offset of the docked launcher (flush with the screen edge).
pub const PANEL_X: f64 = 0.0;
/// Width of the docked launcher capsule.
pub const PANEL_WIDTH: f64 = 72.0;
/// Convex radius of the outer corners.
pub const PANEL_CORNER_RADIUS: f64 = 24.0;
/// Concave radius of the fillets that blend the capsule into the screen edge.
pub const PANEL_FILLET_RADIUS: f64 = 20.0;
/// Vertical padding above and below the icon slots.
pub const PANEL_PAD: f64 = 24.0;

/// Size of a regular app icon.
pub const ICON: f64 = 30.0;
/// Size of the selected app icon.
pub const ICON_SELECTED: f64 = 42.0;
/// Vertical distance between icon centres.
pub const SLOT: f64 = 50.0;
/// Maximum number of icons visible in the capsule at once.
pub const VISIBLE_SLOTS: usize = 9;

/// Size of the rounded highlight behind the selected icon.
pub const SELECTION_SIZE: f64 = 48.0;
/// Width of the white selection pill on the inner edge.
pub const INDICATOR_WIDTH: f64 = 3.0;
/// Height of the white selection pill on the inner edge.
pub const INDICATOR_HEIGHT: f64 = 16.0;

/// Gap between the capsule and the floating name label.
pub const LABEL_GAP: f64 = 10.0;
/// Height of the floating name label capsule.
pub const LABEL_HEIGHT: f64 = 32.0;
/// Horizontal padding inside the name label.
pub const LABEL_PAD: f64 = 12.0;
/// Font size of the name label.
pub const LABEL_FONT_SIZE: f64 = 13.5;
/// Maximum text width before the label is ellipsized.
pub const LABEL_MAX_TEXT: f64 = 190.0;
/// Maximum total width of the name label capsule.
pub const LABEL_MAX_WIDTH: f64 = LABEL_MAX_TEXT + LABEL_PAD * 2.0;

/// Extra space around the capsule so its soft shadow is not clipped.
pub const SHADOW_PAD: f64 = 30.0;

/// Minimum width of the dark backdrop gradient.
pub const BACKDROP_WIDTH_MIN: f64 = 600.0;
/// Backdrop width as a fraction of the output width.
pub const BACKDROP_WIDTH_RATIO: f64 = 0.60;

// Matches the topbar capsules (`rgb(14, 14, 18)` with a 12% white hairline).
pub const PANEL_SURFACE: (f64, f64, f64, f64) = (14.0 / 255.0, 14.0 / 255.0, 18.0 / 255.0, 0.96);
pub const HAIRLINE: (f64, f64, f64, f64) = (1.0, 1.0, 1.0, 0.12);
