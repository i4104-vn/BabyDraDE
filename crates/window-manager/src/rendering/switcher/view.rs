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
    pub(crate) stage_front: (usize, CardLayout),
    pub(crate) stage_peek: Option<(usize, CardLayout)>,
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

    // Dock geometry
    let dock_x = 28.0 - (1.0 - eased) * 45.0;
    let thumb_w = (screen_w * 0.12).clamp(130.0, 185.0);
    let thumb_h = (thumb_w * 0.62).round();
    let thumb_gap = 18.0;
    let thumb_radius = 12.0;
    let thumb_pad = 3.0;

    let (peek_index, dock_indices) = if n == 1 {
        (None, Vec::new())
    } else if n == 2 {
        let other = 1 - sel;
        (None, vec![other])
    } else {
        let peek = if sel == 0 { n - 1 } else { sel - 1 };
        let others = (0..n).filter(|&i| i != sel && i != peek).collect();
        (Some(peek), others)
    };

    let max_visible_dock = (((screen_h - 120.0) / (thumb_h + thumb_gap)).floor() as usize).max(1);
    let visible_dock_count = dock_indices.len().min(max_visible_dock);
    let total_dock_h = visible_dock_count as f64 * thumb_h
        + (visible_dock_count.saturating_sub(1)) as f64 * thumb_gap;
    let dock_start_y = ((screen_h - total_dock_h) / 2.0).max(45.0);

    let dock_items: Vec<(usize, CardLayout)> = dock_indices
        .into_iter()
        .take(visible_dock_count)
        .enumerate()
        .map(|(idx, entry_idx)| {
            let card_y = dock_start_y + idx as f64 * (thumb_h + thumb_gap);
            let card = Rect::new(dock_x, card_y, thumb_w, thumb_h);
            let preview = Rect::new(
                card.x + thumb_pad,
                card.y + thumb_pad,
                card.width - 2.0 * thumb_pad,
                card.height - 2.0 * thumb_pad,
            );
            (
                entry_idx,
                CardLayout {
                    card,
                    preview,
                    radius: thumb_radius,
                },
            )
        })
        .collect();

    // Stage geometry
    let stage_min_x = dock_x + thumb_w + 36.0;
    let stage_max_x = screen_w - 36.0;
    let avail_w = (stage_max_x - stage_min_x).max(240.0);
    let avail_h = (screen_h - 110.0).max(200.0);

    let sel_win = &switcher.entries[sel].window;
    let geo_size = sel_win.geometry().size;
    let aspect = if geo_size.w > 0 && geo_size.h > 0 {
        geo_size.w as f64 / geo_size.h as f64
    } else {
        1.6
    };
    let aspect = aspect.clamp(0.7, 2.2);

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
    let stage_y = ((screen_h - final_stage_h) / 2.0).max(50.0);

    let header_h = 22.0;
    let stage_pad = 4.0;
    let stage_card = Rect::new(stage_x, stage_y, final_stage_w, final_stage_h);
    let stage_preview = Rect::new(
        stage_x + stage_pad,
        stage_y + header_h,
        final_stage_w - 2.0 * stage_pad,
        final_stage_h - header_h - stage_pad,
    );
    let stage_front = (
        sel,
        CardLayout {
            card: stage_card,
            preview: stage_preview,
            radius: 18.0,
        },
    );

    let stage_peek = peek_index.map(|peek_idx| {
        let peek_scale = 0.94;
        let peek_w = final_stage_w * peek_scale;
        let peek_h = final_stage_h * peek_scale;
        let peek_offset_x = (final_stage_w * 0.08).clamp(35.0, 70.0);
        let peek_x = stage_x - peek_offset_x;
        let peek_y = stage_y + 16.0;
        let card = Rect::new(peek_x, peek_y, peek_w, peek_h);
        let preview = Rect::new(
            peek_x + stage_pad,
            peek_y + header_h,
            peek_w - 2.0 * stage_pad,
            peek_h - header_h - stage_pad,
        );
        (
            peek_idx,
            CardLayout {
                card,
                preview,
                radius: 18.0,
            },
        )
    });

    Some(SwitcherLayout {
        stage_front,
        stage_peek,
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

    // Subtle translucent backdrop so desktop wallpaper remains vivid
    context.set_source_rgba(0.015, 0.02, 0.05, 0.14 * layout.alpha);
    context.paint()?;

    // 1. Peek card background (behind front stage)
    if let Some((_, peek)) = &layout.stage_peek {
        draw_shadow(
            context,
            peek.card.x,
            peek.card.y,
            peek.card.width,
            peek.card.height,
            peek.radius,
            24.0,
            0.50 * layout.alpha,
        )?;
        draw_card_background(context, peek.card, peek.radius, 0.88 * layout.alpha)?;
    }

    // 2. Dock thumbnail backgrounds
    for (_, item) in &layout.dock_items {
        draw_shadow(
            context,
            item.card.x,
            item.card.y,
            item.card.width,
            item.card.height,
            item.radius,
            16.0,
            0.42 * layout.alpha,
        )?;
        draw_card_background(context, item.card, item.radius, 0.86 * layout.alpha)?;
    }

    // 3. Front stage card background
    let (_, front) = &layout.stage_front;
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

    let _ = switcher;
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

    // 1. Peek card border
    if let Some((_, peek)) = &layout.stage_peek {
        draw_card_border(context, peek.card, peek.radius, false, 0.70 * layout.alpha)?;
    }

    // 2. Front stage card border & header
    let (sel_idx, front) = &layout.stage_front;
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

    // 3. Dock thumbnails: borders & app icon badges
    for (entry_idx, item) in &layout.dock_items {
        draw_card_border(context, item.card, item.radius, false, layout.alpha)?;
        if let Some(entry) = switcher.entries.get(*entry_idx) {
            let badge_size = 28.0;
            let badge_x = item.card.x - 3.0;
            let badge_y = item.card.y + item.card.height - 22.0;
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

