//! Compositor component models.
//!
//! The seven components owned by the facade are collected here; the structures
//! defined in other feature slices are re-exported so `app::models::state` is
//! the single place to reach them.

use std::sync::{Arc, Mutex};

use common::ShortcutTarget;

use crate::core::models::policy::GesturePolicy;
use crate::core::shortcuts::app::{ShortcutRegistry, ShortcutService};
use crate::core::shortcuts::Mods;
use crate::shared::ports::LoadGesturePolicy;

pub(crate) use crate::modules::wayland::clipboard::ClipboardState;
pub(crate) use crate::modules::wayland::WaylandRuntime;
pub(crate) use crate::modules::window::WindowRegistry;

// Transient input interaction state owned by the compositor.

use std::collections::HashSet;
use std::time::Instant;

use smithay::{
    backend::input::TouchSlot,
    desktop::Window,
    input::pointer::CursorIcon,
    utils::{Logical, Point, Transform},
};

use crate::modules::window::interaction::{
    MoveState, OverlayTouchState, ResizeState, TouchGestureBuffer, TrackpadSwipeBuffer,
};

/// Buffers and flags for move/resize drags, pointer buttons and touch gestures.
#[derive(Default)]
pub(crate) struct InputState {
    pub(crate) move_state: Option<MoveState>,
    pub(crate) resize_state: Option<ResizeState>,
    pub(crate) last_titlebar_click: Option<(Instant, Point<f64, Logical>, Window)>,
    pub(crate) trackpad_swipe: Option<TrackpadSwipeBuffer>,
    /// Last cursor shape requested by the focused Wayland client.
    pub(crate) cursor_icon: CursorIcon,
    pub(crate) pointer_pressed_buttons: HashSet<u32>,
    pub(crate) touch_slots: HashSet<TouchSlot>,
    /// Active single-finger touch interaction with compositor overlays
    /// (launcher or switcher).
    pub(crate) overlay_touch: Option<OverlayTouchState>,
    /// Accumulated per-slot data for multi-touch gesture detection on
    /// touchscreens.
    pub(crate) touch_gesture: TouchGestureBuffer,
    /// Current pointer position in logical coordinates.
    pub(crate) pointer_location: Point<f64, Logical>,
    /// Whether the compositor-drawn cursor is visible.
    pub(crate) cursor_visible: bool,
    /// Touch rotation applied by the auto-rotate sensor, if any.
    pub(crate) touch_transform: Option<Transform>,
}

impl InputState {
    pub(crate) fn new() -> Self {
        Self {
            move_state: None,
            resize_state: None,
            last_titlebar_click: None,
            trackpad_swipe: None,
            cursor_icon: CursorIcon::Default,
            pointer_pressed_buttons: HashSet::new(),
            touch_slots: HashSet::new(),
            overlay_touch: None,
            touch_gesture: TouchGestureBuffer::default(),
            pointer_location: Point::from((100.0, 100.0)),
            cursor_visible: true,
            touch_transform: None,
        }
    }
}

// State for desktop overlays (application launcher and Alt+Tab switcher).

pub(crate) use state::OverlayState;

mod state {
    //! State owned by the compositor overlays.

    use crate::modules::shell::launcher::LauncherState;
    use crate::modules::shell::switcher::models::AltTabSwitcher;

    /// The launcher and the Alt+Tab switcher share overlay semantics (they are
    /// mutually exclusive and consume the same touch gestures), so their state is
    /// owned together.
    pub(crate) struct OverlayState {
        pub(crate) alt_tab_switcher: Option<AltTabSwitcher>,
        pub(crate) launcher: LauncherState,
    }

    impl OverlayState {
        pub(crate) fn new() -> Self {
            Self {
                alt_tab_switcher: None,
                launcher: LauncherState::new(),
            }
        }
    }
}

// Rendering resources owned by the compositor state.

use crate::modules::blur::BlurEngine;

/// Owns the shared blur pipeline used by every frame renderer.
pub(crate) struct RenderState {
    pub(crate) blur_engine: Arc<Mutex<BlurEngine>>,
}

impl RenderState {
    pub(crate) fn new() -> Self {
        Self {
            blur_engine: Arc::new(Mutex::new(BlurEngine::new())),
        }
    }

    /// Hot reloads blur configuration from disk.
    pub(crate) fn reload_config(&self) {
        let mut engine = self
            .blur_engine
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        engine.reload_config();
        tracing::info!(
            "Win+F5: Configuration reloaded from disk ({} app rules registered for blur)",
            engine.params.app_rules.len()
        );
    }
}

// Framework-free configuration snapshot owned by the input pipeline.
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
    /// Runtime shortcut registry shared with the public API and IPC watcher.
    registry: Arc<ShortcutRegistry>,
}

impl ConfigSnapshot {
    pub(crate) fn new(
        load_gesture_policy: LoadGesturePolicy,
        shortcuts: Arc<ShortcutService>,
        registry: Arc<ShortcutRegistry>,
    ) -> Self {
        Self {
            touchpad_natural_scroll: services::config::load_touchpad_config().natural_scroll,
            mouse_natural_scroll: services::config::load_mouse_config().natural_scroll,
            gesture_config: load_gesture_policy(),
            load_gesture_policy,
            shortcuts,
            registry,
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

    /// Returns a runtime-registered shortcut matching an exact key press.
    ///
    /// # Arguments
    ///
    /// * `mods` - Active modifier mask.
    /// * `keysym` - Layout-independent keysym.
    ///
    /// # Returns
    ///
    /// The registered target, or `None` when nothing matches.
    pub(crate) fn registered_target(&self, mods: Mods, keysym: u32) -> Option<ShortcutTarget> {
        self.registry.target_for(mods, keysym)
    }

    pub(crate) fn reload_shortcuts(&self) {
        self.shortcuts.reload();
    }

    pub(crate) fn launch_command(&self, command: String) {
        self.shortcuts.launch(command);
    }
}
