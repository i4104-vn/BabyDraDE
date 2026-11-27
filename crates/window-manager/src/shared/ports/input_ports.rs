//! Narrow ports between the shared window adapter and its consumers.
//!
//! The input pipeline depends on these capabilities instead of one god trait,
//! so each adapter only sees the operations it actually needs. `InputContext`
//! remains as a compatibility aggregate; new code should depend on the
//! smallest port that covers its use case.

use std::{collections::HashSet, time::Instant};

use smithay::{
    backend::input::{KeyState, TouchSlot},
    desktop::{Space, Window},
    input::{
        keyboard::{FilterResult, Keycode, KeysymHandle, ModifiersState},
        pointer::{
            AxisFrame, ButtonEvent, GestureHoldBeginEvent, GestureHoldEndEvent,
            GesturePinchBeginEvent, GesturePinchEndEvent, GesturePinchUpdateEvent,
            GestureSwipeBeginEvent, GestureSwipeEndEvent, GestureSwipeUpdateEvent,
            MotionEvent as PointerMotionEvent,
        },
        touch::{
            DownEvent as TouchDownEvent, MotionEvent as TouchMotionEvent, UpEvent as TouchUpEvent,
        },
    },
    reexports::{
        wayland_protocols::xdg::shell::server::xdg_toplevel::ResizeEdge,
        wayland_server::protocol::wl_surface::WlSurface,
    },
    utils::{Logical, Point, Serial, Size},
    wayland::shell::wlr_layer::Layer,
};

use common::{Action, ShortcutTarget};

use crate::core::shortcuts::Mods;
use crate::shared::ports::GesturePolicy;

use crate::modules::window::interaction::{
    MoveState, OverlayTouchState, ResizeState, TouchGestureBuffer, TrackpadSwipeBuffer,
};

/// Toolkit-neutral editing operations exposed by a search query.
///
/// The input layer edits text through this port instead of importing the
/// Granularity of a caret movement step.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CaretUnit {
    /// Move by a single character.
    Character,
    /// Move by a word boundary.
    Word,
}

/// Whether a caret movement keeps or clears the current selection.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CaretSelection {
    /// Move the caret and drop the selection.
    Move,
    /// Extend the selection to the new caret position.
    Extend,
}

/// launcher's concrete query model, keeping the dependency one-way.
pub(crate) trait QueryText {
    /// Inserts text at the caret, replacing the current selection.
    fn insert(&mut self, text: &str);
    /// Deletes the selection or the character before the caret.
    fn backspace(&mut self);
    /// Deletes the selection or the character after the caret.
    fn delete(&mut self);
    /// Deletes from the previous word boundary up to the caret.
    fn delete_word(&mut self);
    /// Moves the caret, optionally by words and optionally extending selection.
    fn move_caret(&mut self, direction: i32, unit: CaretUnit, selection: CaretSelection);
    /// Moves the caret to the start of the query.
    fn home(&mut self, extend: bool);
    /// Moves the caret to the end of the query.
    fn end(&mut self, extend: bool);
    /// Selects the whole query.
    fn select_all(&mut self);
}

/// Window collection and lifecycle control.
pub(crate) trait WindowControl {
    fn space(&self) -> &Space<Window>;
    fn space_mut(&mut self) -> &mut Space<Window>;
    fn windows(&self) -> &[Window];
    fn active_window(&self) -> Option<Window>;
    fn update_active_window(&self, window: Option<&Window>);
    fn stop_loop(&self);
    fn raise_window(&mut self, window: &Window);
    fn cleanup_windows(&mut self);
    fn unminimize_window(&mut self, window: &Window);
    fn unmaximize_window(&mut self, window: &Window);
    fn close_window(&mut self, window: &Window);
    fn minimize_window(&mut self, window: &Window);
    fn toggle_maximize(&mut self, window: &Window);
    fn toggle_fullscreen(&mut self, window: &Window);
    /// Focuses and raises a window on the seat, using a fresh serial.
    fn focus_window(&mut self, window: Window, serial: smithay::utils::Serial);
}

