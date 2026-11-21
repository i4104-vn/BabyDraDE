//! Right-side Alt+Tab switcher: a docked capsule styled like the launcher,
//! with one small live preview beside the active icon and its title at the
//! bottom of that preview. The old centre stage is gone.

use cairo::{Context, Operator};
use common::constants::launcher::{
    HAIRLINE, ICON, ICON_SELECTED, PANEL_CORNER_RADIUS, PANEL_FILLET_RADIUS, PANEL_PAD,
    PANEL_SURFACE, PANEL_WIDTH, SELECTION_SIZE, SLOT, VISIBLE_SLOTS,
};
use services::graphics::{
    docked_panel_path, draw_docked_panel, draw_text, fit_text, paint_rounded, set_bold_font,
    DockSide,
};

use crate::compositor::AltTabSwitcher;

use super::draw::{draw_app_badge, Rect};

const PREVIEW_W: f64 = 300.0;
const PREVIEW_H: f64 = 176.0;
const PREVIEW_GAP: f64 = 14.0;
const TITLE_H: f64 = 34.0;
const TITLE_FONT: f64 = 13.5;
const TITLE_PAD: f64 = 12.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct CardLayout {
    pub(crate) card: Rect,
    pub(crate) preview: Rect,
    pub(crate) radius: f64,
    pub(crate) opacity: f64,
}

pub(crate) struct SwitcherLayout {
    /// Right-edge capsule body.
    pub(crate) panel: Rect,
    /// Icon slots inside the capsule.
    pub(crate) dock_items: Vec<(usize, CardLayout)>,
    /// Highlight behind the active icon.
    pub(crate) active: Rect,
    /// Small live preview of the active window.
    pub(crate) preview: Rect,
    /// Fade of the preview while switching.
    pub(crate) preview_alpha: f64,
}

/// Slot at the capsule centre. Short lists stay still; long lists scroll but
/// stop before leaving an empty rounded end.
fn visual_center(count: usize, position: f64) -> f64 {
    let count = count as f64;
    if count <= VISIBLE_SLOTS as f64 {
        (count - 1.0) / 2.0
    } else {
        let half = (VISIBLE_SLOTS / 2) as f64;
        position.clamp(half, count - 1.0 - half)
    }
}

pub(crate) fn compute_layout(
    switcher: &AltTabSwitcher,
    screen_w: f64,
    screen_h: f64,
) -> Option<SwitcherLayout> {
    if switcher.entries.is_empty() {
        return None;
    }
    Some(compute_layout_metrics(
        switcher.entries.len(),
        switcher.visual_selection(),
        switcher.stage_progress(),
        screen_w,
        screen_h,
    ))
}

/// Pure geometry, independent of `AltTabEntry` so it can be tested without
/// constructing real Wayland windows.
fn compute_layout_metrics(
    count: usize,
    position: f64,
    stage_progress: f64,
    screen_w: f64,
    screen_h: f64,
) -> SwitcherLayout {
    let visual = visual_center(count, position);

    let slots = count.clamp(1, VISIBLE_SLOTS) as f64;
    let panel_h = slots * SLOT + PANEL_PAD * 2.0;
    let panel_x = screen_w - PANEL_WIDTH;
    let panel_y = (screen_h - panel_h) / 2.0;
    let center_y = screen_h / 2.0;
    let panel = Rect::new(panel_x, panel_y, PANEL_WIDTH, panel_h);

    let (first, last) = if count <= VISIBLE_SLOTS {
        (0, count)
    } else {
        let half = (VISIBLE_SLOTS / 2) as f64;
        (
            (visual - half - 1.0).floor().max(0.0) as usize,
            ((visual + half + 1.0).ceil() as usize).min(count),
        )
    };

    let mut dock_items = Vec::new();
    for index in first..last {
        let rel = index as f64 - visual;
        let selection_rel = index as f64 - position;
        let proximity = (1.0 - selection_rel.abs()).clamp(0.0, 1.0);
        let size = ICON + (ICON_SELECTED - ICON) * proximity * proximity;
        let center_x = panel_x + PANEL_WIDTH / 2.0;
        let center_y = center_y + rel * SLOT;
        let card = Rect::new(center_x - size / 2.0, center_y - size / 2.0, size, size);
        dock_items.push((
            index,
            CardLayout {
                card,
                preview: card,
                radius: size * 0.24,
                opacity: 1.0,
            },
        ));
    }

    let active_center_y = center_y + (position - visual) * SLOT;
    let active = Rect::new(
        panel_x + (PANEL_WIDTH - SELECTION_SIZE) / 2.0,
        active_center_y - SELECTION_SIZE / 2.0,
        SELECTION_SIZE,
        SELECTION_SIZE,
    );

    let preview_x = (panel_x - PREVIEW_GAP - PREVIEW_W).max(8.0);
    let preview_y = (active_center_y - PREVIEW_H / 2.0).clamp(8.0, screen_h - PREVIEW_H - 8.0);
    let preview = Rect::new(preview_x, preview_y, PREVIEW_W, PREVIEW_H);

    SwitcherLayout {
        panel,
        dock_items,
        active,
        preview,
        preview_alpha: stage_progress.clamp(0.0, 1.0),
    }
}

