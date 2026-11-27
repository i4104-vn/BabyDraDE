//! Screen recording Dynamic Island feature.
//!
//! Unified island feature: `build(ctx)` wires the capsule, popover, keyboard
//! controllers, IPC trigger and state poller through the shared context. The
//! UI is split into `ui/` parts and actions into `controller/actions/`.

pub mod controller;
pub mod models;
pub mod service;
pub mod ui;

pub use models::{
    IslandRecordingState, PopoverActionsContext, RecordingArea, RecordingConfig, RecordingMode,
};

use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use gtk4::prelude::*;

use crate::island::{IslandCtx, IslandFeature, CAPSULE_HEIGHT, CAPSULE_WIDTH};
use service::spawn_recording_polling;
use ui::{RecordingCapsuleWidget, RecordingPopover};

pub const PRIORITY: u32 = 85;
pub const POPUP_DURATION: Duration = Duration::from_secs(30);

pub struct RecordingFeature {
    capsule: RecordingCapsuleWidget,
    popover: Rc<RefCell<Option<RecordingPopover>>>,
    state: Rc<RefCell<IslandRecordingState>>,
    was_recording: bool,
}

impl RecordingFeature {
    pub fn new() -> Self {
        Self {
            capsule: RecordingCapsuleWidget::new(),
            popover: Rc::new(RefCell::new(None)),
            state: spawn_recording_polling(),
            was_recording: false,
        }
    }
}

impl Default for RecordingFeature {
    fn default() -> Self {
        Self::new()
    }
}

impl IslandFeature for RecordingFeature {
    fn id(&self) -> &'static str {
        "recording"
    }

    fn priority(&self) -> u32 {
        PRIORITY
    }

    fn size(&self) -> (i32, i32) {
        (CAPSULE_WIDTH, CAPSULE_HEIGHT)
    }

    fn hover_keep(&self) -> bool {
        true
    }

    fn is_alive(&self) -> bool {
        self.state.borrow().is_recording || self.is_popover_open()
    }

    fn is_popover_open(&self) -> bool {
        self.popover
            .borrow()
            .as_ref()
            .is_some_and(|p| p.is_visible())
    }

    fn build(&mut self, ctx: &IslandCtx) -> gtk4::Widget {
        let popover = RecordingPopover::new(&ctx.capsule());
        let popover_rc = self.popover.clone();
        let state = self.state.clone();

        let key_popover = popover_rc.clone();
        let ctx_for_keys = ctx.clone();
        ctx.attach_keyboard(&popover.popover, &popover.popover_box, move || {
            controller::create_keyboard_controller(key_popover.clone(), ctx_for_keys.clone())
        });

        let ctx_for_closed = ctx.clone();
        popover.popover.connect_closed(move |_| {
            if !state.borrow().is_recording && !service::is_recording() {
                ctx_for_closed.release_override();
                ctx_for_closed.hide();
                ctx_for_closed.tick();
            }
        });

        let popover_for_trigger = popover_rc.clone();
        let ctx_for_trigger = ctx.clone();
        ctx.on_trigger(move || {
            ctx_for_trigger.dismiss_popovers();
            ctx_for_trigger.override_show_for(POPUP_DURATION);
            ctx_for_trigger.tick();
            if let Some(popover) = popover_for_trigger.borrow().as_ref() {
                if !service::is_recording() {
                    popover.reset_to_config();
                }
                if popover.root().is_some() {
                    popover.popup();
                }
            }
        });

        self.popover.replace(Some(popover));
        self.capsule.container.clone().upcast()
    }

    fn on_click(&mut self) {
        let Some(popover) = self.popover.borrow().as_ref().cloned() else {
            return;
        };
        if popover.is_visible() {
            popover.popdown();
        } else {
            if !service::is_recording() {
                popover.reset_to_config();
            } else {
                popover.update_data(&self.state.borrow());
            }
            popover.popup();
        }
    }

    fn tick(&mut self, ctx: &IslandCtx) {
        let is_recording = service::is_recording();
        let mut state = self.state.borrow().clone();
        if !is_recording {
            state.is_recording = false;
        }

        if self.is_popover_open() {
            ctx.override_show_for(POPUP_DURATION);
        }
        if let Some(popover) = self.popover.borrow().as_ref() {
            if popover.is_visible() {
                popover.update_data(&state);
            }
        }

        if state.is_recording {
            ctx.show();
            let elapsed = state.elapsed_secs;
            let time = if elapsed >= 3600 {
                format!(
                    "{:02}:{:02}:{:02}",
                    elapsed / 3600,
                    (elapsed % 3600) / 60,
                    elapsed % 60
                )
            } else {
                format!("{:02}:{:02}", elapsed / 60, elapsed % 60)
            };
            let timer_text = if state.is_paused {
                format!("PAUSED {time}")
            } else {
                time
            };
            self.capsule.update_timer(&timer_text);
            self.was_recording = true;
        } else if self.was_recording {
            self.capsule.update_timer("00:00");
            if let Some(popover) = self.popover.borrow().as_ref() {
                popover.reset_to_config();
            }
            if !self.is_popover_open() {
                ctx.override_show_for(Duration::from_secs(3));
            }
            self.was_recording = false;
        }
    }
}
