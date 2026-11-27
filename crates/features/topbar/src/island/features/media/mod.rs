//! Media Player Island feature — real MPRIS metadata via `playerctl`.
//!
//! Uses the same `IslandFeature::build(ctx)` lifecycle as every other feature.
//! Widget construction lives in `ui/view.rs`, metadata rendering in
//! `ui/render.rs`, polling in `poll.rs` and formatting in `format.rs`.

pub mod models;
pub mod service;
pub mod ui;

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::Instant;

use gtk4::prelude::*;

use crate::island::{IslandCtx, IslandFeature, PLAYER_CAPSULE_WIDTH};
use models::{merge_transient_metadata, parse_metadata, PlayerMeta};
use ui::MediaPopover;

pub const PRIORITY: u32 = 50;

pub struct MediaFeature {
    container: gtk4::Box,
    track_label: gtk4::Label,
    art: gtk4::Picture,
    popover: RefCell<Option<MediaPopover>>,
    latest: Rc<RefCell<Option<String>>>,
    last_raw: Option<String>,
    cached: PlayerMeta,
    player_active: bool,
    song_key: String,
    art_loaded: Cell<bool>,
    playing_state: Rc<Cell<bool>>,
    play_state: Cell<Option<bool>>,
    pos_base: Cell<f64>,
    pos_at: Cell<Instant>,
    len_secs: Cell<f64>,
    view_ready: Cell<bool>,
    art_tx: tokio::sync::mpsc::UnboundedSender<(String, Vec<u8>)>,
    art_rx: Option<tokio::sync::mpsc::UnboundedReceiver<(String, Vec<u8>)>>,
    current_art_url: Rc<RefCell<String>>,
}

impl MediaFeature {
    pub fn new() -> Self {
        Self::build_widgets()
    }

    /// Parses cached metadata and requests show/hide through the shared context.
    fn refresh(&mut self, ctx: &IslandCtx) {
        let raw_changed = self.latest.borrow().as_deref() != self.last_raw.as_deref();
        if raw_changed {
            self.last_raw = self.latest.borrow().clone();
            let (metadata, active) = self
                .last_raw
                .as_deref()
                .map(parse_metadata)
                .unwrap_or_default();
            self.cached = merge_transient_metadata(&self.cached, metadata);
            self.player_active = active;
            self.pos_base.set(self.cached.pos_secs);
            self.pos_at.set(Instant::now());
            self.len_secs.set(self.cached.len_secs);
        }

        let popover_open = self
            .popover
            .borrow()
            .as_ref()
            .is_some_and(|popover| popover.is_visible());
        let playing = self.player_active && self.cached.playing;
        self.playing_state.set(playing);

        // Keep the island visible while paused too, so controls stay reachable.
        if self.player_active || popover_open || ctx.is_current() {
            ctx.show();
        } else {
            ctx.hide();
        }

        if raw_changed || !self.view_ready.get() {
            self.update_view();
            self.view_ready.set(true);
        }
        if playing && self.cached.art_url.is_empty() {
            self.art_loaded.set(false);
        }
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
        (PLAYER_CAPSULE_WIDTH, crate::island::CAPSULE_HEIGHT)
    }

    fn capsule_class(&self) -> Option<&'static str> {
        Some("active-music")
    }

    fn is_alive(&self) -> bool {
        self.player_active
    }

    fn is_popover_open(&self) -> bool {
        self.popover
            .borrow()
            .as_ref()
            .is_some_and(|popover| popover.is_visible())
    }

    fn build(&mut self, ctx: &IslandCtx) -> gtk4::Widget {
        let popover = MediaPopover::new(&ctx.capsule());
        self.start_art_receiver(popover.art_container.clone());
        *self.popover.borrow_mut() = Some(popover);
        self.container.clone().upcast()
    }

    fn tick(&mut self, ctx: &IslandCtx) {
        self.refresh(ctx);
        self.update_progress();
    }

    fn on_click(&mut self) {
        if let Some(popover) = self.popover.borrow().as_ref() {
            popover.toggle();
        }
    }
}
