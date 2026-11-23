//! `InputContext` implementation for the concrete compositor state.

use std::{collections::HashSet, time::Instant};

use smithay::{
    backend::input::{InputBackend, InputEvent, KeyState, TouchSlot},
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
            DownEvent as TouchDownEvent, MotionEvent as TouchMotionEvent,
            UpEvent as TouchUpEvent,
        },
    },
    reexports::{
        wayland_protocols::xdg::shell::server::xdg_toplevel::ResizeEdge,
        wayland_server::protocol::wl_surface::WlSurface,
    },
    desktop::Space,
};
use common::Action;
use smithay::utils::{Logical, Point, Serial, Size};
use smithay::wayland::shell::wlr_layer::Layer;

use crate::adapters::window::InputContext;
use crate::application::ports::GesturePolicy;
use crate::domain::shortcuts::Mods;

use super::{
    window::state_data::window_state, MoveState, OverlayTouchState, ResizeState, BabydraState,
    TouchGestureBuffer, TrackpadSwipeBuffer,
};

impl InputContext for BabydraState {
    fn space(&self) -> &Space<Window> {
        &self.space
    }

    fn space_mut(&mut self) -> &mut Space<Window> {
        &mut self.space
    }

    fn windows(&self) -> &[Window] {
        &self.windows
    }

    fn active_window(&self) -> Option<Window> {
        BabydraState::active_window(self)
    }

    fn update_active_window(&self, window: Option<&Window>) {
        BabydraState::update_active_window(self, window);
    }

    fn stop_loop(&self) {
        self.loop_signal.stop();
    }

    fn raise_window(&mut self, window: &Window) {
        BabydraState::raise_window(self, window);
    }

    fn cleanup_windows(&mut self) {
        BabydraState::cleanup_windows(self);
    }

    fn unminimize_window(&mut self, window: &Window) {
        BabydraState::unminimize_window(self, window);
    }

    fn unmaximize_window(&mut self, window: &Window) {
        BabydraState::unmaximize_window(self, window);
    }

    fn close_window(&mut self, window: &Window) {
        BabydraState::close_window(self, window);
    }

    fn minimize_window(&mut self, window: &Window) {
        BabydraState::minimize_window(self, window);
    }

    fn toggle_maximize(&mut self, window: &Window) {
        BabydraState::toggle_maximize(self, window);
    }

    fn toggle_fullscreen(&mut self, window: &Window) {
        BabydraState::toggle_fullscreen(self, window);
    }

    fn window_under(
        &self,
        point: Point<f64, Logical>,
    ) -> Option<(Window, Point<i32, Logical>)> {
        BabydraState::window_under(self, point)
    }

    fn window_border_under(
        &self,
        point: Point<f64, Logical>,
    ) -> Option<(Window, Point<i32, Logical>, ResizeEdge)> {
        BabydraState::window_border_under(self, point)
    }

    fn surface_under(
        &self,
        point: Point<f64, Logical>,
    ) -> Option<(WlSurface, Point<f64, Logical>)> {
        BabydraState::surface_under(self, point)
    }

    fn layer_under(&self, point: Point<f64, Logical>) -> Option<Layer> {
        BabydraState::layer_under(self, point)
    }

    fn titlebar_action(
        &self,
        window: &Window,
        location: Point<i32, Logical>,
        point: Point<f64, Logical>,
    ) -> Option<Action> {
        super::titlebar::hit_test(window, location, point)
    }

    fn saved_window_size(&self, window: &Window) -> Option<Size<i32, Logical>> {
        window_state(window).lock().unwrap().saved_size
    }

    fn move_state_mut(&mut self) -> &mut Option<MoveState> {
        &mut self.move_state
    }

    fn resize_state_mut(&mut self) -> &mut Option<ResizeState> {
        &mut self.resize_state
    }

    fn last_titlebar_click_mut(
        &mut self,
    ) -> &mut Option<(Instant, Point<f64, Logical>, Window)> {
        &mut self.last_titlebar_click
    }

    fn trackpad_swipe(&self) -> Option<&TrackpadSwipeBuffer> {
        self.trackpad_swipe.as_ref()
    }

