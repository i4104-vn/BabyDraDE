//! The one context object passed to every feature callback.

use std::cell::RefCell;
use std::rc::Weak;
use std::time::Duration;

use gtk4::prelude::*;

use super::handle::IslandViewHandle;
use super::IslandEngine;

#[derive(Clone)]
pub struct IslandCtx {
    capsule: gtk4::Box,
    handle: IslandViewHandle,
    engine: Weak<RefCell<IslandEngine>>,
    feature_id: &'static str,
    current: bool,
    hovered: bool,
}

impl IslandCtx {
    pub(crate) fn new(
        capsule: gtk4::Box,
        handle: IslandViewHandle,
        engine: Weak<RefCell<IslandEngine>>,
        feature_id: &'static str,
        current: bool,
        hovered: bool,
    ) -> Self {
        Self {
            capsule,
            handle,
            engine,
            feature_id,
            current,
            hovered,
        }
    }

    pub fn capsule(&self) -> gtk4::Box {
        self.capsule.clone()
    }

    pub fn is_current(&self) -> bool {
        self.current
    }

    pub fn is_hovered(&self) -> bool {
        self.hovered
    }

    // ---- View requests (the only way a feature requests display) ----------

    pub fn show(&self) {
        self.handle.show();
    }

    pub fn show_for(&self, duration: Duration) {
        self.handle.show_for(duration);
    }

    pub fn hide(&self) {
        self.handle.hide();
    }

    pub fn override_show_for(&self, duration: Duration) {
        self.handle.override_show_for(duration);
    }

    pub fn release_override(&self) {
        self.handle.release_override();
    }

    // ---- Shared services --------------------------------------------------

    /// Requests an immediate arbitration tick after a listener / IPC event.
    /// A no-op while the engine is already ticking, so re-entry is safe.
    pub fn tick(&self) {
        if let Some(engine) = self.engine.upgrade() {
            let rc = engine.clone();
            if let Ok(mut engine) = rc.try_borrow_mut() {
                engine.tick(&rc);
            };
        }
    }

    /// Closes every island popover.
    pub fn dismiss_popovers(&self) {
        crate::island::ui::popover::dismiss_all_popovers();
    }

    /// Attaches one keyboard-controller factory to the popover, its content
    /// box, the capsule and the layer window.
    pub fn attach_keyboard<F>(&self, popover: &gtk4::Popover, popover_box: &gtk4::Box, factory: F)
    where
        F: Fn() -> gtk4::EventControllerKey,
    {
        popover.add_controller(factory());
        popover_box.add_controller(factory());
        self.capsule.add_controller(factory());
        if let Some(root) = self.capsule.root() {
            if let Some(window) = root.downcast_ref::<gtk4::Window>() {
                window.add_controller(factory());
            }
        }
    }

    /// Registers this feature's callback for its IPC trigger topic.
    pub fn on_trigger<F: Fn() + 'static>(&self, callback: F) {
        crate::island::ipc::register_trigger(self.feature_id, callback);
    }
}
