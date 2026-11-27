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
//! Topbar/status anchors are special: opening upwards would cover the icon
//! itself, so they use [`FitPolicy::KeepSide`] and are clamped/shrunk instead
//! of flipped. All geometry functions are pure and can be unit tested without
//! a live display; the GDK helpers degrade gracefully when no display exists.

use gtk4::gdk::Rectangle as GdkRectangle;
use gtk4::prelude::*;
use gtk4::{Align, Orientation, Popover, PositionType, Widget};

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
/// old code subtracted the anchor's width/height here for Bottom/Right, which
/// pulled the popup back over the anchor icon. The main-axis correction is
/// only the 1px size of the pointing rect, never the anchor's size.
///
/// `origin` is the parent widget's origin in screen/toplevel coordinates.
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

/// Mirrors GTK's placement of a popover around a pointing rectangle. Pure so
/// tests can verify that [`pointing_rect_for`] reproduces the fitted rect.
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

/// Applies a pointing rectangle computed in the parent widget's coordinates.
fn apply_pointing(
    popover: &Popover,
    position: PositionType,
    cross: CrossAlign,
    pointing: ScreenRect,
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

    popover.set_pointing_to(Some(&GdkRectangle::new(
        pointing.x,
        pointing.y,
        pointing.width.max(1),
        pointing.height.max(1),
    )));
}

/// Applies a computed rectangle by converting it into a GTK pointing rectangle.
fn apply_fit(
    anchor: &Widget,
    popover: &Popover,
    position: PositionType,
    rect: ScreenRect,
    cross: CrossAlign,
) {
    let pointing = pointing_rect_for(rect, position, cross, widget_origin(anchor));
    apply_pointing(popover, position, cross, pointing);
}

/// Fits `popover` into the monitor containing its parent/anchor widget.
///
/// Call this before `popover.popup()` (after setting its child, so it can be
/// measured). The popover's position, cross-axis alignment, offset and pointing
/// rectangle are updated so GTK anchors it at the computed, on-screen
/// rectangle. Returns `None` when no display/monitor is available, leaving the
/// popover untouched for backwards compatibility.
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

    let (position, rect) = fit_rect_with_policy(
        widget_rect(&anchor),
        preferred,
        popover_size(popover),
        bounds,
        margin,
        gap,
        cross,
        policy,
    );

    apply_fit(&anchor, popover, position, rect, cross);
    Some(PopoverFit { position, rect })
}