/// Pure queries about what lies under a pointer position.
pub(crate) trait HitTestQuery {
    fn window_under(&self, point: Point<f64, Logical>) -> Option<(Window, Point<i32, Logical>)>;
    fn window_border_under(
        &self,
        point: Point<f64, Logical>,
    ) -> Option<(Window, Point<i32, Logical>, ResizeEdge)>;
    fn surface_under(&self, point: Point<f64, Logical>)
        -> Option<(WlSurface, Point<f64, Logical>)>;
    fn layer_under(&self, point: Point<f64, Logical>) -> Option<Layer>;
    fn titlebar_action(
        &self,
        window: &Window,
        location: Point<i32, Logical>,
        point: Point<f64, Logical>,
    ) -> Option<Action>;
    fn saved_window_size(&self, window: &Window) -> Option<Size<i32, Logical>>;
}

/// In-progress interaction buffers owned by the input pipeline.
pub(crate) trait InteractionBuffer {
    fn move_state_mut(&mut self) -> &mut Option<MoveState>;
    fn resize_state_mut(&mut self) -> &mut Option<ResizeState>;
    fn last_titlebar_click_mut(&mut self) -> &mut Option<(Instant, Point<f64, Logical>, Window)>;
    fn trackpad_swipe(&self) -> Option<&TrackpadSwipeBuffer>;
    fn trackpad_swipe_mut(&mut self) -> &mut Option<TrackpadSwipeBuffer>;
    fn touch_gesture(&self) -> &TouchGestureBuffer;
    fn touch_gesture_mut(&mut self) -> &mut TouchGestureBuffer;
    fn overlay_touch_mut(&mut self) -> &mut Option<OverlayTouchState>;
    fn touch_slots(&self) -> &HashSet<TouchSlot>;
    fn touch_slots_mut(&mut self) -> &mut HashSet<TouchSlot>;
    fn pointer_pressed_buttons(&self) -> &HashSet<u32>;
    fn pointer_pressed_buttons_mut(&mut self) -> &mut HashSet<u32>;
    fn pointer_location(&self) -> Point<f64, Logical>;
    fn set_pointer_location(&mut self, position: Point<f64, Logical>);
    fn set_cursor_visible(&mut self, visible: bool);
    /// Touch rotation currently applied by the auto-rotate sensor.
    fn touch_transform(&self) -> Option<smithay::utils::Transform>;
    /// Records the touch rotation used by the next absolute input event.
    fn set_touch_transform(&mut self, transform: Option<smithay::utils::Transform>);
}

/// Configuration lookups and system-wide actions triggered by input.
pub(crate) trait SystemActions {
    fn gesture_config(&self) -> &GesturePolicy;
    fn touchpad_natural_scroll(&self) -> bool;
    fn mouse_natural_scroll(&self) -> bool;
    fn reload_input_config(&mut self);
    fn reload_config(&mut self);
    fn shortcut_command(&self, mods: Mods, keysym: u32) -> Option<String>;
    fn registered_shortcut(&self, mods: Mods, keysym: u32) -> Option<ShortcutTarget>;
    fn reload_shortcuts(&self);
    fn launch_command(&self, command: String);
    fn lock_screen(&self);
    fn trigger_screenshot(&self);
}

/// Shell overlays (launcher, search and Alt+Tab switcher) driven by input.
pub(crate) trait OverlayBridge {
    fn launcher_is_visible(&self) -> bool;
    fn launcher_arrow_pressed(&mut self, direction: i32) -> bool;
    fn launcher_move_selection(&mut self, direction: i32);
    fn launcher_arrow_released(&mut self, direction: i32);
    fn launcher_activate(&mut self);
    fn launcher_close(&mut self);
    fn launcher_toggle(&mut self);
    fn launcher_index_at(&self, position: Point<f64, Logical>) -> Option<usize>;
    fn launcher_launch(&mut self, index: usize);
    fn launcher_update_hover(&mut self, position: Point<f64, Logical>);
    fn launcher_drag_by(&mut self, slots: f64);
    fn launcher_finish_drag(&mut self);
    fn launcher_is_over_panel(&self, position: Point<f64, Logical>) -> bool;
    fn launcher_scroll_by(&mut self, slots: f64);

