//! Screen / Audio recording details popover card

use gtk4::prelude::*;
use gtk4::{Box, Button, Label, Orientation};
use ui_kit::icons::get_icon_colored;

/// Builds the popover card layout for active recording
pub fn build_recording_popover() -> Box {
    let popover_box = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(10)
        .css_classes(["recording-popover-box"])
        .width_request(320)
        .build();

    // 1. Header with title & status badge
    let header_box = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(8)
        .css_classes(["recording-popover-header"])
        .build();

    let rec_icon = get_icon_colored("media-record-symbolic", 14, "#ef4444");
    rec_icon.add_css_class("recording-header-icon");
    header_box.append(&rec_icon);

    let title = Label::builder()
        .label("SCREEN RECORDER")
        .css_classes(["recording-header-title"])
        .hexpand(true)
        .xalign(0.0)
        .build();
    header_box.append(&title);

    let status_badge = Label::builder()
        .label("RECORDING")
        .css_classes(["recording-status-badge", "badge-recording"])
        .build();
    header_box.append(&status_badge);

    popover_box.append(&header_box);

    // 2. Hero Timer
    let timer_box = Box::builder()
        .orientation(Orientation::Horizontal)
        .halign(gtk4::Align::Center)
        .css_classes(["recording-timer-container"])
        .build();

    let hero_timer = Label::builder()
        .label("00:00")
        .css_classes(["recording-hero-timer"])
        .build();
    timer_box.append(&hero_timer);
    popover_box.append(&timer_box);

    // 3. Settings / Source Meta Card
    let meta_card = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(4)
        .css_classes(["recording-settings-card"])
        .build();

    let row1 = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(8)
        .css_classes(["recording-setting-row"])
        .build();
    let display_lbl = Label::builder()
        .label("Target")
        .css_classes(["recording-setting-label"])
        .hexpand(true)
        .xalign(0.0)
        .build();
    let area_val = Label::builder()
        .label("Entire Screen")
        .css_classes(["recording-area-label"])
        .build();
    row1.append(&display_lbl);
    row1.append(&area_val);
    meta_card.append(&row1);

    popover_box.append(&meta_card);

    // 4. Action Row (Stop Button)
    let action_row = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(8)
        .css_classes(["recording-action-row"])
        .build();

    let stop_btn = Button::builder()
        .label("Stop Recording")
        .hexpand(true)
        .css_classes([
            "btn-primary",
            "recording-action-stop",
            "recorder-action-btn",
            "destructive-action",
        ])
        .build();
    action_row.append(&stop_btn);
    popover_box.append(&action_row);

    popover_box
}
