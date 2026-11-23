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
