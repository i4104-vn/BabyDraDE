use std::time::Duration;

use cairo::{Context, Operator};

use crate::compositor::AltTabSwitcher;

use super::draw::{app_color, draw_app_badge, Rect};

pub(crate) const ANIMATION_DURATION: Duration = Duration::from_millis(420);

// The switcher is a layout, not a collection of windows laid out at their
// native sizes. Keeping these values independent from the selected window
// prevents the dock and the stage from moving while Alt+Tab is held.
const DOCK_X: f64 = 24.0;
const DOCK_TOP_MARGIN: f64 = 32.0;
const DOCK_BOTTOM_MARGIN: f64 = 32.0;
const DOCK_THUMBNAIL_WIDTH: f64 = 175.0;
const DOCK_THUMBNAIL_HEIGHT: f64 = 115.0;
const DOCK_GAP: f64 = 12.0;
const DOCK_MAX_VISIBLE_ITEMS: usize = 5;
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
    pub(crate) dock_viewport: Rect,
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

    // The dock has a bounded viewport. Cards keep a fixed size, and the
    // selected entry stays centered while the list moves vertically.
    let (dock_viewport, dock_items) = compute_dock_layout(n, sel, screen_h);
    let base_thumb_w = DOCK_THUMBNAIL_WIDTH;

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
    let stage_center = (
        stage_card.center_x(),
        stage_card.y + stage_card.height / 2.0,
    );
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
        dock_viewport,
        dock_items,
    })
}

