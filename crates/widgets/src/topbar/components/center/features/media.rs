//! Media Player Feature for Dynamic Island

use gtk4::prelude::*;
use gtk4::{Box, Button, Label, Orientation, ProgressBar};
use std::cell::Cell;
use std::rc::Rc;
use ui_kit::CircularDisc;

use super::IslandFeature;

pub struct MediaFeature {
    container: Box,
    is_playing: Rc<Cell<bool>>,
    title_label: Label,
}

impl MediaFeature {
    pub fn new() -> Self {
        let container = Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(8)
            .valign(gtk4::Align::Center)
            .halign(gtk4::Align::Center)
            .css_classes(["notch-content", "active-music"])
            .build();

        let disc = CircularDisc::new("♫");
        container.append(disc.widget());

        let title_label = Label::builder()
            .label("Music Player — Ready")
            .css_classes(["notch-player-text"])
            .build();
        container.append(&title_label);

        // Animated visualizer bars
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
        container.append(&visualizer_box);

        Self {
            container,
            is_playing: Rc::new(Cell::new(true)),
            title_label,
        }
    }

    pub fn set_track(&self, title: &str, artist: &str) {
        self.title_label.set_label(&format!("{title} • {artist}"));
    }
}

impl Default for MediaFeature {
    fn default() -> Self {
        Self::new()
    }
}

impl IslandFeature for MediaFeature {
    fn id(&self) -> &'static str {
        "media"
    }

    fn priority(&self) -> u32 {
        10
    }

    fn is_active(&self) -> bool {
        self.is_playing.get()
    }

    fn widget(&self) -> gtk4::Widget {
        self.container.clone().upcast()
    }

    fn details_popover_content(&self) -> Option<gtk4::Widget> {
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

        // Controls: Prev | Play/Pause | Next
        let controls_box = Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(12)
            .halign(gtk4::Align::Center)
            .css_classes(["media-popover-controls"])
            .build();

        let prev_btn = Button::builder().label("󰒮").css_classes(["media-control-btn"]).build();
        let play_btn = Button::builder().label("󰏤").css_classes(["media-control-btn", "play-pause-btn"]).build();
        let next_btn = Button::builder().label("󰒭").css_classes(["media-control-btn"]).build();

        controls_box.append(&prev_btn);
        controls_box.append(&play_btn);
        controls_box.append(&next_btn);
        popover_box.append(&controls_box);

        Some(popover_box.upcast())
    }
}

