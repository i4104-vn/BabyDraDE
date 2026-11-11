use std::time::Duration;

use cairo::{Context, Operator};

use crate::compositor::AltTabSwitcher;

use super::draw::{
    draw_app_badge, draw_card_background, draw_card_border, draw_dots_header, draw_shadow,
    draw_stage_header_pill, Rect,
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

    let progress = (switcher.transition_started.elapsed().as_secs_f64()
        / ANIMATION_DURATION.as_secs_f64())
    .clamp(0.0, 1.0);
    let eased = 1.0 - (1.0 - progress).powi(3);
    let alpha = (eased * 1.3).min(1.0);

    let n = switcher.entries.len();
    let sel = switcher.selected_index.min(n - 1);

    // Left Dock: ALL existing apps are displayed
    let dock_x = 24.0 - (1.0 - eased) * 40.0;
    let top_margin = 40.0;
    let bottom_margin = 40.0;
    let avail_dock_h = (screen_h - top_margin - bottom_margin).max(120.0);
    let gap = if n <= 4 {
        16.0
    } else if n <= 7 {
        12.0
    } else {
        8.0
    };

    let max_per_item_h = ((avail_dock_h - (n.saturating_sub(1) as f64) * gap) / n as f64).floor();
    let thumb_h = max_per_item_h.clamp(55.0, 115.0);
    let thumb_w = (thumb_h / 0.62).clamp(90.0, 180.0);
    let thumb_radius = (thumb_h * 0.12).clamp(8.0, 14.0);
    let thumb_pad = 3.0;

    let total_dock_h = n as f64 * thumb_h + (n.saturating_sub(1) as f64) * gap;
    let dock_start_y = ((screen_h - total_dock_h) / 2.0).max(top_margin);

    let dock_items: Vec<(usize, CardLayout)> = (0..n)
        .map(|idx| {
            let card_y = dock_start_y + idx as f64 * (thumb_h + gap);
            let card = Rect::new(dock_x, card_y, thumb_w, thumb_h);
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
                    radius: thumb_radius,
                },
            )
        })
        .collect();

    // Center Stage: ONLY 1 APP (the currently tabbed-to app, no stacking)
    let stage_min_x = dock_x + thumb_w + 36.0;
    let stage_max_x = screen_w - 36.0;
    let avail_w = (stage_max_x - stage_min_x).max(240.0);
    let avail_h = (screen_h - 110.0).max(200.0);

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
    let aspect = (size.w as f64 / size.h as f64).clamp(0.65, 2.2);

    let max_stage_w = (avail_w * 0.88).clamp(380.0, 1050.0);
    let max_stage_h = (avail_h * 0.84).clamp(300.0, 720.0);
    let mut stage_w = max_stage_w;
    let mut stage_h = stage_w / aspect;
    if stage_h > max_stage_h {
        stage_h = max_stage_h;
        stage_w = stage_h * aspect;
    }

    let anim_scale = 0.94 + 0.06 * eased;
    let final_stage_w = stage_w * anim_scale;
    let final_stage_h = stage_h * anim_scale;
    let stage_x = stage_min_x + (avail_w - final_stage_w) / 2.0;
    let stage_y = ((screen_h - final_stage_h) / 2.0).max(48.0);

    let header_h = 24.0;
    let stage_pad = 4.0;
    let stage_card = Rect::new(stage_x, stage_y, final_stage_w, final_stage_h);
    let stage_preview = Rect::new(
        stage_x + stage_pad,
        stage_y + header_h,
        final_stage_w - 2.0 * stage_pad,
        final_stage_h - header_h - stage_pad,
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
        alpha,
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

    // Subtle translucent backdrop tint over desktop wallpaper
    context.set_source_rgba(0.015, 0.02, 0.05, 0.14 * layout.alpha);
    context.paint()?;

    // 1. Dock thumbnail backgrounds (ALL apps)
    for (idx, item) in &layout.dock_items {
        let is_selected = *idx == switcher.selected_index;
        draw_shadow(
            context,
            item.card.x,
            item.card.y,
            item.card.width,
            item.card.height,
            item.radius,
            if is_selected { 20.0 } else { 14.0 },
            if is_selected { 0.55 * layout.alpha } else { 0.40 * layout.alpha },
        )?;
        draw_card_background(
            context,
            item.card,
            item.radius,
            if is_selected { 0.92 * layout.alpha } else { 0.84 * layout.alpha },
        )?;
    }

    // 2. Center stage card background (ONLY 1 APP)
    let (_, front) = &layout.stage;
    draw_shadow(
        context,
        front.card.x,
        front.card.y,
        front.card.width,
        front.card.height,
        front.radius,
        32.0,
        0.65 * layout.alpha,
    )?;
    draw_card_background(context, front.card, front.radius, 0.94 * layout.alpha)?;

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

    // 1. Center stage card border & header (ONLY 1 APP)
    let (sel_idx, front) = &layout.stage;
    draw_card_border(context, front.card, front.radius, true, layout.alpha)?;
    draw_dots_header(
        context,
        front.card.center_x(),
        front.card.y + 11.0,
        layout.alpha,
    )?;

    if let Some(entry) = switcher.entries.get(*sel_idx) {
        let pill_y = (front.card.y - 36.0).max(10.0);
        draw_stage_header_pill(
            context,
            entry,
            front.card.center_x(),
            pill_y,
            front.card.width,
            layout.alpha,
        )?;
    }

    // 2. Dock thumbnails: borders & app icon badges (ALL apps)
    for (entry_idx, item) in &layout.dock_items {
        let is_selected = *entry_idx == switcher.selected_index;
        draw_card_border(context, item.card, item.radius, is_selected, layout.alpha)?;
        if let Some(entry) = switcher.entries.get(*entry_idx) {
            let badge_size = (item.card.height * 0.32).clamp(20.0, 28.0);
            let badge_x = item.card.x + 4.0;
            let badge_y = item.card.y + item.card.height - badge_size - 4.0;
            draw_app_badge(
                context,
                entry,
                badge_x,
                badge_y,
                badge_size,
                layout.alpha,
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
