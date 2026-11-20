use cairo::{Context, Operator};

use crate::compositor::AltTabSwitcher;

use super::draw::{draw_app_badge, rounded_rect, Rect};

pub(crate) use crate::enums::constants::DOCK_MAX_VISIBLE_ITEMS;
use crate::enums::constants::{
    DOCK_BOTTOM_MARGIN, DOCK_GAP, DOCK_THUMBNAIL_HEIGHT, DOCK_THUMBNAIL_WIDTH, DOCK_TOP_MARGIN,
    DOCK_X, STAGE_ASPECT_RATIO, STAGE_RIGHT_MARGIN, STAGE_SIDE_GAP, STAGE_SLIDE_DISTANCE,
};

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
    /// Khung target cố định ở giữa dock. Không di chuyển, không đổi màu —
    /// các item cuộn qua bên dưới nó.
    pub(crate) dock_target: Rect,
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
    let progress = switcher.stage_progress();

    // Dock is a continuous carousel: `visual_selection` may be fractional,
    // and entries are positioned relative to it. This makes the list scroll
    // smoothly instead of swapping content in fixed slots.
    // Với ít hơn 5 app, tất cả item đứng yên và khung active di chuyển tới
    // item được chọn thay vì cuộn.
    let visual_selection = switcher.visual_selection();
    let (dock_viewport, dock_items, dock_target) =
        compute_dock_layout_mode(n, sel, visual_selection, screen_h);
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
    // Chuyển app: stage hơi thu nhỏ, trượt nhẹ theo hướng điều hướng rồi trở
    // về đúng kích thước. Không đổi opacity để tránh cảm giác nhấp nháy.
    let stage_scale = 0.965 + 0.035 * progress;
    let stage_slide = switcher.scroll_direction * (1.0 - progress) * STAGE_SLIDE_DISTANCE;
    let stage_card = scale_rect_around(stage_card, stage_center, stage_scale);
    let stage_preview = scale_rect_around(stage_preview, stage_center, stage_scale);
    let stage_card = Rect::new(
        stage_card.x + stage_slide,
        stage_card.y,
        stage_card.width,
        stage_card.height,
    );
    let stage_preview = Rect::new(
        stage_preview.x + stage_slide,
        stage_preview.y,
        stage_preview.width,
        stage_preview.height,
    );
    let stage = (
        sel,
        CardLayout {
            card: stage_card,
            preview: stage_preview,
            radius: 18.0,
            // Không fade để tránh cảm giác nhấp nháy; hiệu ứng chuyển app
            // nằm ở chuyển động scale/trượt nhẹ phía trên.
            opacity: 1.0,
        },
    );

    Some(SwitcherLayout {
        stage,
        dock_viewport,
        dock_items,
        dock_target,
    })
}

struct DockMetrics {
    viewport: Rect,
    center_y: f64,
    item_step: f64,
    thumbnail_x: f64,
    thumbnail_width: f64,
    thumbnail_height: f64,
    thumb_pad: f64,
}

fn compute_dock_metrics(entry_count: usize, screen_h: f64) -> DockMetrics {
    let viewport_y = DOCK_TOP_MARGIN.min((screen_h - 1.0).max(0.0));
    let viewport_bottom = (screen_h - DOCK_BOTTOM_MARGIN).max(viewport_y + 1.0);
    let viewport = Rect::new(
        DOCK_X,
        viewport_y,
        DOCK_THUMBNAIL_WIDTH,
        viewport_bottom - viewport_y,
    );

    let visible_count = entry_count.clamp(1, DOCK_MAX_VISIBLE_ITEMS);
    let desired_height = visible_count as f64 * DOCK_THUMBNAIL_HEIGHT
        + visible_count.saturating_sub(1) as f64 * DOCK_GAP;
    let scale = (viewport.height / desired_height).min(1.0);
    let thumbnail_width = DOCK_THUMBNAIL_WIDTH * scale;
    let thumbnail_height = DOCK_THUMBNAIL_HEIGHT * scale;
    let gap = DOCK_GAP * scale;
    let item_step = thumbnail_height + gap;
    let thumb_pad = 3.0 * scale;
    let thumbnail_x = DOCK_X + (DOCK_THUMBNAIL_WIDTH - thumbnail_width) / 2.0;
    let center_y = viewport.y + viewport.height / 2.0;

    DockMetrics {
        viewport,
        center_y,
        item_step,
        thumbnail_x,
        thumbnail_width,
        thumbnail_height,
        thumb_pad,
    }
}

