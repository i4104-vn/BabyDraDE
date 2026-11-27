//! Screen-edge fitting helpers for overlays.
//!
//! GTK places a `Popover` relative to its anchor widget and relies on the
//! compositor to keep the popup inside the *parent window*. Panels built with
//! layer shell are short, screen-anchored windows, so a popover anchored in
//! them can still end up outside the visible monitor. The helpers here use
//! the monitor geometry reported by GDK to compute a rectangle that fits,
//! optionally flip the popover to the opposite side, and translate the result
//! back into the pointing rectangle understood by GTK.
//!
//! The geometry functions are pure and can be unit tested without a live
//! display. The GDK helpers degrade gracefully when no display is available.

use gtk4::gdk::Rectangle as GdkRectangle;
use gtk4::prelude::*;
use gtk4::{Align, Orientation, Popover, PositionType, Widget};

/// Default space kept between a popover and each monitor edge.
pub const EDGE_MARGIN: i32 = 8;
/// Default gap between an anchor and the popover it opens.
pub const ANCHOR_GAP: i32 = 4;

/// Which end of the anchor's cross axis a popover is aligned to.
///
/// A bottom/top popover uses the horizontal axis as its cross axis, while a
/// left/right popover uses the vertical axis. `Start` keeps the popover's
/// leading edge on the anchor's leading edge; `End` keeps the trailing edges
/// together (useful for popovers that should grow upwards or leftwards).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CrossAlign {
    #[default]
    Start,
    End,
}

/// A simple axis-aligned rectangle in screen/toplevel coordinates.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ScreenRect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl ScreenRect {
    /// Creates a rectangle from its origin and size.
    pub fn new(x: i32, y: i32, width: i32, height: i32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// Converts a GDK rectangle into a `ScreenRect`.
    pub fn from_gdk(rect: GdkRectangle) -> Self {
        Self::new(rect.x(), rect.y(), rect.width(), rect.height())
    }

    /// Right edge, saturating at the integer limit for empty/odd rectangles.
    pub fn right(&self) -> i32 {
        self.x.saturating_add(self.width.max(0))
    }

    /// Bottom edge, saturating at the integer limit for empty/odd rectangles.
    pub fn bottom(&self) -> i32 {
        self.y.saturating_add(self.height.max(0))
    }

    /// Returns a rectangle shifted inwards by `amount` on every side.
    pub fn inset(&self, amount: i32) -> Self {
        let amount = amount.max(0);
        Self::new(
            self.x.saturating_add(amount),
            self.y.saturating_add(amount),
            (self.width - 2 * amount).max(0),
            (self.height - 2 * amount).max(0),
        )
    }

    /// Whether the two rectangles overlap.
    pub fn intersects(&self, other: &Self) -> bool {
        self.x < other.right()
            && other.x < self.right()
            && self.y < other.bottom()
            && other.y < self.bottom()
    }
}

/// Returns the position opposite to `position` (Bottom <-> Top, Left <-> Right).
pub fn flip_position(position: PositionType) -> PositionType {
    match position {
        PositionType::Bottom => PositionType::Top,
        PositionType::Top => PositionType::Bottom,
        PositionType::Left => PositionType::Right,
        PositionType::Right => PositionType::Left,
        _ => PositionType::Bottom,
    }
}

/// Computes the raw popover rectangle for `position` around `anchor`, without
/// any clamping or flipping. `size` is `(width, height)`.
pub fn rect_for_position(
    anchor: ScreenRect,
    position: PositionType,
    size: (i32, i32),
    gap: i32,
    cross: CrossAlign,
) -> ScreenRect {
    let gap = gap.max(0);
    let width = size.0.max(0);
    let height = size.1.max(0);

    let cross_x = match cross {
        CrossAlign::Start => anchor.x,
        CrossAlign::End => anchor.right() - width,
    };
    let cross_y = match cross {
        CrossAlign::Start => anchor.y,
        CrossAlign::End => anchor.bottom() - height,
    };

    match position {
        PositionType::Bottom => ScreenRect::new(cross_x, anchor.bottom() + gap, width, height),
        PositionType::Top => ScreenRect::new(cross_x, anchor.y - gap - height, width, height),
        PositionType::Right => ScreenRect::new(anchor.right() + gap, cross_y, width, height),
        PositionType::Left => ScreenRect::new(anchor.x - gap - width, cross_y, width, height),
        _ => ScreenRect::new(cross_x, anchor.bottom() + gap, width, height),
    }
}

