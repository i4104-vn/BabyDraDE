use std::time::Duration;

use cairo::{Context, Operator};

use crate::compositor::AltTabSwitcher;

use super::draw::{draw_app_badge, draw_card_border, Rect};

pub(crate) const ANIMATION_DURATION: Duration = Duration::from_millis(420);

// The switcher is a layout, not a collection of windows laid out at their
// native sizes. Keeping these values independent from the selected window
// prevents the dock and the stage from moving while Alt+Tab is held.
const DOCK_X: f64 = 24.0;
const DOCK_TOP_MARGIN: f64 = 32.0;
const DOCK_BOTTOM_MARGIN: f64 = 32.0;
const STAGE_SIDE_GAP: f64 = 28.0;
const STAGE_RIGHT_MARGIN: f64 = 24.0;
const STAGE_ASPECT_RATIO: f64 = 16.0 / 9.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct CardLayout {
    pub(crate) card: Rect,
    pub(crate) preview: Rect,
    pub(crate) radius: f64,
    pub(crate) opacity: f64,
}

pub(crate) struct SwitcherLayout {
    pub(crate) stage: (usize, CardLayout),
    pub(crate) dock_items: Vec<(usize, CardLayout)>,
}

pub(crate) fn compute_layout(
    switcher: &AltTabSwitcher,
    screen_w: f64,
    screen_h: f64,
) -> Option<SwitcherLayout> {
    if switcher.entries.is_empty() {
        return None;
    }

    let n = switcher.entries.len();
    let sel = switcher.selected_index.min(n - 1);
    let elapsed = switcher.transition_started.elapsed().as_secs_f64();
    let progress = ease_out_cubic((elapsed / ANIMATION_DURATION.as_secs_f64()).clamp(0.0, 1.0));

    // Dock thumbnails stay fixed while the selected preview animates in the center.
    let avail_dock_h = (screen_h - DOCK_TOP_MARGIN - DOCK_BOTTOM_MARGIN).max(120.0);
    let gap = if n <= 4 {
        16.0
    } else if n <= 7 {
        12.0
    } else {
        8.0
    };

    let max_per_item_h = ((avail_dock_h - (n.saturating_sub(1) as f64) * gap) / n as f64).floor();
    let base_thumb_h = max_per_item_h.clamp(52.0, 115.0);
    let base_thumb_w = (base_thumb_h / 0.62).clamp(84.0, 175.0);
    let thumb_pad = 3.0;

    let total_dock_h = n as f64 * base_thumb_h + (n.saturating_sub(1) as f64) * gap;
    let dock_start_y = ((screen_h - total_dock_h) / 2.0).max(DOCK_TOP_MARGIN);

    let dock_items: Vec<(usize, CardLayout)> = (0..n)
        .map(|idx| {
            let card_y = dock_start_y + idx as f64 * (base_thumb_h + gap);
            let card = Rect::new(DOCK_X, card_y, base_thumb_w, base_thumb_h);
            let radius = (base_thumb_h * 0.13).clamp(8.0, 14.0);
            let preview = Rect::new(
                card.x + thumb_pad,
                card.y + thumb_pad,
                card.width - 2.0 * thumb_pad,
                card.height - 2.0 * thumb_pad,
            );

            (
                idx,
                CardLayout {
                    card,
                    preview,
                    radius,
                    opacity: 1.0,
                },
            )
        })
        .collect();

    // Center stage: a fixed 16:9 presentation area. The selected app is fit
    // into this area by the renderer below; its native aspect ratio must not
    // change the stage geometry.
    let stage_min_x = DOCK_X + base_thumb_w + STAGE_SIDE_GAP;
    let stage_max_x = screen_w - STAGE_RIGHT_MARGIN;
    let avail_w = (stage_max_x - stage_min_x).max(240.0);
    let avail_h = (screen_h - 48.0).max(200.0);

    // Occupy most of the available area while remaining stable for every app.
    let max_stage_w = avail_w * 0.96;
    let max_stage_h = avail_h * 0.94;
    let mut stage_w = max_stage_w;
    let mut stage_h = stage_w / STAGE_ASPECT_RATIO;
    if stage_h > max_stage_h {
        stage_h = max_stage_h;
        stage_w = stage_h * STAGE_ASPECT_RATIO;
    }

    let stage_x = stage_min_x + (avail_w - stage_w) / 2.0;
    let stage_y = ((screen_h - stage_h) / 2.0).max(36.0);

    let stage_card = Rect::new(stage_x, stage_y, stage_w, stage_h);
    let stage_preview = stage_card;
    let stage_center = (stage_card.center_x(), stage_card.y + stage_card.height / 2.0);
    let stage_scale = 0.965 + 0.035 * progress;
    let stage_card = scale_rect_around(stage_card, stage_center, stage_scale);
    let stage_preview = scale_rect_around(stage_preview, stage_center, stage_scale);
    let stage = (
        sel,
        CardLayout {
            card: stage_card,
            preview: stage_preview,
            radius: 18.0,
            opacity: 0.28 + 0.72 * progress,
        },
    );

    Some(SwitcherLayout {
        stage,
        dock_items,
    })
}

fn ease_out_cubic(value: f64) -> f64 {
    1.0 - (1.0 - value).powi(3)
}

fn scale_rect_around(rect: Rect, center: (f64, f64), scale: f64) -> Rect {
    let width = rect.width * scale;
    let height = rect.height * scale;
    Rect::new(
        center.0 - width / 2.0,
        center.1 - height / 2.0,
        width,
        height,
    )
}

pub(crate) fn draw_switcher_foreground(
    context: &Context,
    switcher: &AltTabSwitcher,
    layout: &SwitcherLayout,
) -> Result<(), cairo::Error> {
    context.set_operator(Operator::Source);
    context.set_source_rgba(0.0, 0.0, 0.0, 0.0);
    context.paint()?;
    context.set_operator(Operator::Over);

    // The center preview has no frame or backdrop. Only the dock is drawn here,
    // with fixed thumbnail geometry and a static selection outline.
    for (entry_idx, item) in &layout.dock_items {
        let is_selected = *entry_idx == switcher.selected_index;
        draw_card_border(context, item.card, item.radius, is_selected, 1.0)?;

        if let Some(entry) = switcher.entries.get(*entry_idx) {
            let badge_size = (item.card.height * 0.32).clamp(20.0, 30.0);
            let badge_x = item.card.x + 4.0;
            let badge_y = item.card.y + item.card.height - badge_size - 4.0;
            draw_app_badge(context, entry, badge_x, badge_y, badge_size, 1.0)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_layout_empty() {
        let switcher = AltTabSwitcher {
            entries: Vec::new(),
            selected_index: 0,
            transition_started: std::time::Instant::now(),
        };
        assert!(compute_layout(&switcher, 1920.0, 1080.0).is_none());
    }

}
