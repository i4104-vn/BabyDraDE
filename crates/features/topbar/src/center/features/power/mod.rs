//! Power Menu Dynamic Island feature.
//!
//! Uses the common island lifecycle: `build(ctx)` wires the popover, buttons,
//! keyboard controllers and the single IPC trigger via the shared context.

pub mod controller;
pub mod models;
pub mod ui;

pub use models::*;

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::Duration;

use gtk4::prelude::*;

use crate::center::{IslandCtx, IslandFeature, NotchWidget, CAPSULE_HEIGHT, CAPSULE_WIDTH};
use ui::{execute_power_action, highlight_selection, PowerPopover};

pub const PRIORITY: u32 = 100;
const POPUP_DURATION: Duration = Duration::from_secs(30);

pub struct PowerFeature {
    widgets: NotchWidget,
    popover: Rc<RefCell<Option<PowerPopover>>>,
    selected: Rc<Cell<usize>>,
}

impl PowerFeature {
    pub fn new() -> Self {
        Self {
            widgets: NotchWidget::new("power", &services::i18n::trans("island.power_notch")),
            popover: Rc::new(RefCell::new(None)),
            selected: Rc::new(Cell::new(0)),
        }
    }
}

impl Default for PowerFeature {
    fn default() -> Self {
        Self::new()
    }
}

impl IslandFeature for PowerFeature {
    fn id(&self) -> &'static str {
        "power"
    }

    fn priority(&self) -> u32 {
        PRIORITY
    }

    fn size(&self) -> (i32, i32) {
        (CAPSULE_WIDTH, CAPSULE_HEIGHT)
    }

    fn is_alive(&self) -> bool {
        self.is_popover_open()
    }

    fn is_popover_open(&self) -> bool {
        self.popover
            .borrow()
            .as_ref()
            .is_some_and(|p| p.is_visible())
    }

    fn build(&mut self, ctx: &IslandCtx) -> gtk4::Widget {
        let popover = PowerPopover::new(&ctx.capsule());
        let selected = self.selected.clone();
        let popover_rc = self.popover.clone();

        for (index, button) in popover.buttons.iter().enumerate() {
            let popover_clone = popover.clone();
            let ctx_clone = ctx.clone();
            button.click_gesture.connect_pressed(move |_, _, _, _| {
                execute_power_action(index, &popover_clone);
                ctx_clone.release_override();
                ctx_clone.hide();
                ctx_clone.tick();
            });

            let popover_hover = popover.clone();
            let selected = selected.clone();
            button.motion_controller.connect_enter(move |_, _, _| {
                selected.set(index);
                highlight_selection(&popover_hover, index);
            });
        }

        let key_selected = selected.clone();
        let key_popover = popover_rc.clone();
        let ctx_for_keys = ctx.clone();
        ctx.attach_keyboard(&popover.popover, &popover.popover_box, move || {
            controller::create_keyboard_controller(
                key_selected.clone(),
                key_popover.clone(),
                ctx_for_keys.clone(),
            )
        });

        let popover_for_trigger = popover_rc.clone();
        let selected_for_trigger = selected.clone();
        let ctx_for_trigger = ctx.clone();
        ctx.on_trigger(move || {
            selected_for_trigger.set(0);
            ctx_for_trigger.dismiss_popovers();
            ctx_for_trigger.override_show_for(POPUP_DURATION);
            ctx_for_trigger.tick();
            if let Some(popover) = popover_for_trigger.borrow().as_ref() {
                if popover.root().is_some() {
                    highlight_selection(popover, 0);
                    popover.popup();
                }
            }
        });

        self.popover.replace(Some(popover));
        self.widgets.container.clone().upcast()
    }

    fn on_click(&mut self) {
        if let Some(popover) = self.popover.borrow().as_ref() {
            if !popover.is_visible() {
                self.selected.set(0);
                highlight_selection(popover, 0);
            }
            popover.toggle();
        }
    }

    fn on_hide(&mut self) {
        if let Some(popover) = self.popover.borrow().as_ref() {
            popover.popdown();
        }
    }
}