/// Clamps `rect` inside `bounds`, inset by `margin`. Oversized rectangles are
/// shrunk so that both edges stay reachable.
pub fn clamp_to_bounds(rect: ScreenRect, bounds: ScreenRect, margin: i32) -> ScreenRect {
    let area = bounds.inset(margin);
    let width = rect.width.clamp(0, area.width);
    let height = rect.height.clamp(0, area.height);
    let max_x = area.right().saturating_sub(width).max(area.x);
    let max_y = area.bottom().saturating_sub(height).max(area.y);

    ScreenRect::new(
        rect.x.clamp(area.x, max_x),
        rect.y.clamp(area.y, max_y),
        width,
        height,
    )
}

/// Returns whether a rectangle fits inside `bounds` along the axis relevant to
/// `position` (vertical axis for Top/Bottom, horizontal axis for Left/Right).
fn fits_axis(rect: ScreenRect, bounds: ScreenRect, position: PositionType) -> bool {
    match position {
        PositionType::Left | PositionType::Right => {
            rect.x >= bounds.x && rect.right() <= bounds.right()
        }
        _ => rect.y >= bounds.y && rect.bottom() <= bounds.bottom(),
    }
}

/// Computes the final popover position and rectangle.
///
/// If the popover does not fit on the preferred side of `anchor`, the opposite
/// side is tried. Otherwise the rectangle is clamped inside `bounds` (inset by
/// `margin`), which shifts it back on screen near an edge and caps its size.
pub fn fit_rect(
    anchor: ScreenRect,
    preferred: PositionType,
    size: (i32, i32),
    bounds: ScreenRect,
    margin: i32,
    gap: i32,
    cross: CrossAlign,
) -> (PositionType, ScreenRect) {
    let area = bounds.inset(margin);
    let mut position = preferred;
    let mut rect = rect_for_position(anchor, position, size, gap, cross);

    if !fits_axis(rect, area, position) {
        let flipped = flip_position(position);
        let flipped_rect = rect_for_position(anchor, flipped, size, gap, cross);
        if fits_axis(flipped_rect, area, flipped) {
            position = flipped;
            rect = flipped_rect;
        }
    }

    (position, clamp_to_bounds(rect, bounds, margin))
}

/// Result of fitting a real `Popover` on screen.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PopoverFit {
    /// Position actually applied (may be flipped).
    pub position: PositionType,
    /// Desired rectangle in monitor/toplevel coordinates.
    pub rect: ScreenRect,
}

/// Returns the GDK monitor rectangle containing `widget`, in screen coordinates.
///
/// GDK4 does not expose a workarea API; the monitor geometry is the conservative
/// bound available to toolkit clients (shells reserve space via exclusive
/// zones). Popups are positioned in the toplevel's coordinate space, which for
/// full-screen / edge-anchored layer surfaces coincides with the monitor up to
/// the panel margin; the edge margin absorbs that small difference.
pub fn monitor_rect_for(widget: &Widget) -> Option<ScreenRect> {
    let root = widget.root()?;
    let surface = root.surface()?;
    let monitor = gtk4::prelude::WidgetExt::display(&root).monitor_at_surface(&surface)?;
    Some(ScreenRect::from_gdk(monitor.geometry()))
}

/// Returns the allocated bounds of `widget` in its toplevel's coordinate space.
pub fn widget_rect(widget: &Widget) -> ScreenRect {
    let (x, y) = widget
        .root()
        .and_then(|root| widget.translate_coordinates(&root, 0.0, 0.0))
        .unwrap_or((0.0, 0.0));
    ScreenRect::new(
        x.round() as i32,
        y.round() as i32,
        widget.width(),
        widget.height(),
    )
}

/// Largest popover width that fits on `widget`'s monitor with `margin` on both
/// sides. Returns `None` when no display is available.
pub fn max_popover_width(widget: &Widget, margin: i32) -> Option<i32> {
    monitor_rect_for(widget).map(|rect| (rect.width - 2 * margin.max(0)).max(1))
}

/// Largest popover height that fits on `widget`'s monitor with `margin` on both
/// sides. Returns `None` when no display is available.
pub fn max_popover_height(widget: &Widget, margin: i32) -> Option<i32> {
    monitor_rect_for(widget).map(|rect| (rect.height - 2 * margin.max(0)).max(1))
}

