//! Screen / Audio recording details popover card

use gtk4::prelude::*;
use gtk4::{Box, Button, Label, Orientation};

/// Builds the popover card layout for active recording
pub fn build_recording_popover() -> Box {
    let popover_box = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(8)
        .css_classes(["recording-popover-box"])
        .width_request(200)
        .build();

    let title = Label::builder()
        .label("Screen Recording Active")
        .css_classes(["tile-title"])
        .build();
    popover_box.append(&title);

    let stop_btn = Button::builder()
        .label("Stop Recording")
        .css_classes(["destructive-action", "func-btn"])
        .build();
    popover_box.append(&stop_btn);

    popover_box
}
