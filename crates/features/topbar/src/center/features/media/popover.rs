//! Media control popover anchored to the Dynamic Island capsule.

use gtk4::prelude::*;
use ui_kit::icons::get_icon;

use super::poll::playerctl;

#[derive(Clone)]
pub struct MediaPopover {
    pub popover: gtk4::Popover,
    pub title: gtk4::Label,
    pub artist: gtk4::Label,
    pub app_name: gtk4::Label,
    pub play_icon: gtk4::Image,
    pub art_container: gtk4::Picture,
    pub progress: gtk4::ProgressBar,
    pub position_lbl: gtk4::Label,
    pub length_lbl: gtk4::Label,
}

impl MediaPopover {
    pub fn new(capsule: &gtk4::Box) -> Self {
        let popover = gtk4::Popover::builder()
            .autohide(true)
            .has_arrow(false)
            .position(gtk4::PositionType::Bottom)
            .css_classes(["media-popover", "island-popover"])
            .build();
        popover.set_parent(capsule);

        let card = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Vertical)
            .spacing(10)
            .css_classes(["media-popover-box"])
            .width_request(240)
            .build();
        popover.set_child(Some(&card));

        let header = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
        header.add_css_class("media-popover-header");
        let app_name = gtk4::Label::builder()
            .label("Music Player")
            .css_classes(["media-popover-app-name", "tile-title"])
            .xalign(0.0)
            .build();
        header.append(&get_icon("music", 14));
        header.append(&app_name);
        card.append(&header);

        let art_container = gtk4::Picture::new();
        art_container.add_css_class("media-popover-art");
        art_container.set_width_request(120);
        card.append(&art_container);

        let title = gtk4::Label::builder()
            .label("Unknown title")
            .css_classes(["media-popover-title"])
            .wrap(true)
            .max_width_chars(25)
            .justify(gtk4::Justification::Center)
            .build();
        let artist = gtk4::Label::builder()
            .label("Unknown artist")
            .css_classes(["media-popover-artist", "tile-subtitle"])
            .wrap(true)
            .max_width_chars(30)
            .justify(gtk4::Justification::Center)
            .build();
        card.append(&title);
        card.append(&artist);

        let progress_box = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Vertical)
            .spacing(4)
            .css_classes(["media-popover-progress-container"])
            .margin_start(16)
            .margin_end(16)
            .margin_top(10)
            .margin_bottom(6)
            .build();
        let progress = gtk4::ProgressBar::builder()
            .fraction(0.0)
            .css_classes(["media-popover-progress-bar"])
            .build();
        let time_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        time_box.add_css_class("media-popover-time-box");
        let position_lbl = gtk4::Label::builder()
            .label("0:00")
            .css_classes(["media-popover-time-label"])
            .build();
        let spacer = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        spacer.set_hexpand(true);
        let length_lbl = gtk4::Label::builder()
            .label("0:00")
            .css_classes(["media-popover-time-label"])
            .build();
        time_box.append(&position_lbl);
        time_box.append(&spacer);
        time_box.append(&length_lbl);
        progress_box.append(&progress);
        progress_box.append(&time_box);
        card.append(&progress_box);

        let controls = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .spacing(18)
            .halign(gtk4::Align::Center)
            .css_classes(["media-popover-controls"])
            .build();
        let prev = gtk4::Button::new();
        prev.add_css_class("media-control-btn");
        prev.set_child(Some(&get_icon("previous", 16)));
        prev.connect_clicked(|_| playerctl("previous"));
        controls.append(&prev);

        let play_icon = get_icon("play", 22);
        let play = gtk4::Button::new();
        play.add_css_class("media-control-btn");
        play.set_child(Some(&play_icon));
        play.connect_clicked(|_| playerctl("play-pause"));
        controls.append(&play);

        let next = gtk4::Button::new();
        next.add_css_class("media-control-btn");
        next.set_child(Some(&get_icon("next", 16)));
        next.connect_clicked(|_| playerctl("next"));
        controls.append(&next);
        card.append(&controls);

        Self {
            popover,
            title,
            artist,
            app_name,
            play_icon,
            art_container,
            progress,
            position_lbl,
            length_lbl,
        }
    }

    pub fn is_visible(&self) -> bool {
        self.popover.is_visible()
    }

    pub fn toggle(&self) {
        if self.is_visible() {
            self.popover.popdown();
        } else {
            self.popover.popup();
        }
    }
}
