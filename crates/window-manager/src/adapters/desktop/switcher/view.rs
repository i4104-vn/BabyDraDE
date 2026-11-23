//! Right-side Alt+Tab switcher layout and side-specific content. The capsule,
//! icons and indicator come from `ui_kit::components::dock`; here we only add
//! the small live preview and its title strip.

use cairo::{Context, Operator};
use common::constants::launcher::{SLOT, VISIBLE_SLOTS};
use ui_kit::components::dock::{draw_dock, visual_center, DockLayout, DockSide};
use ui_kit::drawing::{draw_text, fit_text, set_bold_font};

use crate::adapters::compositor::AltTabSwitcher;

use super::draw::{draw_app_badge, Rect};

const PREVIEW_W: f64 = 300.0;
const PREVIEW_H: f64 = 176.0;
const PREVIEW_GAP: f64 = 14.0;
const TITLE_H: f64 = 34.0;
const TITLE_FONT: f64 = 13.5;
const TITLE_PAD: f64 = 12.0;

pub(crate) struct SwitcherLayout {
    /// Shared right-edge capsule.
    pub(crate) panel: DockLayout,
    /// App currently highlighted (pointer or keyboard).
    pub(crate) selected: usize,
    /// Small live preview beside the active icon.
    pub(crate) preview: Rect,
    /// Fade of the preview while switching.
    pub(crate) preview_alpha: f64,
}

pub(crate) fn compute_layout(
    switcher: &AltTabSwitcher,
    screen_w: f64,
    screen_h: f64,
) -> Option<SwitcherLayout> {
    if switcher.entries.is_empty() {
        return None;
    }
    let mut layout = compute_layout_metrics(
        switcher.entries.len(),
        switcher.visual_selection(),
        switcher.stage_progress(),
        switcher.selected_index,
        switcher.highlight_position(),
        screen_w,
        screen_h,
    );
    if let Some(hovered) = switcher.hovered {
        layout.selected = hovered.min(switcher.entries.len() - 1);
    }
    Some(layout)
}

fn dock_active_rel(count: usize, position: f64) -> f64 {
    let visual = visual_center(count, position);
    if count <= VISIBLE_SLOTS {
        position.rem_euclid(count as f64) - visual
    } else {
        position - visual
    }
}

fn compute_layout_metrics(
    count: usize,
    _position: f64,
    stage_progress: f64,
    selected: usize,
    highlight_position: f64,
    screen_w: f64,
    screen_h: f64,
) -> SwitcherLayout {
    let panel = DockLayout::new(count, screen_h / 2.0, screen_w, DockSide::Right);
    let active_y = panel.center_y + dock_active_rel(count, highlight_position) * SLOT;

    let preview = Rect::new(
        (panel.x - PREVIEW_GAP - PREVIEW_W).max(8.0),
        (active_y - PREVIEW_H / 2.0).clamp(8.0, screen_h - PREVIEW_H - 8.0),
        PREVIEW_W,
        PREVIEW_H,
    );

    SwitcherLayout {
        panel,
        selected: selected.min(count - 1),
        preview,
        preview_alpha: stage_progress.clamp(0.0, 1.0),
    }
}

pub(crate) fn draw_switcher_foreground(
    context: &Context,
    switcher: &AltTabSwitcher,
    layout: &SwitcherLayout,
    slide_x: f64,
    presence: f64,
) -> Result<(), cairo::Error> {
    context.set_operator(Operator::Source);
    context.set_source_rgba(0.0, 0.0, 0.0, 0.0);
    context.paint()?;
    context.set_operator(Operator::Over);

    draw_dock(
        context,
        &layout.panel,
        switcher.entries.len(),
        switcher.visual_selection(),
        switcher.highlight_position(),
        Some(layout.selected),
        presence,
        slide_x,
        DockSide::Right,
        |ctx, index, cx, cy, size, _| {
            if let Some(entry) = switcher.entries.get(index) {
                draw_app_badge(ctx, entry, cx - size / 2.0, cy - size / 2.0, size, 1.0)
            } else {
                Ok(())
            }
        },
    )?;

    // Side-specific content: title inside the bottom of the small preview.
    if let Some(entry) = switcher.entries.get(layout.selected) {
        let mut p = layout.preview;
        p.x += slide_x;
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

    fn empty_switcher() -> AltTabSwitcher {
        AltTabSwitcher {
            entries: Vec::new(),
            selected_index: 0,
            hovered: None,
            scroll: Tween::settled(0.0),
            highlight: Tween::settled(0.0),
            stage: Tween::settled(1.0),
            presence: Tween::settled(1.0),
            closing: false,
            last_step: None,
            held_direction: None,
            repeat_started: None,
            last_repeat: None,
            scroll_direction: 1.0,
        }
    }

    #[test]
    fn empty_switcher_has_no_layout() {
        assert!(compute_layout(&empty_switcher(), 1920.0, 1080.0).is_none());
    }

    #[test]
    fn preview_sits_left_of_the_right_capsule() {
        let layout = compute_layout_metrics(4, 0.0, 1.0, 0, 0.0, 1920.0, 1080.0);
        assert_eq!(layout.panel.x + layout.panel.width, 1920.0);
        assert!(layout.preview.x + layout.preview.width <= layout.panel.x);
    }

    #[test]
    fn short_and_long_lists_share_the_dock_geometry() {
        let short = compute_layout_metrics(5, 4.0, 1.0, 4, 4.0, 1920.0, 1080.0);
        let long = compute_layout_metrics(12, 11.0, 1.0, 11, 11.0, 1920.0, 1080.0);
        assert_eq!(short.panel.width, long.panel.width);
        assert_eq!(visual_center(5, 7.0), 2.0);
        assert_eq!(visual_center(12, 11.0), 7.0);
    }
}