fn dock_card(metrics: &DockMetrics, slot: f64, opacity: f64) -> CardLayout {
    let card = Rect::new(
        metrics.thumbnail_x,
        metrics.center_y + slot * metrics.item_step - metrics.thumbnail_height / 2.0,
        metrics.thumbnail_width,
        metrics.thumbnail_height,
    );
    let preview = Rect::new(
        card.x + metrics.thumb_pad,
        card.y + metrics.thumb_pad,
        (card.width - 2.0 * metrics.thumb_pad).max(0.0),
        (card.height - 2.0 * metrics.thumb_pad).max(0.0),
    );

    CardLayout {
        card,
        preview,
        radius: (metrics.thumbnail_height * 0.13).clamp(4.0, 14.0),
        opacity,
    }
}

fn compute_dock_items(
    metrics: &DockMetrics,
    entry_count: usize,
    center: f64,
) -> Vec<(usize, CardLayout)> {
    let count = entry_count as i64;
    let base = center.floor() as i64;
    let mut items = Vec::new();

    // Enumerate by slot (not by entry) around the carousel centre. With a
    // circular list of exactly 5 entries the item leaving the top and the one
    // entering the bottom are the same entry; generating slots independently
    // lets both copies exist at once, so the wrap is seamless instead of
    // teleporting across the dock.
    for offset in -3..=3 {
        let unwrapped = base + offset;
        let index = unwrapped.rem_euclid(count) as usize;
        let slot = unwrapped as f64 - center;
        // Năm slot lõi nằm trong [-2, 2]. Item mới đi vào / item cũ đi ra
        // mờ dần trong đúng một slot kế tiếp, nên không bao giờ "pop"
        // hoặc nhấp nháy khi bắt đầu/kết thúc cuộn.
        let fade = (3.0 - slot.abs()).clamp(0.0, 1.0);
        if fade <= 0.001 {
            continue;
        }
        items.push((index, dock_card(metrics, slot, fade)));
    }

    items.sort_by(|a, b| a.1.card.y.total_cmp(&b.1.card.y));
    items
}

/// Layout tĩnh cho trường hợp dưới 5 app: mọi item đứng yên, nhóm được căn
/// giữa dock theo thứ tự 0..n.
fn compute_static_dock_items(
    metrics: &DockMetrics,
    entry_count: usize,
) -> Vec<(usize, CardLayout)> {
    let center_slot = (entry_count as f64 - 1.0) / 2.0;
    (0..entry_count)
        .map(|index| (index, dock_card(metrics, index as f64 - center_slot, 1.0)))
        .collect()
}

fn compute_dock_layout_static(
    entry_count: usize,
    selected_index: usize,
    screen_h: f64,
) -> (Rect, Vec<(usize, CardLayout)>, Rect) {
    if entry_count == 0 {
        let empty = Rect::new(DOCK_X, 0.0, DOCK_THUMBNAIL_WIDTH, 0.0);
        return (empty, Vec::new(), empty);
    }

    let metrics = compute_dock_metrics(entry_count, screen_h);
    let items = compute_static_dock_items(&metrics, entry_count);
    let target = items
        .iter()
        .find(|(index, _)| *index == selected_index)
        .map(|(_, item)| item.preview)
        .unwrap_or_else(|| dock_card(&metrics, 0.0, 1.0).preview);

    (metrics.viewport, items, target)
}