    fn launcher_search_is_active(&self) -> bool;
    /// Neutral editing handle; no shell-specific type crosses this boundary.
    fn launcher_search_text_mut(&mut self) -> &mut dyn QueryText;
    fn launcher_search_move(&mut self, delta: i32);
    fn launcher_search_escape(&mut self) -> bool;
    fn launcher_search_click(&mut self, position: Point<f64, Logical>) -> bool;

    fn cycle_alt_tab(&mut self, backwards: bool);
    fn begin_switcher_scrub(&mut self) -> Option<usize>;
    fn scrub_switcher(&mut self, start_x: f64, base_index: usize, current_x: f64);
    fn commit_switcher_selection(&mut self);
    fn switcher_is_active(&self) -> bool;
    fn switcher_tab_pressed(&mut self, direction: i32) -> bool;
    fn switcher_tab_released(&mut self, direction: i32);
    fn switcher_begin_close(&mut self);
    fn switcher_set_hover(&mut self, index: Option<usize>);
    fn switcher_drag_by(&mut self, slots: f64);
    fn switcher_finish_drag(&mut self);
    fn switcher_index_at(&self, point: (f64, f64), slide: f64) -> Option<usize>;
    fn switcher_slide(&self, panel_width: f64) -> f64;
    fn switcher_window_at(&self, index: usize) -> Option<Window>;
    fn switcher_selected_window(&self) -> Option<Window>;
}

/// Seat dispatch: forwards backend events to Smithay with concrete seat data.
pub(crate) trait SeatSink {
    fn has_keyboard(&self) -> bool;
    fn keyboard_modifiers(&self) -> ModifiersState;
    fn keyboard_input<T, F>(
        &mut self,
        keycode: Keycode,
        state: KeyState,
        serial: Serial,
        time: u32,
        filter: F,
    ) -> Option<T>
    where
        F: FnOnce(&mut Self, &ModifiersState, KeysymHandle<'_>) -> FilterResult<T>;
    fn set_keyboard_focus(&mut self, focus: Option<WlSurface>, serial: Serial);
    fn has_pointer(&self) -> bool;
    fn pointer_is_grabbed(&self) -> bool;
    fn pointer_motion(
        &mut self,
        focus: Option<(WlSurface, Point<f64, Logical>)>,
        event: &PointerMotionEvent,
    );
    fn pointer_button(&mut self, event: &ButtonEvent);
    fn pointer_axis(&mut self, frame: AxisFrame);
    fn pointer_frame(&mut self);
    fn touch_down(
        &mut self,
        focus: Option<(WlSurface, Point<f64, Logical>)>,
        event: &TouchDownEvent,
    );
    fn touch_motion(
        &mut self,
        focus: Option<(WlSurface, Point<f64, Logical>)>,
        event: &TouchMotionEvent,
    );
    fn touch_up(&mut self, event: &TouchUpEvent);
    fn touch_cancel(&mut self);
    fn touch_frame(&mut self);
    fn gesture_swipe_begin(&mut self, event: &GestureSwipeBeginEvent);
    fn gesture_swipe_update(&mut self, event: &GestureSwipeUpdateEvent);
    fn gesture_swipe_end(&mut self, event: &GestureSwipeEndEvent);
    fn gesture_pinch_begin(&mut self, event: &GesturePinchBeginEvent);
    fn gesture_pinch_update(&mut self, event: &GesturePinchUpdateEvent);
    fn gesture_pinch_end(&mut self, event: &GesturePinchEndEvent);
    fn gesture_hold_begin(&mut self, event: &GestureHoldBeginEvent);
    fn gesture_hold_end(&mut self, event: &GestureHoldEndEvent);
}

/// Aggregate of every input-facing port, retained for existing callers.
pub(crate) trait InputContext:
    WindowControl + HitTestQuery + InteractionBuffer + SystemActions + OverlayBridge + SeatSink
{
}

impl<T> InputContext for T where
    T: WindowControl + HitTestQuery + InteractionBuffer + SystemActions + OverlayBridge + SeatSink
{
}
