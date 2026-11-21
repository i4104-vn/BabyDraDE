//! Cairo rendering of the shared dock capsule, icons and indicator.

use cairo::Context;
use common::constants::launcher::{
    HAIRLINE, INDICATOR_HEIGHT, INDICATOR_WIDTH, PANEL_CORNER_RADIUS, PANEL_FILLET_RADIUS,
    PANEL_SURFACE, SELECTION_SIZE, SLOT, VISIBLE_SLOTS,
};

use crate::drawing::{
    docked_panel_path, draw_docked_panel, paint_rounded, DockSide as GraphicsSide,
};

use super::{visual_center, DockLayout, DockSide};

/// Draws the shared capsule, selection highlight, items and inner pill.
/// `draw_item(index, centre_x, centre_y, size, alpha)` supplies side-specific
/// icon rendering; the title/preview content is drawn by the caller.
#[allow(clippy::too_many_arguments)]
pub fn draw_dock<F>(
    context: &Context,
    layout: &DockLayout,
    count: usize,
    position: f64,
    highlight_position: f64,
    selected: Option<usize>,
    presence: f64,
    slide_x: f64,
    side: DockSide,
    mut draw_item: F,
) -> Result<(), cairo::Error>
where
    F: FnMut(&Context, usize, f64, f64, f64, f64) -> Result<(), cairo::Error>,
{
    let graphics_side = match side {
        DockSide::Left => GraphicsSide::Left,
        DockSide::Right => GraphicsSide::Right,
    };
    let x = layout.x + slide_x;

    context.push_group();
    let drawn = draw_capsule_content(
        context,
        layout,
        x,
        count,
        position,
        highlight_position,
        selected,
        graphics_side,
        &mut draw_item,
    );
    let group = context.pop_group()?;
    context.set_source(&group)?;
    context.paint_with_alpha(presence)?;
    drawn
}

#[allow(clippy::too_many_arguments)]
fn draw_capsule_content<F>(
    context: &Context,
    layout: &DockLayout,
    x: f64,
    count: usize,
    position: f64,
    highlight_position: f64,
    selected: Option<usize>,
    side: GraphicsSide,
    draw_item: &mut F,
) -> Result<(), cairo::Error>
where
    F: FnMut(&Context, usize, f64, f64, f64, f64) -> Result<(), cairo::Error>,
{
    draw_docked_panel(
        context,
        x,
        layout.y,
        layout.width,
        layout.height,
        PANEL_CORNER_RADIUS,
        PANEL_FILLET_RADIUS,
        PANEL_SURFACE,
        HAIRLINE,
        side,
    )?;

    context.save()?;
    docked_panel_path(
        context,
        x,
        layout.y,
        layout.width,
        layout.height,
        PANEL_CORNER_RADIUS,
        PANEL_FILLET_RADIUS,
        side,
    )?;
    context.clip();

    let visual = visual_center(count, position);
    if selected.is_some() {
        let active_y = active_center_y(layout, count, highlight_position, visual);
        paint_rounded(
            context,
            x + (layout.width - SELECTION_SIZE) / 2.0,
            active_y - SELECTION_SIZE / 2.0,
            SELECTION_SIZE,
            SELECTION_SIZE,
            17.0,
            (1.0, 1.0, 1.0, 0.09),
            None,
        )?;
    }

    draw_items(
        context,
        layout,
        count,
        visual,
        highlight_position,
        draw_item,
    )?;

    if selected.is_some() {
        let active_y = active_center_y(layout, count, highlight_position, visual);
        let pill_x = match side {
            GraphicsSide::Left => x + 8.0,
            GraphicsSide::Right => x + layout.width - 11.0,
        };
        paint_rounded(
            context,
            pill_x,
            active_y - INDICATOR_HEIGHT / 2.0,
            INDICATOR_WIDTH,
            INDICATOR_HEIGHT,
            INDICATOR_WIDTH / 2.0,
            (1.0, 1.0, 1.0, 0.92),
            None,
        )?;
    }
    context.restore()?;
    Ok(())
}

fn wrapped_diff(value: f64, reference: f64, count: usize) -> f64 {
    let n = count as f64;
    let mut diff = (value - reference).rem_euclid(n);
    if diff > n / 2.0 {
        diff -= n;
    }
    diff
}

fn active_center_y(layout: &DockLayout, count: usize, highlight_position: f64, visual: f64) -> f64 {
    let rel = if count <= VISIBLE_SLOTS {
        wrapped_diff(highlight_position, visual, count)
    } else {
        highlight_position - visual
    };
    layout.center_y + rel * SLOT
}

fn draw_items<F>(
    context: &Context,
    layout: &DockLayout,
    count: usize,
    visual: f64,
    highlight_position: f64,
    draw_item: &mut F,
) -> Result<(), cairo::Error>
where
    F: FnMut(&Context, usize, f64, f64, f64, f64) -> Result<(), cairo::Error>,
{
    if count <= VISIBLE_SLOTS {
        for index in 0..count {
            let rel = index as f64 - visual;
            let sel_rel = wrapped_diff(index as f64, highlight_position, count);
            let (cx, cy, size) = layout.icon_geometry(rel, sel_rel);
            draw_item(context, index, cx, cy, size, 1.0)?;
        }
    } else {
        let base = visual.floor() as i64;
        for offset in -4..=4 {
            let unwrapped = base + offset;
            let index = unwrapped.rem_euclid(count as i64) as usize;
            let slot = unwrapped as f64 - visual;
            let sel_rel = wrapped_diff(unwrapped as f64, highlight_position, count);
            let (cx, cy, size) = layout.icon_geometry(slot, sel_rel);
            draw_item(context, index, cx, cy, size, 1.0)?;
        }
    }
    Ok(())
}