/// Chọn chế độ dock: từ 5 app trở lên cuộn như carousel, dưới 5 app thì
/// hiển thị cố định và chỉ di chuyển khung active.
fn compute_dock_layout_mode(
    entry_count: usize,
    selected_index: usize,
    visual_selection: f64,
    screen_h: f64,
) -> (Rect, Vec<(usize, CardLayout)>, Rect) {
    if entry_count >= DOCK_MAX_VISIBLE_ITEMS {
        compute_dock_layout_animated(entry_count, visual_selection, screen_h)
    } else {
        compute_dock_layout_static(entry_count, selected_index, screen_h)
    }
}

fn compute_dock_layout_animated(
    entry_count: usize,
    center: f64,
    screen_h: f64,
) -> (Rect, Vec<(usize, CardLayout)>, Rect) {
    if entry_count == 0 {
        let empty = Rect::new(DOCK_X, 0.0, DOCK_THUMBNAIL_WIDTH, 0.0);
        return (empty, Vec::new(), empty);
    }

    let metrics = compute_dock_metrics(entry_count, screen_h);
    let items = compute_dock_items(&metrics, entry_count, center);
    // Target luôn ở slot 0 (giữa) và không phụ thuộc vào animation.
    let target = dock_card(&metrics, 0.0, 1.0).preview;

    (metrics.viewport, items, target)
}

