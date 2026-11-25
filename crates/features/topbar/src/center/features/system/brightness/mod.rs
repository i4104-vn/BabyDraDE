//! System Brightness indicator feature for the Dynamic Island.

pub mod service;

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::Duration;

use gtk4::prelude::*;

use crate::center::poller::spawn_poller;
use crate::center::{IslandCtx, IslandFeature, NotchWidget, CAPSULE_HEIGHT, CAPSULE_WIDTH};

pub const PRIORITY: u32 = 95;
pub const SHOW_DURATION: Duration = Duration::from_millis(1500);
const POLL_INTERVAL: Duration = Duration::from_millis(500);

fn icon_for(value: f64) -> &'static str {
    if value <= 20.0 {
        "brightness-low"
    } else if value <= 70.0 {
        "brightness-medium"
    } else {
        "brightness"
    }
}

fn label_for(value: f64) -> String {
    format!("{:.0}%", value.clamp(0.0, 100.0).round())
}

pub struct BrightnessFeature {
    widgets: NotchWidget,
    last: Rc<RefCell<f64>>,
    initialized: Rc<Cell<bool>>,
}

impl BrightnessFeature {
    pub fn new() -> Self {
        let initial = service::read();
        Self {
            widgets: NotchWidget::with_value(
                icon_for(initial),
                &services::i18n::trans("common.brightness"),
                &label_for(initial),
            ),
            last: Rc::new(RefCell::new(initial)),
            initialized: Rc::new(Cell::new(false)),
        }
    }
}

impl Default for BrightnessFeature {
    fn default() -> Self {
        Self::new()
    }
}

impl IslandFeature for BrightnessFeature {
    fn id(&self) -> &'static str {
        "system_brightness"
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

        spawn_poller("island-brightness", POLL_INTERVAL, service::read, move |value| {
            if !initialized.replace(true) {
                *last.borrow_mut() = value;
                return;
            }
            if (value - *last.borrow()).abs() >= 0.5 {
                *last.borrow_mut() = value;
                widgets.update(icon_for(value), &label_for(value));
                ctx.override_show_for(SHOW_DURATION);
                ctx.tick();
            }
        });

        self.widgets.container.clone().upcast()
    }
}
