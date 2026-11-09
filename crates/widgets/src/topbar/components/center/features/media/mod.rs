//! Media Player Feature for Dynamic Island

pub mod popover;
pub mod visualizer;

use gtk4::prelude::*;
use gtk4::{Box, Label, Orientation};
use std::cell::Cell;
use std::rc::Rc;
use ui_kit::CircularDisc;

use super::IslandFeature;
use popover::build_media_popover;
use visualizer::build_visualizer_bars;

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

        let visualizer_box = build_visualizer_bars();
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
        Some(build_media_popover().upcast())
    }
}