/// Fits `popover` around an explicit pointing rectangle expressed in the
/// parent widget's coordinate space (e.g. a right-click at pointer position).
/// Uses the same corrected pointing math as widget-anchored popovers.
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

    let (position, rect) = fit_rect_with_policy(
        anchor,
        preferred,
        popover_size(popover),
        bounds,
        margin,
        gap,
        cross,
        policy,
    );

    apply_pointing(
        popover,
        position,
        cross,
        pointing_rect_for(rect, position, cross, (origin_x, origin_y)),
    );
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

    /// Pure simulation: fit -> pointing rect -> GTK placement -> compare.
    fn placed_rect(
        anchor: ScreenRect,
        preferred: PositionType,
        size: (i32, i32),
        policy: FitPolicy,
        cross: CrossAlign,
        origin: (i32, i32),
    ) -> (PositionType, ScreenRect, ScreenRect) {
        let (position, rect) = fit_rect_with_policy(
            anchor, preferred, size, bounds(), EDGE_MARGIN, ANCHOR_GAP, cross, policy,
        );
        let pointing = pointing_rect_for(rect, position, cross, origin);
        // Use the fitted size: GTK's content is capped to the fitted rect by
        // the caller (ScrolledWindow / size request), not the original request.
        let mut placed =
            popover_rect_from_pointing(pointing, position, cross, (rect.width, rect.height));
        placed.x += origin.0;
        placed.y += origin.1;
        (position, rect, placed)
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
    fn popup_near_bottom_flips_to_top_without_touching_anchor() {
        let anchor = ScreenRect::new(960, 1060, 24, 28);
        let (position, _rect, placed) = placed_rect(
            anchor,
            PositionType::Bottom,
            (200, 300),
            FitPolicy::Flip,
            CrossAlign::Start,
            (960, 1060),
        );
        assert_eq!(position, PositionType::Top);
        assert_eq!(placed.bottom(), anchor.y - ANCHOR_GAP);
        assert!(placed.y >= EDGE_MARGIN);
        assert!(!placed.intersects(&anchor));
    }

    #[test]
    fn popup_near_right_flips_to_left_without_touching_anchor() {
        let anchor = ScreenRect::new(1890, 120, 20, 28);
        let (position, _rect, placed) = placed_rect(
            anchor,
            PositionType::Right,
            (240, 100),
            FitPolicy::Flip,
            CrossAlign::Start,
            (1890, 120),
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
    fn pointing_roundtrip_reproduces_fitted_rect_for_every_side() {
        let wanted = ScreenRect::new(1510, 60, 240, 200);
        let origin = (1400, 40);
        for position in [
            PositionType::Bottom,
            PositionType::Top,
            PositionType::Left,
            PositionType::Right,
        ] {
            for cross in [CrossAlign::Start, CrossAlign::End] {
                let pointing = pointing_rect_for(wanted, position, cross, origin);
                let mut placed = popover_rect_from_pointing(pointing, position, cross, (240, 200));
                placed.x += origin.0;
                placed.y += origin.1;
                assert_eq!(placed, wanted, "{position:?} / {cross:?}");
            }
        }
    }

    #[test]
    fn bottom_pointing_does_not_subtract_anchor_height() {
        // Regression: pointing.y used to be rect.y - anchor.height, which made
        // GTK place the popover on top of the icon.
        let anchor = ScreenRect::new(1500, 8, 24, 28);
        let rect = ScreenRect::new(1500, anchor.bottom() + ANCHOR_GAP, 240, 200);
        let pointing = pointing_rect_for(rect, PositionType::Bottom, CrossAlign::Start, (0, 0));
        assert_eq!(pointing.y, rect.y - 1);
        let placed = popover_rect_from_pointing(pointing, PositionType::Bottom, CrossAlign::Start, (240, 200));
        assert_eq!(placed, rect);
        assert_eq!(placed.y, anchor.bottom() + ANCHOR_GAP);
        assert!(placed.y >= anchor.bottom() + 10);
        assert!(!placed.intersects(&anchor));
    }

    #[test]
    fn topbar_tooltip_stays_below_anchor_with_minimum_gap() {
        assert!(ANCHOR_GAP >= 10 && ANCHOR_GAP <= 16);
        let anchor = ScreenRect::new(1500, 6, 28, 28);
        let (position, rect, placed) = placed_rect(
            anchor,
            PositionType::Bottom,
            (260, 300),
            FitPolicy::KeepSide,
            CrossAlign::Start,
            (0, 0),
        );
        assert_eq!(position, PositionType::Bottom);
        assert_eq!(rect, placed);
        assert!(rect.y >= anchor.bottom() + ANCHOR_GAP);
        assert!(rect.y >= anchor.bottom() + 10);
        assert!(!rect.intersects(&anchor));
        assert!(rect.right() <= bounds().right() - EDGE_MARGIN);
        assert!(rect.bottom() <= bounds().bottom() - EDGE_MARGIN);
    }

    #[test]
    fn topbar_tooltip_near_right_edge_clamps_instead_of_flipping() {
        let anchor = ScreenRect::new(1896, 8, 20, 28);
        let (position, rect, placed) = placed_rect(
            anchor,
            PositionType::Bottom,
            (260, 300),
            FitPolicy::KeepSide,
            CrossAlign::Start,
            (0, 0),
        );
        assert_eq!(position, PositionType::Bottom);
        assert_eq!(rect, placed);
        assert_eq!(rect.right(), bounds().right() - EDGE_MARGIN);
        assert!(rect.y >= anchor.bottom() + 10);
        assert!(!rect.intersects(&anchor));
    }

    #[test]
    fn keep_side_shrinks_near_bottom_without_covering_anchor() {
        let anchor = ScreenRect::new(960, 1000, 24, 28);
        let (position, rect, placed) = placed_rect(
            anchor,
            PositionType::Bottom,
            (200, 300),
            FitPolicy::KeepSide,
            CrossAlign::Start,
            (0, 0),
        );
        assert_eq!(position, PositionType::Bottom);
        assert_eq!(rect, placed);
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
        let (position, rect, placed) = placed_rect(
            anchor,
            PositionType::Right,
            (240, 180),
            FitPolicy::Flip,
            CrossAlign::End,
            (0, 0),
        );
        assert_eq!(position, PositionType::Left);
        assert_eq!(rect, placed);
        assert_eq!(rect.right(), anchor.x - ANCHOR_GAP);
        assert_eq!(rect.bottom(), anchor.bottom());
        assert!(!rect.intersects(&anchor));
    }
}