/// Draws the capsule, icon slots and the title strip of the small preview.
/// The live window pixels are composited by `element.rs` underneath this.
pub(crate) fn draw_switcher_foreground(
    context: &Context,
    switcher: &AltTabSwitcher,
    layout: &SwitcherLayout,
) -> Result<(), cairo::Error> {
    context.set_operator(Operator::Source);
    context.set_source_rgba(0.0, 0.0, 0.0, 0.0);
    context.paint()?;
    context.set_operator(Operator::Over);

    draw_docked_panel(
        context,
        layout.panel.x,
        layout.panel.y,
        layout.panel.width,
        layout.panel.height,
        PANEL_CORNER_RADIUS,
        PANEL_FILLET_RADIUS,
        PANEL_SURFACE,
        HAIRLINE,
        DockSide::Right,
    )?;

    context.save()?;
    docked_panel_path(
        context,
        layout.panel.x,
        layout.panel.y,
        layout.panel.width,
        layout.panel.height,
        PANEL_CORNER_RADIUS,
        PANEL_FILLET_RADIUS,
        DockSide::Right,
    )?;
    context.clip();

    paint_rounded(
        context,
        layout.active.x,
        layout.active.y,
        layout.active.width,
        layout.active.height,
        17.0,
        (1.0, 1.0, 1.0, 0.09),
        None,
    )?;

    for (index, item) in &layout.dock_items {
        if let Some(entry) = switcher.entries.get(*index) {
            let card = item.card;
            draw_app_badge(
                context,
                entry,
                card.x,
                card.y,
                card.width.min(card.height),
                item.opacity,
            )?;
        }
    }

    // White pill on the inner edge, mirroring the launcher workspace dot.
    paint_rounded(
        context,
        layout.panel.x + layout.panel.width - 11.0,
        layout.active.y + (layout.active.height - 16.0) / 2.0,
        3.0,
        16.0,
        1.5,
        (1.0, 1.0, 1.0, 0.92),
        None,
    )?;
    context.restore()?;

    // Title inside the bottom of the small preview.
    if let Some(entry) = switcher.entries.get(switcher.selected_index) {
        let p = layout.preview;
        context.save()?;
        context.rectangle(p.x, p.y + p.height - TITLE_H, p.width, TITLE_H);
        context.clip();

        let gradient =
            cairo::LinearGradient::new(0.0, p.y + p.height - TITLE_H, 0.0, p.y + p.height);
        gradient.add_color_stop_rgba(0.0, 0.0, 0.0, 0.0, 0.0);
        gradient.add_color_stop_rgba(0.45, 0.0, 0.0, 0.0, 0.62);
        gradient.add_color_stop_rgba(1.0, 0.0, 0.0, 0.0, 0.88);
        context.set_source(&gradient).ok();
        context.paint()?;

        set_bold_font(context, TITLE_FONT);
        let title = fit_text(context, &entry.app_name, p.width - TITLE_PAD * 2.0)?;
        let extents = context.text_extents(&title)?;
        let tx = p.x + (p.width - extents.x_advance()) / 2.0 - extents.x_bearing();
        let ty =
            p.y + p.height - TITLE_H + (TITLE_H - extents.height()) / 2.0 - extents.y_bearing();
        draw_text(context, &title, tx, ty, (1.0, 1.0, 1.0, 0.96))?;
        context.restore()?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use animation::Tween;

    use super::*;

    #[test]
    fn empty_switcher_has_no_layout() {
        let switcher = AltTabSwitcher {
            entries: Vec::new(),
            selected_index: 0,
            scroll: Tween::settled(0.0),
            stage: Tween::settled(1.0),
            scroll_direction: 1.0,
        };
        assert!(compute_layout(&switcher, 1920.0, 1080.0).is_none());
    }

    #[test]
    fn capsule_is_docked_to_the_right_and_preview_sits_left_of_it() {
        let layout = compute_layout_metrics(4, 0.0, 1.0, 1920.0, 1080.0);

        assert_eq!(layout.panel.x + layout.panel.width, 1920.0);
        assert!(layout.preview.x + layout.preview.width <= layout.panel.x);
        assert_eq!(layout.preview_alpha, 1.0);
    }

    #[test]
    fn short_lists_keep_all_icons_visible() {
        let layout = compute_layout_metrics(3, 0.0, 1.0, 1920.0, 1080.0);
        assert_eq!(layout.dock_items.len(), 3);
        assert_eq!(layout.panel.height, 3.0 * SLOT + PANEL_PAD * 2.0);
    }

    #[test]
    fn long_lists_clamp_the_visual_center() {
        assert_eq!(visual_center(12, 0.0), 4.0);
        assert_eq!(visual_center(12, 11.0), 7.0);

        let layout = compute_layout_metrics(12, 11.0, 1.0, 1920.0, 1080.0);
        assert!(layout.dock_items.len() <= VISIBLE_SLOTS + 2);
    }
}
