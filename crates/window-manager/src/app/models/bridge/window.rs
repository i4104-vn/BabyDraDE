//! Window-related port implementations for `BabydraState`.

use std::{collections::HashSet, time::Instant};

use common::Action;
use smithay::{
    backend::input::TouchSlot,
    desktop::{Space, Window},
    reexports::{
        wayland_protocols::xdg::shell::server::xdg_toplevel::ResizeEdge,
        wayland_server::protocol::wl_surface::WlSurface,
    },
    utils::{Logical, Point, Serial, Size, Transform},
    wayland::shell::wlr_layer::Layer,
};

use crate::shared::ports::{HitTestQuery, InteractionBuffer, WindowControl};
use crate::modules::window::hit_test::titlebar;
use crate::modules::window::state_data::lock_window_state;
use crate::app::models::runtime::BabydraState;

impl WindowControl for BabydraState {
    fn space(&self) -> &Space<Window> {
        &self.windows.space
    }

    fn space_mut(&mut self) -> &mut Space<Window> {
        &mut self.windows.space
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
        self.resources.loop_signal.stop();
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

    fn focus_window(&mut self, window: Window, serial: Serial) {
        crate::modules::input::keyboard::focus::focus_window(self, window, serial);
    }
}

impl HitTestQuery for BabydraState {
    fn window_under(&self, point: Point<f64, Logical>) -> Option<(Window, Point<i32, Logical>)> {
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
        titlebar::hit_test(window, location, point)
    }

    fn saved_window_size(&self, window: &Window) -> Option<Size<i32, Logical>> {
        // A poisoned lock keeps the last known geometry rather than crashing.
        lock_window_state(window).saved_size
    }
}

impl InteractionBuffer for BabydraState {
    fn move_state_mut(&mut self) -> &mut Option<crate::modules::window::interaction::MoveState> {
        &mut self.input.move_state
    }

    fn resize_state_mut(&mut self) -> &mut Option<crate::modules::window::interaction::ResizeState> {
        &mut self.input.resize_state
    }

    fn last_titlebar_click_mut(&mut self) -> &mut Option<(Instant, Point<f64, Logical>, Window)> {
        &mut self.input.last_titlebar_click
    }

    fn trackpad_swipe(&self) -> Option<&crate::modules::window::interaction::TrackpadSwipeBuffer> {
        self.input.trackpad_swipe.as_ref()
    }

    fn trackpad_swipe_mut(&mut self) -> &mut Option<crate::modules::window::interaction::TrackpadSwipeBuffer> {
        &mut self.input.trackpad_swipe
    }

    fn touch_gesture(&self) -> &crate::modules::window::interaction::TouchGestureBuffer {
        &self.input.touch_gesture
    }

    fn touch_gesture_mut(&mut self) -> &mut crate::modules::window::interaction::TouchGestureBuffer {
        &mut self.input.touch_gesture
    }

    fn overlay_touch_mut(&mut self) -> &mut Option<crate::modules::window::interaction::OverlayTouchState> {
        &mut self.input.overlay_touch
    }

    fn touch_slots(&self) -> &HashSet<TouchSlot> {
        &self.input.touch_slots
    }

    fn touch_slots_mut(&mut self) -> &mut HashSet<TouchSlot> {
        &mut self.input.touch_slots
    }

    fn pointer_pressed_buttons(&self) -> &HashSet<u32> {
        &self.input.pointer_pressed_buttons
    }

    fn pointer_pressed_buttons_mut(&mut self) -> &mut HashSet<u32> {
        &mut self.input.pointer_pressed_buttons
    }

    fn pointer_location(&self) -> Point<f64, Logical> {
        self.input.pointer_location
    }

    fn set_pointer_location(&mut self, position: Point<f64, Logical>) {
        self.input.pointer_location = position;
    }

    fn set_cursor_visible(&mut self, visible: bool) {
        self.input.cursor_visible = visible;
    }

    fn touch_transform(&self) -> Option<Transform> {
        self.input.touch_transform
    }

    fn set_touch_transform(&mut self, transform: Option<Transform>) {
        self.input.touch_transform = transform;
    }
}
