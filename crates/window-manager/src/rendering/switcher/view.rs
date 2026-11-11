use std::time::Duration;

use cairo::{Context, Operator};

use crate::compositor::AltTabSwitcher;

use super::draw::{
    draw_app_badge, draw_card_border, draw_dots_header, draw_shadow, draw_stage_header_pill,
    rounded_rect, Rect,
};

pub(crate) const ANIMATION_DURATION: Duration = Duration::from_millis(260);

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct CardLayout {
    pub(crate) card: Rect,
    pub(crate) preview: Rect,
    pub(crate) radius: f64,
}

pub(crate) struct SwitcherLayout {
    pub(crate) stage: (usize, CardLayout),
    pub(crate) dock_items: Vec<(usize, CardLayout)>,
    pub(crate) alpha: f64,
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

    // Left Dock: Fixed position (no flickering/jumping when switching tabs)
    let dock_x = 24.0;
    let top_margin = 32.0;
    let bottom_margin = 32.0;
    let avail_dock_h = (screen_h - top_margin - bottom_margin).max(120.0);
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
    let dock_start_y = ((screen_h - total_dock_h) / 2.0).max(top_margin);

    let dock_items: Vec<(usize, CardLayout)> = (0..n)
        .map(|idx| {
            let is_sel = idx == sel;
            let card_y = dock_start_y + idx as f64 * (base_thumb_h + gap);
            // Active item pops out bigger (1.10x scale and +6px offset)
            let (thumb_w, thumb_h, card_x, card_y_adj) = if is_sel {
                let w = base_thumb_w * 1.10;
                let h = base_thumb_h * 1.10;
                let x = dock_x + 6.0;
                let y = card_y - (h - base_thumb_h) / 2.0;
                (w, h, x, y)
            } else {
                (base_thumb_w, base_thumb_h, dock_x, card_y)
            };

            let card = Rect::new(card_x, card_y_adj, thumb_w, thumb_h);
            let radius = (thumb_h * 0.13).clamp(8.0, 14.0);
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
                },
            )
        })
        .collect();

    // Center Stage: MUCH LARGER - occupying most of the remaining screen area
    let stage_min_x = dock_x + base_thumb_w * 1.10 + 28.0;
    let stage_max_x = screen_w - 24.0;
    let avail_w = (stage_max_x - stage_min_x).max(240.0);
    let avail_h = (screen_h - 48.0).max(200.0);

    let sel_win = &switcher.entries[sel].window;
    let geo_size = sel_win.geometry().size;
    let bbox_size = sel_win.bbox().size;
    let size = if geo_size.w > 0 && geo_size.h > 0 {
        geo_size
    } else if bbox_size.w > 0 && bbox_size.h > 0 {
        bbox_size
    } else {
        (1280, 800).into()
    };
    let aspect = (size.w as f64 / size.h as f64).clamp(0.65, 2.3);

    // Occupy 96% of available width and 94% of available height
    let max_stage_w = (avail_w * 0.96).max(400.0);
    let max_stage_h = (avail_h * 0.94).max(300.0);
    let mut stage_w = max_stage_w;
    let mut stage_h = stage_w / aspect;
    if stage_h > max_stage_h {
        stage_h = max_stage_h;
        stage_w = stage_h * aspect;
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
    let stage = (
        sel,
        CardLayout {
            card: stage_card,
            preview: stage_preview,
            radius: 18.0,
        },
    );

    Some(SwitcherLayout {
        stage,
        dock_items,
        alpha: 1.0,
    })
}

pub(crate) fn draw_switcher_background(
    context: &Context,
    switcher: &AltTabSwitcher,
    layout: &SwitcherLayout,
) -> Result<(), cairo::Error> {
    context.set_operator(Operator::Source);
    context.set_source_rgba(0.0, 0.0, 0.0, 0.0);
    context.paint()?;
    context.set_operator(Operator::Over);

    // 1. Dock thumbnail drop shadows
    for (idx, item) in &layout.dock_items {
        let is_selected = *idx == switcher.selected_index;
        if is_selected {
            draw_shadow(
                context,
                item.card.x,
                item.card.y,
                item.card.width,
                item.card.height,
                item.radius,
                24.0,
                0.65,
            )?;
        } else {
            draw_shadow(
                context,
                item.card.x,
                item.card.y,
                item.card.width,
                item.card.height,
                item.radius,
                14.0,
                0.35,
            )?;
        }
    }

    // 2. Center stage deep soft shadow for the large real app window
    let (_, front) = &layout.stage;
    draw_shadow(
        context,
        front.card.x,
        front.card.y,
        front.card.width,
        front.card.height,
        front.radius,
        38.0,
        0.60,
    )?;

    Ok(())
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
    context.set_source_rgba(0.38, 0.68, 1.0, 0.45);
    context.set_line_width(3.0);
    context.stroke()?;

    rounded_rect(
        context,
        front.card.x,
        front.card.y,
        front.card.width,
        front.card.height,
        front.radius,
    );
    context.set_source_rgba(1.0, 1.0, 1.0, 0.85);
    context.set_line_width(1.8);
    context.stroke()?;

    draw_dots_header(
        context,
        front.card.center_x(),
        front.card.y + 11.0,
        1.0,
    )?;

    if let Some(entry) = switcher.entries.get(*sel_idx) {
        let pill_y = (front.card.y - 34.0).max(6.0);
        draw_stage_header_pill(
            context,
            entry,
            front.card.center_x(),
            pill_y,
            front.card.width,
            1.0,
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
            context.set_source_rgba(0.35, 0.65, 1.0, 0.65);
            context.set_line_width(3.0);
            context.stroke()?;

            rounded_rect(
                context,
                item.card.x,
                item.card.y,
                item.card.width,
                item.card.height,
                item.radius,
            );
            context.set_source_rgba(1.0, 1.0, 1.0, 0.95);
            context.set_line_width(2.0);
            context.stroke()?;
        } else {
            draw_card_border(context, item.card, item.radius, false, 1.0)?;
        }

        if let Some(entry) = switcher.entries.get(*entry_idx) {
            let badge_size = (item.card.height * 0.32).clamp(20.0, 30.0);
            let badge_x = item.card.x + 4.0;
            let badge_y = item.card.y + item.card.height - badge_size - 4.0;
            draw_app_badge(
                context,
                entry,
                badge_x,
                badge_y,
                badge_size,
                1.0,
            )?;
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

    #[test]
    fn test_card_rect_inset() {
        let rect = Rect::new(10.0, 20.0, 100.0, 80.0);
        let inset = rect.inset(5.0, 10.0);
        assert_eq!(inset.x, 15.0);
        assert_eq!(inset.y, 30.0);
        assert_eq!(inset.width, 90.0);
        assert_eq!(inset.height, 60.0);
    }
}
