//! Narrow view of compositor state required by the input pipeline.
//!
//! The input adapters depend on this trait instead of the concrete compositor
//! state, so the dependency between the two adapters points one way only.

use std::{collections::HashSet, time::Instant};

use smithay::{
    backend::input::{KeyState, TouchSlot},
    desktop::Space,
    desktop::Window,
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

use common::Action;

use crate::application::ports::GesturePolicy;
use crate::domain::shortcuts::Mods;

use super::interaction::{
    MoveState, OverlayTouchState, ResizeState, TouchGestureBuffer, TrackpadSwipeBuffer,
};

/// Everything the input pipeline needs from the state it drives.
pub(crate) trait InputContext {
    // -- shared window state --
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

    // -- hit testing --
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

    // -- in-progress interactions --
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

    // -- input configuration --
    fn gesture_config(&self) -> &GesturePolicy;
    fn touchpad_natural_scroll(&self) -> bool;
    fn mouse_natural_scroll(&self) -> bool;
    fn reload_input_config(&mut self);
    fn reload_config(&mut self);

    // -- shortcuts and desktop actions --
    fn shortcut_command(&self, mods: Mods, keysym: u32) -> Option<String>;
    fn reload_shortcuts(&self);
    fn launch_command(&self, command: String);
    fn lock_screen(&self);
    fn trigger_screenshot(&self);

    // -- launcher overlay --
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

    // -- Alt+Tab switcher overlay --
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

    // -- seat dispatch (delegates to Smithay with the concrete seat data) --
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
