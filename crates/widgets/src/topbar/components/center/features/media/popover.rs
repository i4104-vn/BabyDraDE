//! Media player details popover card

use gtk4::prelude::*;
use gtk4::{Box, Button, Label, Orientation, ProgressBar};
use ui_kit::icons::get_icon;

/// Builds the popover card layout for active media playback
pub fn build_media_popover() -> Box {
    let popover_box = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(10)
        .css_classes(["media-popover-box"])
        .width_request(240)
        .build();

    let app_name = Label::builder()
        .label("Now Playing")
        .css_classes(["media-popover-app-name", "tile-title"])
        .xalign(0.0)
        .build();
    popover_box.append(&app_name);

    let title = Label::builder()
        .label("Cosmic Dreams")
        .css_classes(["media-popover-title"])
        .xalign(0.0)
        .build();
    popover_box.append(&title);

    let artist = Label::builder()
        .label("BabyDra Ambient")
        .css_classes(["media-popover-artist", "tile-subtitle"])
        .xalign(0.0)
        .build();
    popover_box.append(&artist);

    let progress = ProgressBar::builder()
        .fraction(0.35)
        .css_classes(["media-popover-progress-bar"])
        .build();
    popover_box.append(&progress);

    // Playback controls: Previous | Play/Pause | Next
    let controls_box = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(12)
        .halign(gtk4::Align::Center)
        .css_classes(["media-popover-controls"])
        .build();

    let prev_icon = get_icon("previous", 14);
    let prev_btn = Button::builder()
        .css_classes(["media-control-btn"])
        .child(&prev_icon)
        .build();

    let pause_icon = get_icon("pause", 14);
    let play_btn = Button::builder()
        .css_classes(["media-control-btn", "play-pause-btn"])
        .child(&pause_icon)
        .build();

    let next_icon = get_icon("next", 14);
    let next_btn = Button::builder()
        .css_classes(["media-control-btn"])
        .child(&next_icon)
        .build();

    controls_box.append(&prev_btn);
    controls_box.append(&play_btn);
    controls_box.append(&next_btn);
    popover_box.append(&controls_box);

    popover_box
}
