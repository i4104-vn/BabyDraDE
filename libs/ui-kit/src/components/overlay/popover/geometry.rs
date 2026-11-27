//! Screen-edge fitting helpers for overlays.
//!
//! GTK places a `Popover` relative to its parent widget using the widget's
//! real bounds (or the pointing rectangle supplied by a caller) and relies on
//! the compositor to keep the popup inside the *parent window*. Panels built
//! with layer shell are short, screen-anchored windows, so a popover anchored
//! in them can still end up outside the visible monitor. The helpers here use
//! the monitor geometry reported by GDK to compute a rectangle that fits,
//! optionally flip the popover to the opposite side, and shift GTK's native
//! placement onto it with `Popover::set_offset()`.
//!
//! Widget-anchored popovers never set their own pointing rectangle: a rect
//! derived from `widget_origin()` is unreliable when the widget-to-root
//! translation is unavailable (layer-shell roots, unrealized popups) and used
//! to collapse to (0,0), sending the popup to the window's top-left corner.
//! GTK's native placement already puts a popover below its icon; only the
//! screen-edge correction is applied here.
//!
//! Topbar/status anchors are special: opening upwards would cover the icon
//! itself, so they use [`FitPolicy::KeepSide`] and are clamped/shifted instead
//! of flipped. All geometry functions are pure and can be unit tested without
//! a live display; the GDK helpers degrade gracefully when no display exists.

use gtk4::gdk::Rectangle as GdkRectangle;
use gtk4::prelude::*;
use gtk4::{Orientation, Popover, PositionType, Widget};

/// Default space kept between a popover and each monitor edge.
pub const EDGE_MARGIN: i32 = 8;
/// Gap between an anchor and the popover it opens (12px keeps tooltips and
/// menus clear of the icon row while staying compact).
pub const ANCHOR_GAP: i32 = 12;

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

/// How `fit_rect` reacts when the requested side has insufficient room.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FitPolicy {
    /// Try the opposite side when the preferred side lacks room. Used by
    /// generic dialogs/menus where flipping is the only way to stay visible.
    #[default]
    Flip,
    /// Stay on the preferred side; clamp and shrink instead of flipping.
    /// Used for topbar/status anchors so a popover never covers its icon.
    KeepSide,
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

/// Clamps a fitted rectangle to `area` while keeping it on the requested side
/// of `anchor`. Unlike [`clamp_to_bounds`], this never shifts the popover back
/// across the anchor: the cross axis is clamped normally, the main axis is
/// clamped to `[anchor side + gap, area edge]` and the size is shrunk to fit.
/// With no room at all the popover collapses to zero size instead of covering
/// the anchor.
fn clamp_on_side(
    rect: ScreenRect,
    anchor: ScreenRect,
    position: PositionType,
    area: ScreenRect,
    gap: i32,
    cross: CrossAlign,
) -> ScreenRect {
    let gap = gap.max(0);
    match position {
        PositionType::Bottom | PositionType::Top => {
            let width = rect.width.clamp(0, area.width);
            let max_x = area.right().saturating_sub(width).max(area.x);
            let x = match cross {
                CrossAlign::Start => rect.x.clamp(area.x, max_x),
                CrossAlign::End => anchor.right().saturating_sub(width).clamp(area.x, max_x),
            };
            if position == PositionType::Bottom {
                let min_y = anchor.bottom().saturating_add(gap);
                let y = rect.y.max(min_y).min(area.bottom());
                let height = area.bottom().saturating_sub(y).min(rect.height.max(0));
                ScreenRect::new(x, y, width, height)
            } else {
                let max_bottom = anchor.y.saturating_sub(gap);
                let bottom = rect.bottom().min(max_bottom).max(area.y);
                let height = bottom.saturating_sub(area.y).min(rect.height.max(0));
                ScreenRect::new(x, bottom - height, width, height)
            }
        }
        PositionType::Left | PositionType::Right => {
            let height = rect.height.clamp(0, area.height);
            let max_y = area.bottom().saturating_sub(height).max(area.y);
            let y = match cross {
                CrossAlign::Start => rect.y.clamp(area.y, max_y),
                CrossAlign::End => anchor.bottom().saturating_sub(height).clamp(area.y, max_y),
            };
            if position == PositionType::Right {
                let min_x = anchor.right().saturating_add(gap);
                let x = rect.x.max(min_x).min(area.right());
                let width = area.right().saturating_sub(x).min(rect.width.max(0));
                ScreenRect::new(x, y, width, height)
            } else {
                let max_right = anchor.x.saturating_sub(gap);
                let right = rect.right().min(max_right).max(area.x);
                let width = right.saturating_sub(area.x).min(rect.width.max(0));
                ScreenRect::new(right - width, y, width, height)
            }
        }
        _ => clamp_to_bounds(rect, area, 0),
    }
}

