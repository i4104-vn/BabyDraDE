//! Widget construction and artwork loading for the compact player view.

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::Instant;

use gdk_pixbuf::prelude::*;
use gtk4::prelude::*;

use super::super::service::spawn_playerctl_polling;
use super::visualizer::create_visualizer;
use super::super::MediaFeature;

impl MediaFeature {
    /// Builds the capsule widgets and initializes all feature state.
    pub(crate) fn build_widgets() -> Self {
        let container = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .spacing(8)
            .valign(gtk4::Align::Center)
            .halign(gtk4::Align::Center)
            .css_classes(["notch-content", "active-music"])
            .build();
        let art = gtk4::Picture::new();
        art.set_width_request(18);
        art.set_height_request(18);
        container.append(&art);

        let track_label = gtk4::Label::builder()
            .label("Music Player")
            .css_classes(["notch-player-text"])
            .ellipsize(gtk4::pango::EllipsizeMode::End)
            .max_width_chars(16)
            .hexpand(false)
            .build();
        container.append(&track_label);
        let (visualizer, playing_state) = create_visualizer();
        container.append(&visualizer);

        let (art_tx, art_rx) = tokio::sync::mpsc::unbounded_channel();

        Self {
            container,
            track_label,
            art,
            popover: RefCell::new(None),
            latest: spawn_playerctl_polling(),
            last_raw: None,
            cached: Default::default(),
            player_active: false,
            song_key: String::new(),
            art_loaded: Cell::new(false),
            playing_state,
            play_state: Cell::new(None),
            pos_base: Cell::new(0.0),
            pos_at: Cell::new(Instant::now()),
            len_secs: Cell::new(0.0),
            view_ready: Cell::new(false),
            art_tx,
            art_rx: Some(art_rx),
            current_art_url: Rc::new(RefCell::new(String::new())),
        }
    }

    /// Applies fetched artwork to the capsule and popover on the main context.
    pub(crate) fn start_art_receiver(&mut self, popover_art: gtk4::Picture) {
        let Some(mut receiver) = self.art_rx.take() else {
            return;
        };
        let capsule_art = self.art.clone();
        let current_url = self.current_art_url.clone();
        let loaded = self.art_loaded.clone();
        glib::MainContext::default().spawn_local(async move {
            while let Some((url, bytes)) = receiver.recv().await {
                if url != *current_url.borrow() {
                    continue;
                }
                let loader = gdk_pixbuf::PixbufLoader::new();
                if loader.write(&bytes).is_err() || loader.close().is_err() {
                    continue;
                }
                let Some(pixbuf) = loader.pixbuf() else {
                    continue;
                };
                if let Some(circle) = ui_kit::drawing::crop_circle(&bytes, 18) {
                    capsule_art.set_pixbuf(Some(&circle));
                } else {
                    capsule_art.set_pixbuf(Some(&pixbuf));
                }
                popover_art.set_pixbuf(Some(&pixbuf));
                loaded.set(true);
            }
        });
    }
}
