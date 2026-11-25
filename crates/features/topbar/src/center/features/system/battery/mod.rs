//! System Battery status feature for the Dynamic Island.
//!
//! Shows a capsule when the charger is plugged in (5 s) or when the battery
//! drops to <= 20% while discharging.

pub mod service;

use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use gtk4::prelude::*;

use crate::center::poller::spawn_poller;
use crate::center::{IslandCtx, IslandFeature, NotchWidget, CAPSULE_HEIGHT, CAPSULE_WIDTH};
use service::BatteryInfo;

pub const PRIORITY: u32 = 94;
pub const SHOW_DURATION: Duration = Duration::from_secs(5);
const POLL_INTERVAL: Duration = Duration::from_millis(1000);

fn presentation(info: Option<BatteryInfo>) -> (&'static str, String, String) {
    let Some(info) = info else {
        return ("battery", String::new(), String::new());
    };
    let title = if info.is_charging {
        services::i18n::trans("island.charging")
    } else if info.percentage <= 20 {
        services::i18n::trans("island.low_battery")
    } else {
        services::i18n::trans("panel.system")
    };
    ("battery", title, format!("{}%", info.percentage))
}

pub struct BatteryFeature {
    widgets: NotchWidget,
    state: Rc<RefCell<BatteryState>>,
}

#[derive(Default)]
struct BatteryState {
    powered: bool,
    low_warned: bool,
    initialized: bool,
}

impl BatteryFeature {
    pub fn new() -> Self {
        let (icon, title, value) = presentation(service::read());
        Self {
            widgets: NotchWidget::with_value(icon, &title, &value),
            state: Rc::new(RefCell::new(BatteryState {
                powered: service::read().is_some_and(|info| info.is_charging || service::is_ac_plugged()),
                low_warned: service::read().is_some_and(|info| info.percentage <= 20 && !info.is_charging),
                initialized: false,
            })),
        }
    }
}

impl Default for BatteryFeature {
    fn default() -> Self {
        Self::new()
    }
}

impl IslandFeature for BatteryFeature {
    fn id(&self) -> &'static str {
        "system_battery"
    }

    fn priority(&self) -> u32 {
        PRIORITY
    }

    fn size(&self) -> (i32, i32) {
        (CAPSULE_WIDTH, CAPSULE_HEIGHT)
    }

    fn build(&mut self, ctx: &IslandCtx) -> gtk4::Widget {
        let widgets = self.widgets.clone();
        let state = self.state.clone();
        let ctx = ctx.clone();

        spawn_poller("island-battery", POLL_INTERVAL, service::read, move |info| {
            let Some(info) = info else {
                return;
            };
            if info.is_ac_only {
                return;
            }
            let mut state = state.borrow_mut();
            let powered = info.is_charging || service::is_ac_plugged();

            if !state.initialized {
                state.initialized = true;
                state.powered = powered;
                state.low_warned = info.percentage <= 20 && !powered;
                return;
            }

            let title = if !state.powered && powered {
                state.low_warned = false;
                services::i18n::trans("island.charging")
            } else if !powered && info.percentage <= 20 && !state.low_warned {
                state.low_warned = true;
                services::i18n::trans("island.low_battery")
            } else {
                if powered || info.percentage > 20 {
                    state.low_warned = false;
                }
                state.powered = powered;
                return;
            };

            state.powered = powered;
            widgets.update_all("battery", &title, &format!("{}%", info.percentage));
            ctx.override_show_for(SHOW_DURATION);
            ctx.tick();
        });

        self.widgets.container.clone().upcast()
    }
}
