//! Media Player Island feature — real MPRIS metadata via `playerctl`.

pub mod model;
pub mod poll;
pub mod popover;
pub mod visualizer;

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use gdk_pixbuf::prelude::*;
use gtk4::prelude::*;

use crate::center::engine::{IslandCtx, IslandFeature, IslandViewHandle, PLAYER_CAPSULE_WIDTH};
use model::{format_time, merge_transient_metadata, parse_metadata, PlayerMeta};
use poll::spawn_playerctl_polling;
use popover::MediaPopover;
use visualizer::create_visualizer;

pub const PRIORITY: u32 = 50;

pub struct MediaFeature {
    container: gtk4::Box,
    track_label: gtk4::Label,
    art: gtk4::Picture,
    popover: RefCell<Option<MediaPopover>>,
    handle: RefCell<Option<IslandViewHandle>>,
    latest: Rc<RefCell<Option<String>>>,
    last_raw: Option<String>,
    cached: PlayerMeta,
    player_active: bool,
    song_key: String,
    art_loaded: Cell<bool>,
    playing_state: Rc<Cell<bool>>,
    play_state: Cell<Option<bool>>,
    view_ready: Cell<bool>,
    art_tx: tokio::sync::mpsc::UnboundedSender<(String, Vec<u8>)>,
    art_rx: Option<tokio::sync::mpsc::UnboundedReceiver<(String, Vec<u8>)>>,
    current_art_url: Rc<RefCell<String>>,
}

impl MediaFeature {
    pub fn new() -> Self {
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
            handle: RefCell::new(None),
            latest: spawn_playerctl_polling(),
            last_raw: None,
            cached: PlayerMeta::default(),
            player_active: false,
            song_key: String::new(),
            art_loaded: Cell::new(false),
            playing_state,
            play_state: Cell::new(None),
            view_ready: Cell::new(false),
            art_tx,
            art_rx: Some(art_rx),
            current_art_url: Rc::new(RefCell::new(String::new())),
        }
    }

    fn refresh(&mut self, ctx: &IslandCtx) {
        let raw_changed = self.latest.borrow().as_deref() != self.last_raw.as_deref();
        if raw_changed {
            self.last_raw = self.latest.borrow().clone();
            let (meta, player_active) = self
                .last_raw
                .as_deref()
                .map(parse_metadata)
                .unwrap_or_default();
            self.cached = merge_transient_metadata(&self.cached, meta);
            self.player_active = player_active;
        }

        let popover_open = self
            .popover
            .borrow()
            .as_ref()
            .is_some_and(|popover| popover.is_visible());
        let playing = self.player_active && self.cached.playing;
        self.playing_state.set(playing);

        // Show the island whenever a MPRIS player exists — including while
        // paused, so the track and controls stay reachable.
        if let Some(handle) = self.handle.borrow().as_ref() {
            if self.player_active || popover_open || ctx.is_current() {
                handle.show();
            } else {
                handle.hide();
            }
        }

        if raw_changed || !self.view_ready.get() {
            self.update_view();
            self.view_ready.set(true);
        }

        if playing && self.cached.art_url.is_empty() {
            self.art_loaded.set(false);
        }
    }

    fn update_view(&mut self) {
        let meta = &self.cached;
        let base = if meta.title.is_empty() {
            if meta.artist.is_empty() {
                "Music Player".to_string()
            } else {
                meta.artist.clone()
            }
        } else if meta.artist.is_empty() {
            meta.title.clone()
        } else {
            format!("{} - {}", meta.artist, meta.title)
        };
        let display = if self.player_active && !self.cached.playing {
            format!("{base}")
        } else {
            base
        };
        let short: String = if display.chars().count() > 18 {
            display.chars().take(15).collect::<String>() + "..."
        } else {
            display
        };
        self.track_label.set_label(&short);

        let song_key = format!(
            "{}|{}|{}|{}",
            meta.title, meta.artist, meta.player_name_raw, meta.art_url
        );
        let song_changed = song_key != self.song_key;
        if !song_changed {
            return;
        }
        self.song_key = song_key;
        self.art_loaded.set(false);

        if let Some(popover) = self.popover.borrow().as_ref() {
            popover.title.set_text(if meta.title.is_empty() {
                "Unknown title"
            } else {
                &meta.title
            });
            popover.artist.set_text(if meta.artist.is_empty() {
                "Unknown artist"
            } else {
                &meta.artist
            });
            let player = if meta.player_name_raw.is_empty() {
                "Music Player".to_string()
            } else {
                let mut chars = meta.player_name_raw.chars();
                chars
                    .next()
                    .map(|c| c.to_uppercase().collect::<String>() + chars.as_str())
                    .unwrap_or_default()
            };
            popover.app_name.set_text(&player);

            if meta.len_secs > 0.0 {
                popover
                    .progress_bar
                    .set_fraction((meta.pos_secs / meta.len_secs).clamp(0.0, 1.0));
                popover.position_lbl.set_text(&format_time(meta.pos_secs));
                popover.length_lbl.set_text(&format_time(meta.len_secs));
            }
        }

        if let Some(popover) = self.popover.borrow().as_ref() {
            if self.play_state.get() != Some(meta.playing) {
                let icon = if meta.playing { "pause" } else { "play" };
                ui_kit::icons::set_image_from_icon(&popover.play_btn_icon, icon, 22);
                self.play_state.set(Some(meta.playing));
            }
        }

        if !meta.art_url.is_empty() {
            *self.current_art_url.borrow_mut() = meta.art_url.clone();
            let url = meta.art_url.clone();
            let tx = self.art_tx.clone();
            std::thread::spawn(move || {
                let bytes = if let Some(path) = url.strip_prefix("file://") {
                    std::fs::read(path).ok()
                } else if url.starts_with('/') {
                    std::fs::read(&url).ok()
                } else if url.starts_with("http") {
                    std::process::Command::new("curl")
                        .args(["-s", "-L", "--max-time", "5", &url])
                        .output()
                        .ok()
                        .filter(|out| out.status.success())
                        .map(|out| out.stdout)
                } else {
                    Some(Vec::new())
                };
                if let Some(bytes) = bytes {
                    let _ = tx.send((url, bytes));
                }
            });
        }
    }

    fn start_art_receiver(&mut self, popover_art: gtk4::Picture) {
        let Some(mut rx) = self.art_rx.take() else {
            return;
        };
        let capsule_art = self.art.clone();
        let current_url = self.current_art_url.clone();
        let loaded = self.art_loaded.clone();
        glib::MainContext::default().spawn_local(async move {
            while let Some((url, bytes)) = rx.recv().await {
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
                capsule_art.set_pixbuf(Some(&pixbuf));
                popover_art.set_pixbuf(Some(&pixbuf));
                loaded.set(true);
            }
        });
    }
}