/// Fits `popover` into the monitor containing its parent/anchor widget.
///
/// Call this before `popover.popup()` (after setting its child, so it can be
/// measured). The popover's position, cross-axis alignment, offset and pointing
/// rectangle are updated so GTK anchors it at the computed, on-screen
/// rectangle. Returns `None` when no display/monitor is available, leaving the
/// popover untouched for backwards compatibility.
pub fn fit_popover(
    popover: &Popover,
    preferred: PositionType,
    cross: CrossAlign,
    margin: i32,
    gap: i32,
) -> Option<PopoverFit> {
    let anchor = popover.parent()?;
    let bounds = monitor_rect_for(&anchor)?;

    let anchor_rect = widget_rect(&anchor);
    let (min_width, natural_width, _, _) = popover.measure(Orientation::Horizontal, -1);
    let (min_height, natural_height, _, _) =
        popover.measure(Orientation::Vertical, natural_width);
    let size = (natural_width.max(min_width), natural_height.max(min_height));

    let (position, rect) = fit_rect(anchor_rect, preferred, size, bounds, margin, gap, cross);

    apply_fit(&anchor, popover, anchor_rect, position, rect, cross);
    Some(PopoverFit { position, rect })
}

/// Convenience wrapper using the shared edge margin and anchor gap.
pub fn fit_popover_default(popover: &Popover, preferred: PositionType) -> Option<PopoverFit> {
    fit_popover(
        popover,
        preferred,
        CrossAlign::Start,
        EDGE_MARGIN,
        ANCHOR_GAP,
    )
}

/// Fits `popover` into its monitor and presents it.
///
/// This is the drop-in replacement for call sites that previously just called
/// `popover.popup()`; fitting is skipped when no display is available.
pub fn popup_fit(popover: &Popover, preferred: PositionType) {
    fit_popover_default(popover, preferred);
    popover.popup();
}

