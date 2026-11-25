//! System Network (Wi-Fi and Ethernet) status feature for the Dynamic Island.

pub mod service;

use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use gtk4::prelude::*;

use crate::center::poller::spawn_poller;
use crate::center::{IslandCtx, IslandFeature, NotchWidget, CAPSULE_HEIGHT, CAPSULE_WIDTH};
use service::{ActiveNetworkType, NetworkSnapshot};

pub const PRIORITY: u32 = 92;
pub const SHOW_DURATION: Duration = Duration::from_millis(2000);
const POLL_INTERVAL: Duration = Duration::from_millis(800);

fn presentation(snapshot: &NetworkSnapshot) -> (&'static str, String, String) {
    if !snapshot.connected {
        return (
            "wifi",
            services::i18n::trans("island.disconnected"),
            String::new(),
        );
    }
    match snapshot.kind {
        ActiveNetworkType::Wifi => (
            "wifi",
            snapshot.name.clone(),
            format!("{}%", snapshot.strength),
        ),
        ActiveNetworkType::Ethernet => ("ethernet", snapshot.name.clone(), String::new()),
        ActiveNetworkType::Disconnected => ("wifi", snapshot.name.clone(), String::new()),
    }
}

pub struct NetworkFeature {
    widgets: NotchWidget,
    last: Rc<RefCell<NetworkSnapshot>>,
}

impl NetworkFeature {
    pub fn new() -> Self {
        let initial = service::read();
        let (icon, title, value) = presentation(&initial);
        Self {
            widgets: NotchWidget::with_value(icon, &title, &value),
            last: Rc::new(RefCell::new(initial)),
        }
    }
}

impl Default for NetworkFeature {
    fn default() -> Self {
        Self::new()
    }
}

impl IslandFeature for NetworkFeature {
    fn id(&self) -> &'static str {
        "system_network"
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

        spawn_poller(
            "island-network",
            POLL_INTERVAL,
            service::read,
            move |snapshot: NetworkSnapshot| {
                if snapshot == *last.borrow() {
                    return;
                }
                *last.borrow_mut() = snapshot.clone();
                let (icon, title, value) = presentation(&snapshot);
                widgets.update_all(icon, &title, &value);
                ctx.override_show_for(SHOW_DURATION);
                ctx.tick();
            },
        );

        self.widgets.container.clone().upcast()
    }
}
