use std::time::Duration;

use cairo::{Context, Operator};

use crate::compositor::AltTabSwitcher;

use super::draw::{app_color, draw_app_badge, draw_card_border, rounded_rect, Rect};

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

    // ── Clip to dock viewport ──────────────────────────────────────────────────
    context.save()?;
    context.rectangle(
        layout.dock_viewport.x,
        layout.dock_viewport.y,
        layout.dock_viewport.width,
        layout.dock_viewport.height,
    );
    context.clip();

    // ── Per-card: tilted border + app badge ───────────────────────────────────
    // Each card is rotated ~4° counter-clockwise around its own centre to give
    // a "leaning left" perspective depth effect.
    const TILT_ANGLE: f64 = -0.07; // radians ≈ –4°

    for (entry_idx, item) in &layout.dock_items {
        let is_selected = *entry_idx == switcher.selected_index;

        // Rotate each card individually around its centre.
        context.save()?;
        let cx = item.card.x + item.card.width / 2.0;
        let cy = item.card.y + item.card.height / 2.0;
        context.translate(cx, cy);
        context.rotate(TILT_ANGLE);
        context.translate(-cx, -cy);

        if let Some(entry) = switcher.entries.get(*entry_idx) {
            let (r, g, b) = app_color(&entry.app_name);

            if is_selected {
                // Active: single crisp app-colored border (no glow ring).
                rounded_rect(
                    context,
                    item.card.x,
                    item.card.y,
                    item.card.width,
                    item.card.height,
                    item.radius,
                );
                context.set_source_rgba(
                    (r + 0.35).min(1.0),
                    (g + 0.35).min(1.0),
                    (b + 0.35).min(1.0),
                    0.95,
                );
                context.set_line_width(2.2);
                context.stroke()?;
            } else {
                // Inactive: subtle app-tinted border.
                rounded_rect(
                    context,
                    item.card.x,
                    item.card.y,
                    item.card.width,
                    item.card.height,
                    item.radius,
                );
                context.set_source_rgba(r, g, b, 0.38);
                context.set_line_width(1.2);
                context.stroke()?;
            }

            // App badge at bottom-left corner of card.
            let badge_size = (item.card.height * 0.32).clamp(20.0, 30.0);
            let badge_x = item.card.x + 4.0;
            let badge_y = item.card.y + item.card.height - badge_size - 4.0;
            draw_app_badge(context, entry, badge_x, badge_y, badge_size, 1.0)?;
        } else {
            draw_card_border(context, item.card, item.radius, is_selected, 1.0)?;
        }

        context.restore()?;
    }

    context.restore()?; // end dock clip

    // ── Scroll fade gradients ──────────────────────────────────────────────────
    // Anchored to the actual items list boundaries (not the full viewport) so
    // the dark band always sits right at the edge of the visible cards.
    if let (Some((_, first)), Some((_, last))) =
        (layout.dock_items.first(), layout.dock_items.last())
    {
        let items_top = first.card.y;
        let items_bottom = last.card.y + last.card.height;
        let fade_h = (DOCK_THUMBNAIL_HEIGHT * 0.22).clamp(14.0, 32.0);
        let vp = layout.dock_viewport;

        // Top fade: solid → transparent, anchored to first card's top edge.
        {
            let grad = cairo::LinearGradient::new(0.0, items_top, 0.0, items_top + fade_h);
            grad.add_color_stop_rgba(0.0, 0.04, 0.05, 0.08, 0.80);
            grad.add_color_stop_rgba(1.0, 0.04, 0.05, 0.08, 0.0);
            context.rectangle(vp.x, items_top, vp.width, fade_h);
            context.set_source(&grad)?;
            context.fill()?;
        }

        // Bottom fade: transparent → solid, anchored to last card's bottom edge.
        {
            let grad =
                cairo::LinearGradient::new(0.0, items_bottom - fade_h, 0.0, items_bottom);
            grad.add_color_stop_rgba(0.0, 0.04, 0.05, 0.08, 0.0);
            grad.add_color_stop_rgba(1.0, 0.04, 0.05, 0.08, 0.80);
            context.rectangle(vp.x, items_bottom - fade_h, vp.width, fade_h);
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
