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

// ---------------------------------------------------------------------------
// Search overlay (the launcher expands into this as soon as text is typed).
// ---------------------------------------------------------------------------

/// Preferred width of the centered search column in logical pixels.
pub const SEARCH_PANEL_WIDTH: f64 = 640.0;
/// Hard ceiling for the column height; also clamped to the output height.
pub const SEARCH_PANEL_MAX_HEIGHT: f64 = 560.0;
/// Extra room around the column so fly-in offsets and text are not clipped.
pub const SEARCH_BUFFER_PAD: f64 = 36.0;
/// Minimum gap kept between the column and the screen edges.
pub const SEARCH_SCREEN_MARGIN: f64 = 32.0;

/// Horizontal padding inside the search column.
pub const SEARCH_PAD_X: f64 = 20.0;
/// Vertical padding inside the search column.
pub const SEARCH_PAD_Y: f64 = 18.0;
/// Height of the query input row.
pub const SEARCH_INPUT_HEIGHT: f64 = 62.0;
/// Glyph size of the query and result text.
pub const SEARCH_FONT_SIZE: f64 = 16.0;
/// Glyph size of the category headings.
pub const SEARCH_HEADER_FONT_SIZE: f64 = 11.5;
/// Gap between the input divider and the first category heading.
pub const SEARCH_SECTION_GAP: f64 = 18.0;
/// Height of a category heading row.
pub const SEARCH_HEADER_HEIGHT: f64 = 28.0;
/// Height of a single result row.
pub const SEARCH_ROW_HEIGHT: f64 = 46.0;
/// Maximum number of rows listed under *Apps* and *Files*. Web is always one.
pub const SEARCH_ROWS_PER_CATEGORY: usize = 3;
/// Size of the square icon badge drawn on each result row.
pub const SEARCH_ICON_SIZE: f64 = 28.0;
/// Width of the input caret.
pub const SEARCH_CARET_WIDTH: f64 = 2.0;
/// Milliseconds per blink phase of the caret.
pub const SEARCH_CARET_BLINK_MS: u64 = 530;
/// Horizontal breathing room when the query scrolls under a long caret move.
pub const SEARCH_CARET_SCROLL_PAD: f64 = 28.0;
/// Width of the thin accent bar marking the selected result row.
pub const SEARCH_SELECTION_BAR: f64 = 3.0;

/// Opacity of the full-screen dimmer at full presence. Deliberately dark: the
/// search column is drawn flat (no card) and needs the contrast.
pub const SEARCH_BACKDROP_ALPHA: f64 = 0.84;

// --- Staggered reveal -----------------------------------------------------
// These are fractions of one master timeline, so the input always settles
// first and every result element follows it from top to bottom.

/// Fraction of the timeline used to reveal the query input.
pub const SEARCH_INPUT_SPAN: f64 = 0.18;
/// Fraction at which the first result element (heading) starts.
pub const SEARCH_ROW_START: f64 = 0.18;
/// Fraction of the timeline each result element takes to fly in.
pub const SEARCH_ROW_SPAN: f64 = 0.20;
/// Fraction added to the start of each successive result element.
pub const SEARCH_ROW_STEP: f64 = 0.06;
/// Distance each element flies upwards from its resting position.
pub const SEARCH_ELEMENT_FLY_DISTANCE: f64 = 26.0;
