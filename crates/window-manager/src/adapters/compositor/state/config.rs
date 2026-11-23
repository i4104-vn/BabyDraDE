//! Cached configuration snapshot used by the input pipeline.

use std::sync::Arc;

use crate::application::ports::{GesturePolicy, LoadGesturePolicy};
use crate::application::shortcuts::ShortcutService;
use crate::domain::shortcuts::Mods;

/// Framework-free configuration values refreshed only on config reload.
pub(crate) struct ConfigSnapshot {
    /// Cached touchpad configuration loaded from `~/.babydra/configs/input.toml`.
    pub(crate) touchpad_natural_scroll: bool,
    /// Cached mouse/pointer configuration natural scroll setting.
    pub(crate) mouse_natural_scroll: bool,
    /// Framework-free gesture policy snapshot.
    pub(crate) gesture_config: GesturePolicy,
    load_gesture_policy: LoadGesturePolicy,
    /// Injected shortcut service shared with the reload watcher and input pipeline.
    shortcuts: Arc<ShortcutService>,
}

impl ConfigSnapshot {
    pub(crate) fn new(
        load_gesture_policy: LoadGesturePolicy,
        shortcuts: Arc<ShortcutService>,
    ) -> Self {
        Self {
            touchpad_natural_scroll: services::config::load_touchpad_config().natural_scroll,
            mouse_natural_scroll: services::config::load_mouse_config().natural_scroll,
            gesture_config: load_gesture_policy(),
            load_gesture_policy,
            shortcuts,
        }
    }

    /// Reloads input device configuration from disk and refreshes all caches.
    pub(crate) fn reload(&mut self) {
        services::config::invalidate_cache();
        self.touchpad_natural_scroll = services::config::load_touchpad_config().natural_scroll;
        self.mouse_natural_scroll = services::config::load_mouse_config().natural_scroll;
        self.gesture_config = (self.load_gesture_policy)();
    }

    pub(crate) fn gesture_config(&self) -> &GesturePolicy {
        &self.gesture_config
    }

    pub(crate) fn touchpad_natural_scroll(&self) -> bool {
        self.touchpad_natural_scroll
    }

    pub(crate) fn mouse_natural_scroll(&self) -> bool {
        self.mouse_natural_scroll
    }

    pub(crate) fn shortcut_command(&self, mods: Mods, keysym: u32) -> Option<String> {
        self.shortcuts.command_for(mods, keysym)
    }

    pub(crate) fn reload_shortcuts(&self) {
        self.shortcuts.reload();
    }

    pub(crate) fn launch_command(&self, command: String) {
        self.shortcuts.launch(command);
    }
}