impl Default for MediaFeature {
    fn default() -> Self {
        Self::new()
    }
}

impl IslandFeature for MediaFeature {
    fn id(&self) -> &'static str {
        "media_player"
    }

    fn priority(&self) -> u32 {
        PRIORITY
    }

    fn size(&self) -> (i32, i32) {
        (PLAYER_CAPSULE_WIDTH, crate::center::engine::CAPSULE_HEIGHT)
    }

    fn capsule_class(&self) -> Option<&'static str> {
        Some("active-music")
    }

    fn is_alive(&self) -> bool {
        self.player_active
    }

    fn is_popover_open(&self) -> bool {
        self.popover.borrow().as_ref().is_some_and(|p| p.is_visible())
    }

    fn widget(&self) -> gtk4::Widget {
        self.container.clone().upcast()
    }

    fn init(&mut self, handle: IslandViewHandle) {
        *self.handle.borrow_mut() = Some(handle);
    }

    fn attach(&mut self, capsule: &gtk4::Box) {
        let popover = MediaPopover::new(capsule);
        self.start_art_receiver(popover.art_container.clone());
        // Store the real picture widget instead of the placeholder box.
        *self.popover.borrow_mut() = Some(popover);
    }

    fn tick(&mut self, ctx: &IslandCtx) {
        self.refresh(ctx);
    }

    fn on_click(&mut self) {
        if let Some(popover) = self.popover.borrow().as_ref() {
            popover.toggle();
        }
    }
}