/// Computes the final popover position and rectangle.
///
/// If the popover does not fit on the preferred side of `anchor`, the opposite
/// side is tried (unless `policy` is [`FitPolicy::KeepSide`]). Afterwards the
/// rectangle is clamped inside `area` without ever crossing back over the
/// anchor.
pub fn fit_rect_with_policy(
    anchor: ScreenRect,
    preferred: PositionType,
    size: (i32, i32),
    bounds: ScreenRect,
    margin: i32,
    gap: i32,
    cross: CrossAlign,
    policy: FitPolicy,
) -> (PositionType, ScreenRect) {
    let area = bounds.inset(margin);
    let mut position = preferred;
    let mut rect = rect_for_position(anchor, position, size, gap, cross);

    if !fits_axis(rect, area, position) && policy == FitPolicy::Flip {
        let flipped = flip_position(position);
        let flipped_rect = rect_for_position(anchor, flipped, size, gap, cross);
        if fits_axis(flipped_rect, area, flipped) {
            position = flipped;
            rect = flipped_rect;
        }
    }

    (position, clamp_on_side(rect, anchor, position, area, gap, cross))
}

/// Computes the final popover position and rectangle, allowing a flip to the
/// opposite side when the preferred side lacks room.
pub fn fit_rect(
    anchor: ScreenRect,
    preferred: PositionType,
    size: (i32, i32),
    bounds: ScreenRect,
    margin: i32,
    gap: i32,
    cross: CrossAlign,
) -> (PositionType, ScreenRect) {
    fit_rect_with_policy(anchor, preferred, size, bounds, margin, gap, cross, FitPolicy::Flip)
}

/// Returns the GTK pointing rectangle (parent-widget coordinates) that places
/// a 1x1-pointed popover exactly at `rect` (screen/toplevel coordinates).
///
/// GTK anchors a Bottom popover's *top* edge to the pointing rect's bottom
/// edge, a Top popover's *bottom* edge to the pointing rect's top edge, and
/// likewise horizontally; `halign`/`valign` select the cross-axis edge. The
/// main-axis correction is only the 1px size of the pointing rect, never the
/// anchor's size.
///
/// `origin` is the parent widget's origin in screen/toplevel coordinates.
/// Retained for callers that position a popover with an explicit pointing
/// rectangle; the fitters in this module do not use it (see the module docs).
pub fn pointing_rect_for(
    rect: ScreenRect,
    position: PositionType,
    cross: CrossAlign,
    origin: (i32, i32),
) -> ScreenRect {
    let (x, y) = match (position, cross) {
        (PositionType::Bottom, CrossAlign::Start) => (rect.x, rect.y - 1),
        (PositionType::Bottom, CrossAlign::End) => (rect.right() - 1, rect.y - 1),
        (PositionType::Top, CrossAlign::Start) => (rect.x, rect.bottom()),
        (PositionType::Top, CrossAlign::End) => (rect.right() - 1, rect.bottom()),
        (PositionType::Right, CrossAlign::Start) => (rect.x - 1, rect.y),
        (PositionType::Right, CrossAlign::End) => (rect.x - 1, rect.bottom() - 1),
        (PositionType::Left, CrossAlign::Start) => (rect.right(), rect.y),
        (PositionType::Left, CrossAlign::End) => (rect.right(), rect.bottom() - 1),
        _ => (rect.x, rect.y - 1),
    };
    ScreenRect::new(
        x.saturating_sub(origin.0),
        y.saturating_sub(origin.1),
        1,
        1,
    )
}

