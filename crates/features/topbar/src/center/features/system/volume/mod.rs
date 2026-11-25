//! System Volume indicator feature for the Dynamic Island.

pub mod service;

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::Duration;

use gtk4::prelude::*;

use crate::center::poller::spawn_poller;
use crate::center::{IslandCtx, IslandFeature, NotchWidget, CAPSULE_HEIGHT, CAPSULE_WIDTH};
use service::VolumeState;

pub const PRIORITY: u32 = 95;
pub const SHOW_DURATION: Duration = Duration::from_millis(1500);
const POLL_INTERVAL: Duration = Duration::from_millis(500);

fn icon_for(volume: f64, muted: bool) -> &'static str {
    if muted || volume == 0.0 {
        "volume-mute"
    } else if volume <= 45.0 {
        "volume-low"
    } else {
        "volume"
    }
}

fn label_for(volume: f64, muted: bool) -> String {
    if muted {
        "Mute".to_string()
    } else {
        format!("{:.0}%", volume.round())
    }
}

pub struct VolumeFeature {
    widgets: NotchWidget,
    last: Rc<RefCell<VolumeState>>,
    initialized: Rc<Cell<bool>>,
}

impl VolumeFeature {
    pub fn new() -> Self {
        let initial = service::read();
        Self {
            widgets: NotchWidget::with_value(
                icon_for(initial.volume, initial.muted),
                &services::i18n::trans("volume.title"),
                &label_for(initial.volume, initial.muted),
            ),
            last: Rc::new(RefCell::new(initial)),
            initialized: Rc::new(Cell::new(false)),
        }
    }
}

impl Default for VolumeFeature {
    fn default() -> Self {
        Self::new()
    }
}

impl IslandFeature for VolumeFeature {
    fn id(&self) -> &'static str {
        "system_volume"
    }

    fn priority(&self) -> u32 {
        PRIORITY
    }

    fn size(&self) -> (i32, i32) {
        (CAPSULE_WIDTH, CAPSULE_HEIGHT)
    }

    fn build(&mut self, ctx: &IslandCtx) -> gtk4::Widget {
        let widgets = self.widgets.clone();
        let last = self.last.clone();
        let initialized = self.initialized.clone();
        let ctx = ctx.clone();

        spawn_poller(
            "island-volume",
            POLL_INTERVAL,
            service::read,
            move |state| {
                let previous = *last.borrow();
                if !initialized.replace(true) {
                    *last.borrow_mut() = state;
                    return;
                }
                if (state.volume - previous.volume).abs() >= 0.5 || state.muted != previous.muted {
                    *last.borrow_mut() = state;
                    widgets.update(
                        icon_for(state.volume, state.muted),
                        &label_for(state.volume, state.muted),
                    );
                    ctx.override_show_for(SHOW_DURATION);
                    ctx.tick();
                }
            },
        );

        self.widgets.container.clone().upcast()
    }
}
