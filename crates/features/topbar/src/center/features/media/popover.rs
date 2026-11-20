//! Glassmorphic media control popover anchored to the Dynamic Island capsule.
//! Layout and lifecycle copied from the old `babydra-island` implementation.

use std::ops::Deref;

use gtk4::prelude::*;
use ui_kit::icons::get_icon;

use crate::center::ui::IslandPopover;

use super::poll::playerctl;

#[derive(Clone)]
pub struct MediaPopover {
    pub base: IslandPopover,
    pub title: gtk4::Label,
    pub artist: gtk4::Label,
    pub art_container: gtk4::Picture,
    pub app_name: gtk4::Label,
    pub play_btn_icon: gtk4::Image,
    pub progress_container: gtk4::Box,
    pub progress_bar: gtk4::ProgressBar,
    pub position_lbl: gtk4::Label,
    pub length_lbl: gtk4::Label,
}

impl Deref for MediaPopover {
    type Target = IslandPopover;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl MediaPopover {
    pub fn new(capsule: &gtk4::Box) -> Self {
        let base = IslandPopover::new_slide(capsule, "media-popover", "media-popover-box", 450);

        let header = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
        header.add_css_class("media-popover-header");
        header.set_valign(gtk4::Align::Center);
        let app_name = gtk4::Label::builder()
            .label("Music Player")
            .css_classes(["media-popover-app-name", "tile-title"])
            .build();
        header.append(&get_icon("logo", 14));
        header.append(&app_name);
        base.popover_box.append(&header);

        let art_container = gtk4::Picture::new();
        art_container.add_css_class("media-popover-art");
        art_container.set_width_request(120);
        base.popover_box.append(&art_container);

        let title = gtk4::Label::builder()
            .label("Unknown title")
            .css_classes(["media-popover-title"])
            .halign(gtk4::Align::Center)
            .justify(gtk4::Justification::Center)
            .wrap(true)
            .max_width_chars(25)
            .build();
        let artist = gtk4::Label::builder()
            .label("Unknown artist")
            .css_classes(["media-popover-artist", "tile-subtitle"])
            .halign(gtk4::Align::Center)
            .justify(gtk4::Justification::Center)
            .wrap(true)
            .max_width_chars(30)
            .build();
        base.popover_box.append(&title);
        base.popover_box.append(&artist);

        let progress_container = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Vertical)
            .spacing(4)
            .css_classes(["media-popover-progress-container"])
            .margin_start(16)
            .margin_end(16)
            .margin_top(10)
            .margin_bottom(6)
            .build();
        let progress_bar = gtk4::ProgressBar::builder()
            .fraction(0.0)
            .css_classes(["media-popover-progress-bar"])
            .build();
        let time_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        time_box.add_css_class("media-popover-time-box");
        let position_lbl = gtk4::Label::builder()
            .label("0:00")
            .css_classes(["media-popover-time-label"])
            .halign(gtk4::Align::Start)
            .build();
        let spacer = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        spacer.set_hexpand(true);
        let length_lbl = gtk4::Label::builder()
            .label("0:00")
            .css_classes(["media-popover-time-label"])
            .halign(gtk4::Align::End)
            .build();
        time_box.append(&position_lbl);
        time_box.append(&spacer);
        time_box.append(&length_lbl);
        progress_container.append(&progress_bar);
        progress_container.append(&time_box);
        base.popover_box.append(&progress_container);

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

        let play_btn_icon = get_icon("play", 22);
        let play = gtk4::Button::new();
        play.add_css_class("media-control-btn");
        play.set_child(Some(&play_btn_icon));
        play.connect_clicked(|_| playerctl("play-pause"));
        controls.append(&play);

        let next = gtk4::Button::new();
        next.add_css_class("media-control-btn");
        next.set_child(Some(&get_icon("next", 16)));
        next.connect_clicked(|_| playerctl("next"));
        controls.append(&next);
        base.popover_box.append(&controls);

        Self {
            base,
            title,
            artist,
            art_container,
            app_name,
            play_btn_icon,
            progress_container,
            progress_bar,
            position_lbl,
            length_lbl,
        }
    }
}