/// Mirrors GTK's placement of a popover around an explicit pointing rectangle.
/// Kept as the pure counterpart of [`pointing_rect_for`] for callers that
/// manage the pointing rectangle themselves.
pub fn popover_rect_from_pointing(
    pointing: ScreenRect,
    position: PositionType,
    cross: CrossAlign,
    size: (i32, i32),
) -> ScreenRect {
    let width = size.0.max(0);
    let height = size.1.max(0);
    let cross_x = match cross {
        CrossAlign::Start => pointing.x,
        CrossAlign::End => pointing.right() - width,
    };
    let cross_y = match cross {
        CrossAlign::Start => pointing.y,
        CrossAlign::End => pointing.bottom() - height,
    };
    match position {
        PositionType::Bottom => ScreenRect::new(cross_x, pointing.bottom(), width, height),
        PositionType::Top => ScreenRect::new(cross_x, pointing.y - height, width, height),
        PositionType::Right => ScreenRect::new(pointing.right(), cross_y, width, height),
        PositionType::Left => ScreenRect::new(pointing.x - width, cross_y, width, height),
        _ => ScreenRect::new(cross_x, pointing.bottom(), width, height),
    }
}

/// Rectangle GTK places a popover in when no pointing rectangle and no offset
/// are set.
///
/// With GTK's default alignment (`Fill`/`Center` behaves as `Center` here) a
/// popup is centered on the anchor's cross axis and its leading main-axis edge
/// touches the anchor's trailing edge. `size` is `(width, height)` and the
/// result is in the same (toplevel/screen) coordinate space as `anchor`.
///
/// This models GTK's *native* placement; this module only computes the offset
/// that shifts it onto the fitted rectangle. It never writes a pointing rect
/// for widget-anchored popovers.
pub fn native_rect_for(anchor: ScreenRect, position: PositionType, size: (i32, i32)) -> ScreenRect {
    let width = size.0.max(0);
    let height = size.1.max(0);
    let cross_x = anchor.x + anchor.width / 2 - width / 2;
    let cross_y = anchor.y + anchor.height / 2 - height / 2;

    match position {
        PositionType::Bottom => ScreenRect::new(cross_x, anchor.bottom(), width, height),
        PositionType::Top => ScreenRect::new(cross_x, anchor.y - height, width, height),
        PositionType::Right => ScreenRect::new(anchor.right(), cross_y, width, height),
        PositionType::Left => ScreenRect::new(anchor.x - width, cross_y, width, height),
        _ => ScreenRect::new(cross_x, anchor.bottom(), width, height),
    }
}

