//! Visualizer bar widget for Dynamic Island media player

use gtk4::prelude::*;
use gtk4::{Box, Orientation};

/// Constructs the 3-bar audio visualizer indicator
pub fn build_visualizer_bars() -> Box {
    let visualizer_box = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(2)
        .css_classes(["notch-visualizer"])
        .valign(gtk4::Align::Center)
        .build();

    for h in [8, 14, 10] {
        let bar = Box::builder()
            .css_classes(["visualizer-bar"])
            .width_request(3)
            .height_request(h)
            .build();
        visualizer_box.append(&bar);
    }

    visualizer_box
}
