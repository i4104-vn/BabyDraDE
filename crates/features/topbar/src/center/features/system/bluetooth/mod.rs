//! System Bluetooth device status feature for the Dynamic Island.

pub mod service;

use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use gtk4::prelude::*;

use crate::center::poller::spawn_poller;
use crate::center::{IslandCtx, IslandFeature, NotchWidget, CAPSULE_HEIGHT, CAPSULE_WIDTH};
use service::BtConnectedDevice;

pub const PRIORITY: u32 = 92;
pub const SHOW_DURATION: Duration = Duration::from_millis(2000);
const POLL_INTERVAL: Duration = Duration::from_millis(1000);

fn presentation(devices: &[BtConnectedDevice]) -> (String, String) {
    match devices.first() {
        Some(device) => (
            device.name.clone(),
            device
                .battery
                .map(|level| format!("{}%", level))
                .unwrap_or_default(),
        ),
        None => (services::i18n::trans("island.disconnected"), String::new()),
    }
}

pub struct BluetoothFeature {
    widgets: NotchWidget,
    last: Rc<RefCell<Vec<BtConnectedDevice>>>,
}

impl BluetoothFeature {
    pub fn new() -> Self {
        let initial = service::read();
        let (title, value) = presentation(&initial);
        Self {
            widgets: NotchWidget::with_value("bluetooth", &title, &value),
            last: Rc::new(RefCell::new(initial)),
        }
    }
}

impl Default for BluetoothFeature {
    fn default() -> Self {
        Self::new()
    }
}

impl IslandFeature for BluetoothFeature {
    fn id(&self) -> &'static str {
        "system_bluetooth"
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
        let ctx = ctx.clone();

        spawn_poller("island-bluetooth", POLL_INTERVAL, service::read, move |devices| {
            if devices == *last.borrow() {
                return;
            }
            *last.borrow_mut() = devices.clone();
            let (title, value) = presentation(&devices);
            widgets.update_all("bluetooth", &title, &value);
            ctx.override_show_for(SHOW_DURATION);
            ctx.tick();
        });

        self.widgets.container.clone().upcast()
    }
}