/// Offset that moves a natively placed popover from `native` onto `target`.
///
/// The values are passed straight to `Popover::set_offset()`; GTK applies them
/// on top of its own placement, leaving the caller's pointing rect untouched.
pub fn offset_for_rect(native: ScreenRect, target: ScreenRect) -> (i32, i32) {
    (
        target.x.saturating_sub(native.x),
        target.y.saturating_sub(native.y),
    )
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

/// Returns the origin of `widget` in its toplevel's coordinate space.
fn widget_origin(widget: &Widget) -> (i32, i32) {
    let (x, y) = widget
        .root()
        .and_then(|root| widget.translate_coordinates(&root, 0.0, 0.0))
        .unwrap_or((0.0, 0.0));
    (x.round() as i32, y.round() as i32)
}

/// Returns the allocated bounds of `widget` in its toplevel's coordinate space.
pub fn widget_rect(widget: &Widget) -> ScreenRect {
    let (x, y) = widget_origin(widget);
    ScreenRect::new(x, y, widget.width(), widget.height())
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

/// Measures the natural size of a popover's content.
fn popover_size(popover: &Popover) -> (i32, i32) {
    let (min_width, natural_width, _, _) = popover.measure(Orientation::Horizontal, -1);
    let (min_height, natural_height, _, _) = popover.measure(Orientation::Vertical, natural_width);
    (natural_width.max(min_width), natural_height.max(min_height))
}

/// Shifts GTK's native placement onto the fitted rectangle.
///
/// Only the position and the offset are written. Setting `halign`/`valign` or
/// a pointing rectangle here would override GTK's own placement, which is
/// already correct relative to the parent widget.
fn apply_offset(popover: &Popover, position: PositionType, native: ScreenRect, target: ScreenRect) {
    popover.set_position(position);
    let (dx, dy) = offset_for_rect(native, target);
    popover.set_offset(dx, dy);
}

/// Applies a computed rectangle by shifting GTK's native placement.
///
/// `set_pointing_to(None)` restores GTK's default behaviour of using the
/// parent widget's allocated bounds; a stale pointing rect (or one rebuilt
/// from `widget_origin()`) can put the popup at the window's top-left corner
/// when the translation to the root is unreliable.
fn apply_fit(
    anchor: &Widget,
    popover: &Popover,
    position: PositionType,
    size: (i32, i32),
    rect: ScreenRect,
) {
    popover.set_pointing_to(None);
    apply_offset(
        popover,
        position,
        native_rect_for(widget_rect(anchor), position, size),
        rect,
    );
}

/// Fits `popover` into the monitor containing its parent/anchor widget.
///
/// Call this before `popover.popup()` (after setting its child, so it can be
/// measured). GTK keeps its native placement relative to the parent widget;
/// only the popover's position and offset are updated so it lands on the
/// computed, on-screen rectangle. Returns `None` when no display/monitor is
/// available, leaving the popover untouched for backwards compatibility.
pub fn fit_popover_with_policy(
    popover: &Popover,
    preferred: PositionType,
    cross: CrossAlign,
    margin: i32,
    gap: i32,
    policy: FitPolicy,
) -> Option<PopoverFit> {
    let anchor = popover.parent()?;
    let bounds = monitor_rect_for(&anchor)?;

    let anchor_rect = widget_rect(&anchor);
    let size = popover_size(popover);
    let (position, rect) = fit_rect_with_policy(
        anchor_rect,
        preferred,
        size,
        bounds,
        margin,
        gap,
        cross,
        policy,
    );

    apply_fit(&anchor, popover, position, size, rect);
    Some(PopoverFit { position, rect })
}

/// Fits `popover` around an explicit pointing rectangle expressed in the
/// parent widget's coordinate space (e.g. a right-click at pointer position).
///
/// The caller's pointing rectangle (set with `set_pointing_to`) is left
/// untouched: GTK already places the popup around it. Just like the
/// widget-anchored path, only an offset is applied to keep the result on
/// screen.
pub fn fit_popover_at_point(
    popover: &Popover,
    point: ScreenRect,
    preferred: PositionType,
    cross: CrossAlign,
    margin: i32,
    gap: i32,
    policy: FitPolicy,
) -> Option<PopoverFit> {
    let parent = popover.parent()?;
    let bounds = monitor_rect_for(&parent)?;
    let (origin_x, origin_y) = widget_origin(&parent);
    let anchor = ScreenRect::new(
        point.x.saturating_add(origin_x),
        point.y.saturating_add(origin_y),
        point.width.max(1),
        point.height.max(1),
    );

    let size = popover_size(popover);
    let (position, rect) =
        fit_rect_with_policy(anchor, preferred, size, bounds, margin, gap, cross, policy);

    // `point` is in parent-widget coordinates; GTK's native placement is
    // measured there too, so convert it to the toplevel space used by `rect`.
    let native = native_rect_for(point, position, size);
    let native = ScreenRect::new(
        native.x.saturating_add(origin_x),
        native.y.saturating_add(origin_y),
        native.width,
        native.height,
    );
    apply_offset(popover, position, native, rect);
    Some(PopoverFit { position, rect })
}

/// Fits `popover` with flipping enabled (generic dialogs/menus).
pub fn fit_popover(
    popover: &Popover,
    preferred: PositionType,
    cross: CrossAlign,
    margin: i32,
    gap: i32,
) -> Option<PopoverFit> {
    fit_popover_with_policy(popover, preferred, cross, margin, gap, FitPolicy::Flip)
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

/// Fits `popover` strictly below its parent anchor, clamping instead of
/// flipping so a topbar/status icon is never covered.
pub fn fit_popover_below(popover: &Popover) -> Option<PopoverFit> {
    fit_popover_with_policy(
        popover,
        PositionType::Bottom,
        CrossAlign::Start,
        EDGE_MARGIN,
        ANCHOR_GAP,
        FitPolicy::KeepSide,
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

/// Presents a tooltip/menu strictly below its anchor (no flip, no overlap).
pub fn popup_fit_below(popover: &Popover) {
    fit_popover_below(popover);
    popover.popup();
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bounds() -> ScreenRect {
        ScreenRect::new(0, 0, 1920, 1080)
    }

    /// Pure simulation: fit -> native placement -> offset -> final rect.
    fn placed_rect(
        anchor: ScreenRect,
        preferred: PositionType,
        size: (i32, i32),
        policy: FitPolicy,
        cross: CrossAlign,
    ) -> (PositionType, ScreenRect, ScreenRect, (i32, i32)) {
        let (position, rect) = fit_rect_with_policy(
            anchor,
            preferred,
            size,
            bounds(),
            EDGE_MARGIN,
            ANCHOR_GAP,
            cross,
            policy,
        );
        // The caller caps the popover content to the fitted size, so simulate
        // GTK's native placement with that size rather than the raw request.
        let native = native_rect_for(anchor, position, (rect.width, rect.height));
        let offset = offset_for_rect(native, rect);
        let mut placed = native;
        placed.x += offset.0;
        placed.y += offset.1;
        (position, rect, placed, offset)
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
    fn native_bottom_placement_is_centered_and_flush() {
        // GTK uses the parent widget's bounds as the pointing rect; with its
        // default alignment the popup is centered and touches the bottom edge.
        let anchor = ScreenRect::new(100, 8, 24, 28);
        let native = native_rect_for(anchor, PositionType::Bottom, (200, 100));
        assert_eq!(native, ScreenRect::new(12, anchor.bottom(), 200, 100));
    }

    #[test]
    fn native_side_placement_is_centered_on_cross_axis() {
        let anchor = ScreenRect::new(1890, 120, 20, 28);
        let right = native_rect_for(anchor, PositionType::Right, (240, 100));
        assert_eq!(right, ScreenRect::new(anchor.right(), 84, 240, 100));
        let left = native_rect_for(anchor, PositionType::Left, (240, 100));
        assert_eq!(left, ScreenRect::new(anchor.x - 240, 84, 240, 100));
    }

    #[test]
    fn popup_near_bottom_flips_to_top_without_touching_anchor() {
        let anchor = ScreenRect::new(960, 1060, 24, 28);
        let (position, _rect, placed, _offset) = placed_rect(
            anchor,
            PositionType::Bottom,
            (200, 300),
            FitPolicy::Flip,
            CrossAlign::Start,
        );
        assert_eq!(position, PositionType::Top);
        assert_eq!(placed.bottom(), anchor.y - ANCHOR_GAP);
        assert!(placed.y >= EDGE_MARGIN);
        assert!(!placed.intersects(&anchor));
    }

    #[test]
    fn popup_near_right_flips_to_left_without_touching_anchor() {
        let anchor = ScreenRect::new(1890, 120, 20, 28);
        let (position, _rect, placed, _offset) = placed_rect(
            anchor,
            PositionType::Right,
            (240, 100),
            FitPolicy::Flip,
            CrossAlign::Start,
        );
        assert_eq!(position, PositionType::Left);
        assert_eq!(placed.right(), anchor.x - ANCHOR_GAP);
        assert!(placed.x >= EDGE_MARGIN);
        assert!(!placed.intersects(&anchor));
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

    #[test]
    fn offset_roundtrip_matches_fitted_rect_for_every_side_and_cross() {
        let anchor = ScreenRect::new(900, 400, 24, 28);
        let size = (240, 200);
        for position in [
            PositionType::Bottom,
            PositionType::Top,
            PositionType::Left,
            PositionType::Right,
        ] {
            for cross in [CrossAlign::Start, CrossAlign::End] {
                let (_position, rect, placed, offset) =
                    placed_rect(anchor, position, size, FitPolicy::Flip, cross);
                let native = native_rect_for(anchor, position, size);
                assert_eq!(
                    offset,
                    offset_for_rect(native, rect),
                    "{position:?} / {cross:?}"
                );
                assert_eq!(placed, rect, "{position:?} / {cross:?}");
                assert!(!placed.intersects(&anchor), "{position:?} / {cross:?}");
            }
        }
    }

    #[test]
    fn bottom_offset_keeps_gap_without_covering_anchor() {
        // Regression: the old fitter rebuilt a pointing rect from
        // widget_origin() and the popup jumped to the window's top-left. The
        // gap now comes from the y offset on top of GTK's flush placement.
        let anchor = ScreenRect::new(1500, 8, 24, 28);
        let (position, rect, placed, offset) = placed_rect(
            anchor,
            PositionType::Bottom,
            (240, 200),
            FitPolicy::KeepSide,
            CrossAlign::Start,
        );
        assert_eq!(position, PositionType::Bottom);
        assert_eq!(offset.1, ANCHOR_GAP);
        assert_eq!(rect, placed);
        assert_eq!(placed.y, anchor.bottom() + ANCHOR_GAP);
        assert!(placed.y >= anchor.bottom() + 10);
        assert!(!placed.intersects(&anchor));
    }

    #[test]
    fn pointer_point_offset_keeps_callers_pointing_rect() {
        // Callers set a 1x1 pointing rect themselves. GTK centers the popup on
        // it natively; only the offset moves it to the fitted rectangle.
        let point = ScreenRect::new(400, 300, 1, 1);
        let native = native_rect_for(point, PositionType::Bottom, (200, 100));
        assert_eq!(native, ScreenRect::new(300, 301, 200, 100));
        let target = ScreenRect::new(400, 313, 200, 100);
        assert_eq!(offset_for_rect(native, target), (100, 12));
    }

    #[test]
    fn topbar_tooltip_stays_below_anchor_with_minimum_gap() {
        assert!(ANCHOR_GAP >= 10 && ANCHOR_GAP <= 16);
        let anchor = ScreenRect::new(1500, 6, 28, 28);
        let (position, rect, placed, offset) = placed_rect(
            anchor,
            PositionType::Bottom,
            (260, 300),
            FitPolicy::KeepSide,
            CrossAlign::Start,
        );
        assert_eq!(position, PositionType::Bottom);
        assert_eq!(rect, placed);
        assert_eq!(offset.1, ANCHOR_GAP);
        assert!(rect.y >= anchor.bottom() + ANCHOR_GAP);
        assert!(rect.y >= anchor.bottom() + 10);
        assert!(!rect.intersects(&anchor));
        assert!(rect.right() <= bounds().right() - EDGE_MARGIN);
        assert!(rect.bottom() <= bounds().bottom() - EDGE_MARGIN);
    }

    #[test]
    fn topbar_tooltip_near_right_edge_clamps_instead_of_flipping() {
        let anchor = ScreenRect::new(1896, 8, 20, 28);
        let (position, rect, placed, offset) = placed_rect(
            anchor,
            PositionType::Bottom,
            (260, 300),
            FitPolicy::KeepSide,
            CrossAlign::Start,
        );
        assert_eq!(position, PositionType::Bottom);
        assert_eq!(rect, placed);
        // The right-edge correction is a purely horizontal shift.
        assert!(offset.0 < 0);
        assert_eq!(offset.1, ANCHOR_GAP);
        assert_eq!(placed.right(), bounds().right() - EDGE_MARGIN);
        assert!(rect.y >= anchor.bottom() + 10);
        assert!(!rect.intersects(&anchor));
    }

    #[test]
    fn keep_side_shrinks_near_bottom_without_covering_anchor() {
        let anchor = ScreenRect::new(960, 1000, 24, 28);
        let (position, rect, placed, offset) = placed_rect(
            anchor,
            PositionType::Bottom,
            (200, 300),
            FitPolicy::KeepSide,
            CrossAlign::Start,
        );
        assert_eq!(position, PositionType::Bottom);
        assert_eq!(rect, placed);
        assert_eq!(offset.1, ANCHOR_GAP);
        assert_eq!(rect.y, anchor.bottom() + ANCHOR_GAP);
        assert!(rect.height < 300);
        assert_eq!(rect.bottom(), bounds().bottom() - EDGE_MARGIN);
        assert!(!rect.intersects(&anchor));
    }

    #[test]
    fn end_aligned_side_popover_never_covers_its_button() {
        // Audio-device button on the right edge of the panel: the fitter flips
        // it to the left and End alignment keeps its bottom on the button.
        let anchor = ScreenRect::new(1896, 700, 24, 28);
        let (position, rect, placed, _offset) = placed_rect(
            anchor,
            PositionType::Right,
            (240, 180),
            FitPolicy::Flip,
            CrossAlign::End,
        );
        assert_eq!(position, PositionType::Left);
        assert_eq!(rect, placed);
        assert_eq!(rect.right(), anchor.x - ANCHOR_GAP);
        assert_eq!(rect.bottom(), anchor.bottom());
        assert!(!rect.intersects(&anchor));
    }
}