fn compute_dock_layout(
    entry_count: usize,
    selected_index: usize,
    screen_h: f64,
) -> (Rect, Vec<(usize, CardLayout)>) {
    if entry_count == 0 {
        return (
            Rect::new(DOCK_X, 0.0, DOCK_THUMBNAIL_WIDTH, 0.0),
            Vec::new(),
        );
    }

    let viewport_y = DOCK_TOP_MARGIN.min((screen_h - 1.0).max(0.0));
    let viewport_bottom = (screen_h - DOCK_BOTTOM_MARGIN).max(viewport_y + 1.0);
    let viewport = Rect::new(
        DOCK_X,
        viewport_y,
        DOCK_THUMBNAIL_WIDTH,
        viewport_bottom - viewport_y,
    );

    let visible_count = entry_count.min(DOCK_MAX_VISIBLE_ITEMS);
    let desired_height = visible_count as f64 * DOCK_THUMBNAIL_HEIGHT
        + visible_count.saturating_sub(1) as f64 * DOCK_GAP;
    let scale = (viewport.height / desired_height).min(1.0);
    let thumbnail_width = DOCK_THUMBNAIL_WIDTH * scale;
    let thumbnail_height = DOCK_THUMBNAIL_HEIGHT * scale;
    let gap = DOCK_GAP * scale;
    let item_step = thumbnail_height + gap;
    let items_height =
        visible_count as f64 * thumbnail_height + visible_count.saturating_sub(1) as f64 * gap;
    let first_item_y = viewport.y + (viewport.height - items_height) / 2.0;
    let thumb_pad = 3.0 * scale;
    let thumbnail_x = DOCK_X + (DOCK_THUMBNAIL_WIDTH - thumbnail_width) / 2.0;

    // Keep a circular slice around the selection. This makes the entries at
    // either end of the Alt+Tab list adjacent in the dock as well.
    let before_count = visible_count / 2;
    let first_index = if selected_index >= before_count {
        selected_index - before_count
    } else {
        entry_count - (before_count - selected_index)
    };

    let dock_items = (0..visible_count)
        .map(|slot| {
            let idx = (first_index + slot) % entry_count;
            let card = Rect::new(
                thumbnail_x,
                first_item_y + slot as f64 * item_step,
                thumbnail_width,
                thumbnail_height,
            );
            let preview = Rect::new(
                card.x + thumb_pad,
                card.y + thumb_pad,
                (card.width - 2.0 * thumb_pad).max(0.0),
                (card.height - 2.0 * thumb_pad).max(0.0),
            );

            (
                idx,
                CardLayout {
                    card,
                    preview,
                    radius: (thumbnail_height * 0.13).clamp(4.0, 14.0),
                    opacity: 1.0,
                },
            )
        })
        .collect();

    (viewport, dock_items)
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

    let vp = layout.dock_viewport;

    // ── Per-card decorations (clipped to viewport) ─────────────────────────────
    context.save()?;
    context.rectangle(vp.x, vp.y, vp.width, vp.height);
    context.clip();

    for (entry_idx, item) in &layout.dock_items {
        let is_selected = *entry_idx == switcher.selected_index;
        let p = item.preview;

        // Perspective depth gradient: transparent on the left (near side),
        // dark on the right (far side) — the right edge recedes to a vanishing
        // point. Left edge appears taller because the right corners are clipped
        // by the trapezoidal shape.
        {
            let persp = (p.height * 0.10).clamp(6.0, 16.0);

            // Clip gradient to trapezoid: left side full height, right shorter.
            context.save()?;
            context.new_path();
            context.move_to(p.x, p.y);
            context.line_to(p.x + p.width, p.y + persp);
            context.line_to(p.x + p.width, p.y + p.height - persp);
            context.line_to(p.x, p.y + p.height);
            context.close_path();
            context.clip();

            // Horizontal dark gradient: clear on left, dark on right.
            let grad = cairo::LinearGradient::new(p.x, 0.0, p.x + p.width, 0.0);
            grad.add_color_stop_rgba(0.0, 0.0, 0.0, 0.0, 0.0);
            grad.add_color_stop_rgba(0.45, 0.0, 0.0, 0.0, 0.0);
            grad.add_color_stop_rgba(0.80, 0.0, 0.0, 0.0, 0.28);
            grad.add_color_stop_rgba(1.0, 0.0, 0.0, 0.0, 0.65);
            context.rectangle(p.x, p.y, p.width, p.height);
            context.set_source(&grad)?;
            context.fill()?;
            context.restore()?;

            // Thin bright highlight on the left edge: near-side lighting.
            let hi = cairo::LinearGradient::new(p.x, 0.0, p.x + 12.0, 0.0);
            hi.add_color_stop_rgba(0.0, 1.0, 1.0, 1.0, 0.20);
            hi.add_color_stop_rgba(1.0, 1.0, 1.0, 1.0, 0.0);
            context.rectangle(p.x, p.y, 12.0, p.height);
            context.set_source(&hi)?;
            context.fill()?;
        }

        // Active card: trapezoidal border tight around preview. No container.
        if is_selected {
            if let Some(entry) = switcher.entries.get(*entry_idx) {
                let (r, g, b) = app_color(&entry.app_name);
                let persp = (p.height * 0.10).clamp(6.0, 16.0);

                context.new_path();
                context.move_to(p.x, p.y);
                context.line_to(p.x + p.width, p.y + persp);
                context.line_to(p.x + p.width, p.y + p.height - persp);
                context.line_to(p.x, p.y + p.height);
                context.close_path();
                context.set_source_rgba(
                    (r + 0.35).min(1.0),
                    (g + 0.35).min(1.0),
                    (b + 0.35).min(1.0),
                    0.95,
                );
                context.set_line_width(2.2);
                context.stroke()?;

                // Badge only on active card.
                let badge_size = (item.card.height * 0.32).clamp(20.0, 30.0);
                draw_app_badge(
                    context,
                    entry,
                    p.x + 4.0,
                    p.y + p.height - badge_size - 4.0,
                    badge_size,
                    1.0,
                )?;
            }
        } else if let Some(entry) = switcher.entries.get(*entry_idx) {
            // Inactive: faint badge so the user can still identify the app.
            let badge_size = (item.card.height * 0.32).clamp(20.0, 30.0);
            draw_app_badge(
                context,
                entry,
                p.x + 4.0,
                p.y + p.height - badge_size - 4.0,
                badge_size,
                0.55,
            )?;
        }
    }

    context.restore()?; // end dock clip

    // ── Scroll fade gradients at actual items list boundaries ──────────────────
    // Multi-stop smooth gradient anchored to first/last card edge.
    if let (Some((_, first)), Some((_, last))) =
        (layout.dock_items.first(), layout.dock_items.last())
    {
        let top_y = first.card.y;
        let bottom_y = last.card.y + last.card.height;
        // Covers ~half a card for a smooth, gradual feel.
        let fade_h = (DOCK_THUMBNAIL_HEIGHT * 0.55).clamp(36.0, 72.0);

        // Top: opaque at first card edge → transparent half-a-card down.
        {
            let grad = cairo::LinearGradient::new(0.0, top_y, 0.0, top_y + fade_h);
            grad.add_color_stop_rgba(0.0, 0.03, 0.03, 0.06, 0.95);
            grad.add_color_stop_rgba(0.30, 0.03, 0.03, 0.06, 0.55);
            grad.add_color_stop_rgba(0.65, 0.03, 0.03, 0.06, 0.12);
            grad.add_color_stop_rgba(1.0, 0.03, 0.03, 0.06, 0.0);
            context.rectangle(vp.x, top_y, vp.width, fade_h);
            context.set_source(&grad)?;
            context.fill()?;
        }

        // Bottom: transparent half-a-card up → opaque at last card edge.
        {
            let grad =
                cairo::LinearGradient::new(0.0, bottom_y - fade_h, 0.0, bottom_y);
            grad.add_color_stop_rgba(0.0, 0.03, 0.03, 0.06, 0.0);
            grad.add_color_stop_rgba(0.35, 0.03, 0.03, 0.06, 0.12);
            grad.add_color_stop_rgba(0.70, 0.03, 0.03, 0.06, 0.55);
            grad.add_color_stop_rgba(1.0, 0.03, 0.03, 0.06, 0.95);
            context.rectangle(vp.x, bottom_y - fade_h, vp.width, fade_h);
            context.set_source(&grad)?;
            context.fill()?;
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
    fn dock_viewport_is_bounded_and_thumbnail_size_is_fixed() {
        let (short_viewport, short_items) = compute_dock_layout(2, 0, 1080.0);
        let (long_viewport, long_items) = compute_dock_layout(30, 12, 1080.0);

        assert_eq!(short_viewport.height, 1016.0);
        assert_eq!(long_viewport.height, short_viewport.height);
        assert_eq!(short_items[0].1.card.width, DOCK_THUMBNAIL_WIDTH);
        assert_eq!(short_items[0].1.card.height, DOCK_THUMBNAIL_HEIGHT);
        assert_eq!(long_items[0].1.card.width, DOCK_THUMBNAIL_WIDTH);
        assert_eq!(long_items[0].1.card.height, DOCK_THUMBNAIL_HEIGHT);
    }

    #[test]
    fn selected_dock_item_stays_centered_at_both_ends_and_in_the_middle() {
        let entry_count = 30;

        for selected_index in [0, 14, entry_count - 1] {
            let (viewport, items) = compute_dock_layout(entry_count, selected_index, 1080.0);
            let (_, selected) = items
                .iter()
                .find(|(idx, _)| *idx == selected_index)
                .expect("selected item should intersect the viewport");
            let selected_center = selected.card.y + selected.card.height / 2.0;
            let viewport_center = viewport.y + viewport.height / 2.0;

            assert!((selected_center - viewport_center).abs() < 0.001);
        }
    }

    #[test]
    fn visible_dock_items_keep_top_to_bottom_entry_order() {
        let (_, items) = compute_dock_layout(30, 12, 1080.0);

        assert!(items
            .windows(2)
            .all(|pair| { pair[0].0 < pair[1].0 && pair[0].1.card.y < pair[1].1.card.y }));
    }
}