    fn trackpad_swipe_mut(&mut self) -> &mut Option<TrackpadSwipeBuffer> {
        &mut self.trackpad_swipe
    }

    fn touch_gesture(&self) -> &TouchGestureBuffer {
        &self.touch_gesture
    }

    fn touch_gesture_mut(&mut self) -> &mut TouchGestureBuffer {
        &mut self.touch_gesture
    }

    fn overlay_touch_mut(&mut self) -> &mut Option<OverlayTouchState> {
        &mut self.overlay_touch
    }

    fn touch_slots(&self) -> &HashSet<TouchSlot> {
        &self.touch_slots
    }

    fn touch_slots_mut(&mut self) -> &mut HashSet<TouchSlot> {
        &mut self.touch_slots
    }

    fn pointer_pressed_buttons(&self) -> &HashSet<u32> {
        &self.pointer_pressed_buttons
    }

    fn pointer_pressed_buttons_mut(&mut self) -> &mut HashSet<u32> {
        &mut self.pointer_pressed_buttons
    }

    fn pointer_location(&self) -> Point<f64, Logical> {
        self.pointer_location
    }

    fn set_pointer_location(&mut self, position: Point<f64, Logical>) {
        self.pointer_location = position;
    }

    fn set_cursor_visible(&mut self, visible: bool) {
        self.cursor_visible = visible;
    }

    fn gesture_config(&self) -> &GesturePolicy {
        &self.gesture_config
    }

    fn touchpad_natural_scroll(&self) -> bool {
        self.touchpad_natural_scroll
    }

    fn mouse_natural_scroll(&self) -> bool {
        self.mouse_natural_scroll
    }

    fn reload_input_config(&mut self) {
        BabydraState::reload_input_config(self);
    }

    fn reload_config(&mut self) {
        BabydraState::reload_config(self);
    }

    fn shortcut_command(&self, mods: Mods, keysym: u32) -> Option<String> {
        self.shortcuts.command_for(mods, keysym)
    }

    fn reload_shortcuts(&self) {
        self.shortcuts.reload();
    }

    fn launch_command(&self, command: String) {
        self.shortcuts.launch(command);
    }

    fn lock_screen(&self) {
        crate::adapters::desktop::lock::spawn();
    }

    fn open_clipboard_picker(&self) {
        crate::adapters::desktop::clipboard::trigger_picker();
    }

    fn trigger_screenshot(&self) {
        crate::adapters::desktop::screenshot::trigger();
    }

    fn launcher_is_visible(&self) -> bool {
        self.launcher.is_visible()
    }

    fn launcher_arrow_pressed(&mut self, direction: i32) -> bool {
        self.launcher.arrow_pressed(direction)
    }

    fn launcher_move_selection(&mut self, direction: i32) {
        self.launcher.move_selection(direction);
    }

    fn launcher_arrow_released(&mut self, direction: i32) {
        self.launcher.arrow_released(direction);
    }

    fn launcher_activate(&mut self) {
        self.launcher.activate();
    }

    fn launcher_close(&mut self) {
        self.launcher.close();
    }

    fn launcher_toggle(&mut self) {
        self.launcher.toggle();
    }

    fn launcher_index_at(&self, position: Point<f64, Logical>) -> Option<usize> {
        self.launcher.index_at(position)
    }

    fn launcher_launch(&mut self, index: usize) {
        self.launcher.launch(index);
    }

    fn launcher_update_hover(&mut self, position: Point<f64, Logical>) {
        self.launcher.update_hover(position);
    }

    fn launcher_drag_by(&mut self, slots: f64) {
        self.launcher.drag_by(slots);
    }

    fn launcher_finish_drag(&mut self) {
        self.launcher.finish_drag();
    }

    fn launcher_is_over_panel(&self, position: Point<f64, Logical>) -> bool {
        self.launcher.is_over_panel(position)
    }

    fn launcher_scroll_by(&mut self, slots: f64) {
        self.launcher.scroll_by(slots);
    }

    fn cycle_alt_tab(&mut self, backwards: bool) {
        BabydraState::cycle_alt_tab(self, backwards);
    }

