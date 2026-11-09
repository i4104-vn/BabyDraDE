//! Screen / Audio Recording Feature for Dynamic Island

pub mod popover;

use gtk4::prelude::*;
use gtk4::{Box, Label, Orientation};
use std::cell::Cell;
use std::rc::Rc;

use super::IslandFeature;
use popover::build_recording_popover;

pub struct RecordingFeature {
    container: Box,
    is_recording: Rc<Cell<bool>>,
    timer_label: Label,
}

impl RecordingFeature {
    pub fn new() -> Self {
        let container = Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(6)
            .valign(gtk4::Align::Center)
            .halign(gtk4::Align::Center)
            .css_classes(["notch-content", "recording-notch"])
            .build();

        // Pulsing red dot
        let red_dot = Label::builder()
            .label("⏺")
            .css_classes(["recording-dot", "status-icon"])
            .build();
        container.append(&red_dot);

        let label = Label::builder()
            .label("REC")
            .css_classes(["recording-title", "badge-title"])
            .build();
        container.append(&label);

        let timer_label = Label::builder()
            .label("00:00")
            .css_classes(["recording-timer", "notch-system-value"])
            .build();
        container.append(&timer_label);

        Self {
            container,
            is_recording: Rc::new(Cell::new(false)),
            timer_label,
        }
    }

    pub fn set_duration(&self, duration_text: &str) {
        self.timer_label.set_label(duration_text);
    }

    pub fn start_recording(&self) {
        self.is_recording.set(true);
    }

    pub fn stop_recording(&self) {
        self.is_recording.set(false);
    }
}

impl Default for RecordingFeature {
    fn default() -> Self {
        Self::new()
    }
}

impl IslandFeature for RecordingFeature {
    fn id(&self) -> &'static str {
        "recording"
    }

    fn priority(&self) -> u32 {
        50
    }

    fn is_active(&self) -> bool {
        self.is_recording.get()
    }

    fn widget(&self) -> gtk4::Widget {
        self.container.clone().upcast()
    }

    fn details_popover_content(&self) -> Option<gtk4::Widget> {
        Some(build_recording_popover().upcast())
    }
}
