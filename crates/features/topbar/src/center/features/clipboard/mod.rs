//! Clipboard history Dynamic Island feature.
//!
//! Follows the common island pattern: one `IslandFeature::build(ctx)` wires the
//! view, its popover, keyboard controllers and IPC trigger through the shared
//! context. `service.rs` only exposes clipboard data operations.

pub mod controller;
pub mod service;
pub mod ui;

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::{Duration, Instant};

use gtk4::prelude::*;

use crate::center::{
    IslandCtx, IslandFeature, NotchWidget, CAPSULE_HEIGHT, CAPSULE_WIDTH,
};
use ui::{render_popover, ClipboardPopover};

pub const PRIORITY: u32 = 95;
const POPUP_DURATION: Duration = Duration::from_secs(15);
const ENABLED_CHECK_INTERVAL: Duration = Duration::from_secs(2);

pub struct ClipboardFeature {
    widgets: NotchWidget,
    popover: Rc<RefCell<Option<ClipboardPopover>>>,
    selected: Rc<Cell<usize>>,
    last_revision: u64,
    enabled: bool,
    enabled_checked_at: Instant,
}

impl ClipboardFeature {
    pub fn new() -> Self {
        Self {
            widgets: NotchWidget::new("paste", &services::i18n::trans("island.clipboard_notch")),
            popover: Rc::new(RefCell::new(None)),
            selected: Rc::new(Cell::new(0)),
            last_revision: service::revision(),
            enabled: service::is_enabled(),
            enabled_checked_at: Instant::now(),
        }
    }

    fn render_current_list(&mut self) {
        let entries = service::get_entries();
        self.last_revision = service::revision();
        if entries.is_empty() {
            self.selected.set(0);
        } else if self.selected.get() >= entries.len() {
            self.selected.set(entries.len() - 1);
        }
        if let Some(popover) = self.popover.borrow().as_ref() {
            render_popover(popover, &entries, self.selected.get());
        }
    }
}

impl Default for ClipboardFeature {
    fn default() -> Self {
        Self::new()
    }
}

impl IslandFeature for ClipboardFeature {
    fn id(&self) -> &'static str {
        "clipboard"
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
        self.popover.borrow().as_ref().is_some_and(|p| p.is_visible())
    }

    fn build(&mut self, ctx: &IslandCtx) -> gtk4::Widget {
        let ctx = ctx.clone();
        let popover = ClipboardPopover::new(&ctx.capsule());
        let popover_rc = self.popover.clone();
        let selected = self.selected.clone();

        controller::pointer::wire(&popover, selected.clone(), popover_rc.clone(), &ctx);

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

        // One IPC callback, registered through the shared context.
        let trigger_popover = popover_rc.clone();
        let trigger_selected = selected.clone();
        let ctx_for_trigger = ctx.clone();
        ctx.on_trigger(move || {
            if !service::is_enabled() {
                return;
            }
            trigger_selected.set(0);
            ctx_for_trigger.dismiss_popovers();
            ctx_for_trigger.override_show_for(POPUP_DURATION);
            ctx_for_trigger.tick();
            let entries = service::get_entries();
            if let Some(popover) = trigger_popover.borrow().as_ref() {
                if popover.root().is_some() {
                    render_popover(popover, &entries, 0);
                    popover.popup();
                }
            }
        });

        self.popover.replace(Some(popover));
        self.widgets.container.clone().upcast()
    }

    fn tick(&mut self, _ctx: &IslandCtx) {
        if self.enabled_checked_at.elapsed() >= ENABLED_CHECK_INTERVAL {
            self.enabled = service::is_enabled();
            self.enabled_checked_at = Instant::now();
        }
        if self.enabled && service::revision() != self.last_revision {
            self.render_current_list();
        }
    }

    fn on_click(&mut self) {
        if let Some(popover) = self.popover.borrow().as_ref() {
            if !popover.is_visible() {
                self.selected.set(0);
                render_popover(popover, &service::get_entries(), 0);
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