/// Applies a computed rectangle by converting it into a GTK pointing rectangle.
fn apply_fit(
    anchor: &Widget,
    popover: &Popover,
    anchor_rect: ScreenRect,
    position: PositionType,
    rect: ScreenRect,
    cross: CrossAlign,
) {
    popover.set_position(position);
    let align = match cross {
        CrossAlign::Start => Align::Start,
        CrossAlign::End => Align::End,
    };
    match position {
        PositionType::Bottom | PositionType::Top => popover.set_halign(align),
        PositionType::Left | PositionType::Right => popover.set_valign(align),
        _ => {}
    }
    // The gap between anchor and popover is already part of `rect`, so GTK
    // must not add another one.
    popover.set_offset(0, 0);

    let cross_x = match cross {
        CrossAlign::Start => rect.x,
        CrossAlign::End => rect.right() - anchor_rect.width,
    };
    let cross_y = match cross {
        CrossAlign::Start => rect.y,
        CrossAlign::End => rect.bottom() - anchor_rect.height,
    };

    // In GTK's layout the popover edge meets the pointing rectangle's edge:
    // its top touches the rect bottom for Bottom, its bottom touches the rect
    // top for Top, etc. Invert that mapping to get the pointing rectangle.
    let (point_x, point_y) = match position {
        PositionType::Bottom => (cross_x, rect.y - anchor_rect.height),
        PositionType::Top => (cross_x, rect.bottom()),
        PositionType::Right => (rect.x - anchor_rect.width, cross_y),
        PositionType::Left => (rect.right(), cross_y),
        _ => (cross_x, rect.bottom()),
    };

    let (offset_x, offset_y) = anchor
        .root()
        .and_then(|root| anchor.translate_coordinates(&root, 0.0, 0.0))
        .unwrap_or((0.0, 0.0));
    let pointing = GdkRectangle::new(
        point_x - offset_x.round() as i32,
        point_y - offset_y.round() as i32,
        1,
        1,
    );
    popover.set_pointing_to(Some(&pointing));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bounds() -> ScreenRect {
        ScreenRect::new(0, 0, 1920, 1080)
    }

    #[test]
    fn clamp_shifts_left_overflow_right() {
        let rect = ScreenRect::new(-60, 20, 200, 300);
        let clamped = clamp_to_bounds(rect, bounds(), EDGE_MARGIN);
        assert_eq!(clamped.x, EDGE_MARGIN);
        assert_eq!(clamped.right(), EDGE_MARGIN + 200);
        assert_eq!(clamped.y, 20);
    }

    #[test]
    fn clamp_shifts_right_overflow_left_and_shrinks() {
        let rect = ScreenRect::new(1800, 20, 240, 300);
        let clamped = clamp_to_bounds(rect, bounds(), EDGE_MARGIN);
        assert_eq!(clamped.right(), 1920 - EDGE_MARGIN);
        assert_eq!(clamped.width, 240);

        let too_wide = ScreenRect::new(0, 0, 4000, 4000);
        let capped = clamp_to_bounds(too_wide, bounds(), 8);
        assert_eq!(capped.width, 1920 - 16);
        assert_eq!(capped.height, 1080 - 16);
        assert_eq!(capped.x, 8);
        assert_eq!(capped.y, 8);
    }

    #[test]
    fn flip_position_is_symmetric() {
        assert_eq!(flip_position(PositionType::Bottom), PositionType::Top);
        assert_eq!(flip_position(PositionType::Top), PositionType::Bottom);
        assert_eq!(flip_position(PositionType::Right), PositionType::Left);
        assert_eq!(flip_position(PositionType::Left), PositionType::Right);
    }

    #[test]
    fn bottom_menu_near_left_edge_is_shifted_inward() {
        // Anchor hugging the left monitor edge, like the panel logo.
        let anchor = ScreenRect::new(0, 8, 24, 28);
        let (position, rect) = fit_rect(
            anchor,
            PositionType::Bottom,
            (200, 320),
            bounds(),
            EDGE_MARGIN,
            ANCHOR_GAP,
            CrossAlign::Start,
        );
        assert_eq!(position, PositionType::Bottom);
        assert_eq!(rect.x, EDGE_MARGIN);
        assert_eq!(rect.y, anchor.bottom() + ANCHOR_GAP);
        assert!(rect.right() <= 1920 - EDGE_MARGIN);
        assert!(rect.bottom() <= 1080 - EDGE_MARGIN);
    }

    #[test]
    fn popup_near_bottom_flips_to_top() {
        let anchor = ScreenRect::new(960, 1060, 24, 28);
        let (position, rect) = fit_rect(
            anchor,
            PositionType::Bottom,
            (200, 300),
            bounds(),
            EDGE_MARGIN,
            ANCHOR_GAP,
            CrossAlign::Start,
        );
        assert_eq!(position, PositionType::Top);
        assert_eq!(rect.bottom(), anchor.y - ANCHOR_GAP);
        assert!(rect.y >= EDGE_MARGIN);
    }

    #[test]
    fn popup_near_right_flips_to_left() {
        let anchor = ScreenRect::new(1890, 120, 20, 28);
        let (position, rect) = fit_rect(
            anchor,
            PositionType::Right,
            (240, 100),
            bounds(),
            EDGE_MARGIN,
            ANCHOR_GAP,
            CrossAlign::Start,
        );
        assert_eq!(position, PositionType::Left);
        assert_eq!(rect.right(), anchor.x - ANCHOR_GAP);
        assert!(rect.x >= EDGE_MARGIN);
    }

    #[test]
    fn end_cross_alignment_keeps_trailing_edges_together() {
        let anchor = ScreenRect::new(1890, 120, 20, 28);
        let rect = rect_for_position(
            anchor,
            PositionType::Left,
            (240, 100),
            ANCHOR_GAP,
            CrossAlign::End,
        );
        assert_eq!(rect.right(), anchor.x - ANCHOR_GAP);
        assert_eq!(rect.bottom(), anchor.bottom());

        let (position, fitted) = fit_rect(
            anchor,
            PositionType::Right,
            (240, 100),
            bounds(),
            EDGE_MARGIN,
            ANCHOR_GAP,
            CrossAlign::End,
        );
        assert_eq!(position, PositionType::Left);
        assert_eq!(fitted.bottom(), anchor.bottom());
        assert!(fitted.x >= EDGE_MARGIN);
    }

    #[test]
    fn small_bounds_still_produce_non_negative_rectangles() {
        let tiny = ScreenRect::new(0, 0, 4, 4);
        let clamped = clamp_to_bounds(ScreenRect::new(-50, -50, 200, 200), tiny, 8);
        assert_eq!(clamped.x, 8);
        assert_eq!(clamped.y, 8);
        assert_eq!(clamped.width, 0);
        assert_eq!(clamped.height, 0);
    }

    #[test]
    fn intersects_uses_half_open_intervals() {
        let a = ScreenRect::new(0, 0, 10, 10);
        let b = ScreenRect::new(10, 0, 10, 10);
        assert!(!a.intersects(&b));
        assert!(a.intersects(&ScreenRect::new(9, 9, 5, 5)));
    }
}
