use std::time::Duration;

use cairo::{Context, Operator};

use crate::compositor::AltTabSwitcher;

use super::draw::{
    draw_app_badge, draw_card_border, draw_stage_header_pill, rounded_rect, Rect,
};

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
    pub(crate) pulse: f64,
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
    let pulse = (progress * std::f64::consts::PI).sin().max(0.0);

    // Left dock: every card has the same geometry. Selection is indicated by
    // its border only, so changing the selected app cannot reflow the dock.
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

            // Stagger the dock from top to bottom. Each preview slides in from
            // the left and settles with a small spring-like lift when selected.
            let item_elapsed = (elapsed - idx as f64 * 0.028).max(0.0);
            let item_progress = ease_out_cubic((item_elapsed / 0.30).clamp(0.0, 1.0));
            let selected_lift = if idx == sel {
                0.035 * (item_progress * std::f64::consts::PI).sin().max(0.0)
            } else {
                0.0
            };
            let scale = 0.94 + 0.06 * item_progress + selected_lift;
            let center = (card.center_x(), card.y + card.height / 2.0);
            let mut card = scale_rect_around(card, center, scale);
            let mut preview = scale_rect_around(preview, center, scale);
            let slide_x = -26.0 * (1.0 - item_progress);
            card.x += slide_x;
            preview.x += slide_x;

            (
                idx,
                CardLayout {
                    card,
                    preview,
                    radius,
                    opacity: 0.16 + 0.84 * item_progress,
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

    let header_h = 24.0;
    let stage_pad = 4.0;
    let stage_card = Rect::new(stage_x, stage_y, stage_w, stage_h);
    let stage_preview = Rect::new(
        stage_x + stage_pad,
        stage_y + header_h,
        stage_w - 2.0 * stage_pad,
        stage_h - header_h - stage_pad,
    );
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
        pulse,
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

    // 1. Center stage: prominent active border, header dots & title pill above real window
    let (sel_idx, front) = &layout.stage;

    // Glowing active border: outer blue glow ring + inner crisp white border
    rounded_rect(
        context,
        front.card.x - 1.5,
        front.card.y - 1.5,
        front.card.width + 3.0,
        front.card.height + 3.0,
        front.radius + 1.5,
    );
    context.set_source_rgba(0.38, 0.68, 1.0, (0.32 + 0.36 * layout.pulse) * front.opacity);
    context.set_line_width(2.4 + 1.8 * layout.pulse);
    context.stroke()?;

    rounded_rect(
        context,
        front.card.x,
        front.card.y,
        front.card.width,
        front.card.height,
        front.radius,
    );
    context.set_source_rgba(1.0, 1.0, 1.0, 0.85 * front.opacity);
    context.set_line_width(1.8);
    context.stroke()?;

    if let Some(entry) = switcher.entries.get(*sel_idx) {
        let pill_y = (front.card.y - 34.0).max(6.0);
        draw_stage_header_pill(
            context,
            entry,
            front.card.center_x(),
            pill_y,
            front.card.width,
            front.opacity,
        )?;
    }

    // 2. Dock thumbnails: borders & app icon badges (active item pops out with glowing active border)
    for (entry_idx, item) in &layout.dock_items {
        let is_selected = *entry_idx == switcher.selected_index;
        if is_selected {
            // Prominent active border: outer blue glow + crisp bright inner line
            rounded_rect(
                context,
                item.card.x - 1.5,
                item.card.y - 1.5,
                item.card.width + 3.0,
                item.card.height + 3.0,
                item.radius + 1.5,
            );
            context.set_source_rgba(
                0.35,
                0.65,
                1.0,
                (0.48 + 0.32 * layout.pulse) * item.opacity,
            );
            context.set_line_width(2.4 + 1.6 * layout.pulse);
            context.stroke()?;

            rounded_rect(
                context,
                item.card.x,
                item.card.y,
                item.card.width,
                item.card.height,
                item.radius,
            );
            context.set_source_rgba(1.0, 1.0, 1.0, 0.95 * item.opacity);
            context.set_line_width(2.0);
            context.stroke()?;
        } else {
            draw_card_border(context, item.card, item.radius, false, item.opacity)?;
        }

        if let Some(entry) = switcher.entries.get(*entry_idx) {
            let badge_size = (item.card.height * 0.32).clamp(20.0, 30.0);
            let badge_x = item.card.x + 4.0;
            let badge_y = item.card.y + item.card.height - badge_size - 4.0;
            draw_app_badge(context, entry, badge_x, badge_y, badge_size, item.opacity)?;
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