    fn begin_switcher_scrub(&mut self) -> Option<usize> {
        BabydraState::begin_switcher_scrub(self)
    }

    fn scrub_switcher(&mut self, start_x: f64, base_index: usize, current_x: f64) {
        BabydraState::scrub_switcher(self, start_x, base_index, current_x);
    }

    fn commit_switcher_selection(&mut self) {
        BabydraState::commit_switcher_selection(self);
    }

    fn switcher_is_active(&self) -> bool {
        self.alt_tab_switcher.is_some()
    }

    fn switcher_tab_pressed(&mut self, direction: i32) -> bool {
        self.alt_tab_switcher
            .as_mut()
            .map(|switcher| switcher.tab_pressed(direction))
            .unwrap_or(false)
    }

    fn switcher_tab_released(&mut self, direction: i32) {
        if let Some(switcher) = self.alt_tab_switcher.as_mut() {
            switcher.tab_released(direction);
        }
    }

    fn switcher_begin_close(&mut self) {
        if let Some(switcher) = self.alt_tab_switcher.as_mut() {
            switcher.begin_close();
        }
    }

    fn switcher_set_hover(&mut self, index: Option<usize>) {
        if let Some(switcher) = self.alt_tab_switcher.as_mut() {
            switcher.set_hover(index);
        }
    }

    fn switcher_drag_by(&mut self, slots: f64) {
        if let Some(switcher) = self.alt_tab_switcher.as_mut() {
            switcher.drag_by(slots);
        }
    }

    fn switcher_finish_drag(&mut self) {
        if let Some(switcher) = self.alt_tab_switcher.as_mut() {
            switcher.finish_drag();
        }
    }

    fn switcher_index_at(&self, point: (f64, f64), slide: f64) -> Option<usize> {
        let switcher = self.alt_tab_switcher.as_ref()?;
        let output = self.space.outputs().next().cloned();
        let (size, scale) = output
            .as_ref()
            .and_then(|output| {
                let mode = output.current_mode()?;
                let size = output.current_transform().transform_size(mode.size);
                Some((
                    size,
                    smithay::utils::Scale::from(output.current_scale().fractional_scale()),
                ))
            })
            .unwrap_or(((1920, 1080).into(), smithay::utils::Scale::from(1.0)));
        let logical_w = (size.w as f64 / scale.x).max(1.0);
        let logical_h = (size.h as f64 / scale.y).max(1.0);
        let layout = crate::adapters::desktop::switcher::view::compute_layout(
            switcher, logical_w, logical_h,
        )?;
        layout.panel.index_at(
            point,
            slide,
            switcher.entries.len(),
            switcher.visual_selection(),
        )
    }

    fn switcher_slide(&self, panel_width: f64) -> f64 {
        self.alt_tab_switcher
            .as_ref()
            .map(|switcher| (1.0 - switcher.presence().clamp(0.0, 1.0)) * panel_width)
            .unwrap_or_default()
    }

    fn switcher_window_at(&self, index: usize) -> Option<Window> {
        self.alt_tab_switcher
            .as_ref()
            .and_then(|switcher| switcher.entries.get(index))
            .map(|entry| entry.window.clone())
    }

    fn switcher_selected_window(&self) -> Option<Window> {
        self.alt_tab_switcher.as_ref().and_then(|switcher| {
            switcher
                .entries
                .get(switcher.selected_index)
                .map(|entry| entry.window.clone())
        })
    }

    fn has_keyboard(&self) -> bool {
        self.seat.get_keyboard().is_some()
    }

    fn keyboard_modifiers(&self) -> ModifiersState {
        self.seat.get_keyboard().unwrap().modifier_state()
    }