#[cfg(test)]
fn compute_dock_layout(
    entry_count: usize,
    selected_index: usize,
    screen_h: f64,
) -> (Rect, Vec<(usize, CardLayout)>) {
    let (viewport, items, _) =
        compute_dock_layout_mode(entry_count, selected_index, selected_index as f64, screen_h);
    (viewport, items)
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

    context.save()?;
    context.rectangle(vp.x, vp.y, vp.width, vp.height);
    context.clip();

    let target = layout.dock_target;
    let target_radius = target.height * 0.10;

    // Vùng target tĩnh: nền rất nhẹ để mắt bám vào đích đến, nhưng không đổi
    // màu và không di chuyển nên không gây nhấp nháy.
    context.save()?;
    rounded_rect(
        context,
        target.x,
        target.y,
        target.width,
        target.height,
        target_radius,
    );
    context.set_source_rgba(1.0, 1.0, 1.0, 0.045);
    context.fill()?;
    context.restore()?;

    // Các card trượt qua bên dưới target.
    for (entry_idx, item) in &layout.dock_items {
        let opacity = item.opacity;
        if opacity <= 0.01 {
            continue;
        }

        let c = item.card;
        let p = item.preview;

        if let Some(entry) = switcher.entries.get(*entry_idx) {
            // Nền card bo tròn, lấp đầy toàn bộ vùng item.
            context.save()?;
            rounded_rect(context, c.x, c.y, c.width, c.height, c.height * 0.12);
            context.set_source_rgba(0.14, 0.14, 0.18, 0.60 * opacity);
            context.fill()?;
            context.restore()?;

            // Badge bo góc, gắn sát góc dưới-trái của preview, không padding thừa.
            let badge_size = (c.height * 0.30).clamp(20.0, 30.0);
            draw_app_badge(
                context,
                entry,
                p.x,
                p.y + p.height - badge_size,
                badge_size,
                0.85 * opacity,
            )?;
        }
    }

    // Viền target luôn nằm trên cùng, cố định ở giữa.
    context.save()?;
    rounded_rect(
        context,
        target.x,
        target.y,
        target.width,
        target.height,
        target_radius,
    );
    context.set_source_rgba(0.88, 0.92, 1.0, 0.90);
    context.set_line_width(2.0);
    context.stroke()?;
    context.restore()?;

    context.restore()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use animation::Tween;

    use super::*;

    #[test]
    fn test_compute_layout_empty() {
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

    #[test]
    fn dock_scroll_moves_entries_up_by_one_slot() {
        let (_, before, _) = compute_dock_layout_animated(30, 12.0, 1080.0);
        let (_, after, _) = compute_dock_layout_animated(30, 13.0, 1080.0);
        let y_before = before.iter().find(|(idx, _)| *idx == 13).unwrap().1.card.y;
        let y_after = after.iter().find(|(idx, _)| *idx == 13).unwrap().1.card.y;

        // Ở 1080p, 5 item vừa đủ nên scale = 1: bước = 115 + 12.
        assert!((y_before - y_after - 127.0).abs() < 0.001);
    }

    #[test]
    fn target_frame_stays_fixed_while_dock_scrolls() {
        let (_, _, target_at_rest) = compute_dock_layout_animated(30, 12.0, 1080.0);
        let (_, _, target_mid_scroll) = compute_dock_layout_animated(30, 12.5, 1080.0);

        assert_eq!(target_at_rest, target_mid_scroll);
    }

    #[test]
    fn entering_item_fades_instead_of_popping() {
        let (_, items, _) = compute_dock_layout_animated(30, 12.5, 1080.0);
        let (_, entering) = items
            .iter()
            .find(|(idx, _)| *idx == 15)
            .expect("item below the target should be entering");

        assert!(entering.opacity > 0.0 && entering.opacity < 1.0);
    }

    #[test]
    fn fewer_than_five_items_stay_static_and_active_frame_moves() {
        let (_, items_first, target_first) = compute_dock_layout_mode(3, 0, 0.0, 1080.0);
        let (_, items_last, target_last) = compute_dock_layout_mode(3, 2, 2.0, 1080.0);

        // Vị trí từng item không đổi khi chọn item khác.
        for ((first_index, first), (last_index, last)) in items_first.iter().zip(items_last.iter())
        {
            assert_eq!(first_index, last_index);
            assert_eq!(first.card, last.card);
            assert_eq!(first.preview, last.preview);
        }

        // Khung active di chuyển tới item đang chọn.
        assert_eq!(target_first, items_first[0].1.preview);
        assert_eq!(target_last, items_last[2].1.preview);
        assert_ne!(target_first, target_last);
    }

    #[test]
    fn static_dock_group_is_vertically_centered() {
        let (viewport, items, _) = compute_dock_layout_mode(3, 1, 1.0, 1080.0);
        let first = items.first().unwrap().1.card;
        let last = items.last().unwrap().1.card;
        let group_center = (first.y + last.y + last.height) / 2.0;
        let viewport_center = viewport.y + viewport.height / 2.0;

        assert!((group_center - viewport_center).abs() < 0.001);
    }

    #[test]
    fn five_items_keep_the_scrolling_carousel_with_a_fixed_target() {
        let (_, _, target_before) = compute_dock_layout_mode(5, 0, 0.0, 1080.0);
        let (_, _, target_after) = compute_dock_layout_mode(5, 1, 1.0, 1080.0);

        // Với 5 app, target vẫn cố định ở giữa và item sẽ cuộn qua.
        assert_eq!(target_before, target_after);
    }

    #[test]
    fn five_app_wrap_renders_both_edge_copies_mid_scroll() {
        // Đúng 5 app: item rời mép trên và item vào mép dưới là cùng một item.
        // Ở giữa bước cuộn phải có 2 bản sao để không bị teleport.
        let (_, items, _) = compute_dock_layout_animated(5, 0.5, 1080.0);
        let copies = items
            .iter()
            .filter(|(index, _)| *index == 3)
            .map(|(_, item)| item.card.y)
            .collect::<Vec<_>>();

        assert_eq!(copies.len(), 2);
        assert!(copies[0] < copies[1]);
    }

    #[test]
    fn five_apps_at_rest_show_exactly_five_unique_items() {
        let (_, items, _) = compute_dock_layout_animated(5, 0.0, 1080.0);
        let mut indices = items.iter().map(|(index, _)| *index).collect::<Vec<_>>();
        indices.sort_unstable();
        indices.dedup();

        assert_eq!(items.len(), 5);
        assert_eq!(indices.len(), 5);
        assert!(items.iter().all(|(_, item)| item.opacity >= 1.0));
    }
}