    fn keyboard_input<T, F>(
        &mut self,
        keycode: Keycode,
        state: KeyState,
        serial: Serial,
        time: u32,
        filter: F,
    ) -> Option<T>
    where
        F: FnOnce(&mut Self, &ModifiersState, KeysymHandle<'_>) -> FilterResult<T>,
    {
        self.seat
            .get_keyboard()
            .unwrap()
            .input::<T, F>(self, keycode, state, serial, time, filter)
    }

    fn set_keyboard_focus(&mut self, focus: Option<WlSurface>, serial: Serial) {
        if let Some(keyboard) = self.seat.get_keyboard() {
            keyboard.set_focus(self, focus, serial);
        }
    }

    fn has_pointer(&self) -> bool {
        self.seat.get_pointer().is_some()
    }

    fn pointer_is_grabbed(&self) -> bool {
        self.seat.get_pointer().unwrap().is_grabbed()
    }

    fn pointer_motion(
        &mut self,
        focus: Option<(WlSurface, Point<f64, Logical>)>,
        event: &PointerMotionEvent,
    ) {
        self.seat.get_pointer().unwrap().motion(self, focus, event);
    }

    fn pointer_button(&mut self, event: &ButtonEvent) {
        self.seat.get_pointer().unwrap().button(self, event);
    }

    fn pointer_axis(&mut self, frame: AxisFrame) {
        self.seat.get_pointer().unwrap().axis(self, frame);
    }

    fn pointer_frame(&mut self) {
        self.seat.get_pointer().unwrap().frame(self);
    }

    fn touch_down(
        &mut self,
        focus: Option<(WlSurface, Point<f64, Logical>)>,
        event: &TouchDownEvent,
    ) {
        if let Some(touch) = self.seat.get_touch() {
            touch.down(self, focus, event);
        }
    }

    fn touch_motion(
        &mut self,
        focus: Option<(WlSurface, Point<f64, Logical>)>,
        event: &TouchMotionEvent,
    ) {
        if let Some(touch) = self.seat.get_touch() {
            touch.motion(self, focus, event);
        }
    }

    fn touch_up(&mut self, event: &TouchUpEvent) {
        if let Some(touch) = self.seat.get_touch() {
            touch.up(self, event);
        }
    }

    fn touch_cancel(&mut self) {
        if let Some(touch) = self.seat.get_touch() {
            touch.cancel(self);
        }
    }

    fn touch_frame(&mut self) {
        if let Some(touch) = self.seat.get_touch() {
            touch.frame(self);
        }
    }

    fn gesture_swipe_begin(&mut self, event: &GestureSwipeBeginEvent) {
        if let Some(pointer) = self.seat.get_pointer() {
            pointer.gesture_swipe_begin(self, event);
        }
    }

    fn gesture_swipe_update(&mut self, event: &GestureSwipeUpdateEvent) {
        if let Some(pointer) = self.seat.get_pointer() {
            pointer.gesture_swipe_update(self, event);
        }
    }

    fn gesture_swipe_end(&mut self, event: &GestureSwipeEndEvent) {
        if let Some(pointer) = self.seat.get_pointer() {
            pointer.gesture_swipe_end(self, event);
        }
    }

    fn gesture_pinch_begin(&mut self, event: &GesturePinchBeginEvent) {
        if let Some(pointer) = self.seat.get_pointer() {
            pointer.gesture_pinch_begin(self, event);
        }
    }

    fn gesture_pinch_update(&mut self, event: &GesturePinchUpdateEvent) {
        if let Some(pointer) = self.seat.get_pointer() {
            pointer.gesture_pinch_update(self, event);
        }
    }

    fn gesture_pinch_end(&mut self, event: &GesturePinchEndEvent) {
        if let Some(pointer) = self.seat.get_pointer() {
            pointer.gesture_pinch_end(self, event);
        }
    }

    fn gesture_hold_begin(&mut self, event: &GestureHoldBeginEvent) {
        if let Some(pointer) = self.seat.get_pointer() {
            pointer.gesture_hold_begin(self, event);
        }
    }

    fn gesture_hold_end(&mut self, event: &GestureHoldEndEvent) {
        if let Some(pointer) = self.seat.get_pointer() {
            pointer.gesture_hold_end(self, event);
        }
    }
}

impl BabydraState {
    /// Central dispatch function for all hardware and synthetic input events.
    pub(crate) fn process_input_event<I: InputBackend>(&mut self, event: InputEvent<I>)
    where
        I::Device: 'static,
    {
        crate::adapters::input::process_input_event(self, event);
    }
}
