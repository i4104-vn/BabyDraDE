//! Compositor input processing pipeline.
//!
//! Dispatches events from the backend (keyboard, pointer, touchpad, touch)
//! to the appropriate specialized input sub-modules.

pub(crate) mod common;
pub(crate) mod keyboard;
pub(crate) mod keymap;
pub(crate) mod pointer;
pub(crate) mod touch;
pub(crate) mod touchpad;

use smithay::{
    backend::input::{InputBackend, InputEvent},
    utils::SERIAL_COUNTER,
};

use crate::{adapters::compositor::state::BabydraState, application::gestures::WindowIntent};

/// Performs the action chosen by the gesture use case against compositor state.
///
/// Returns whether the event was consumed (i.e. must not be replayed to
/// clients). `MiddleClick` is performed by the touch adapter, which owns the
/// pointer event synthesis.
pub(crate) fn execute_window_intent(state: &mut BabydraState, intent: WindowIntent) -> bool {
    match intent {
        WindowIntent::Consumed => true,
        WindowIntent::CycleFocus { forward } => {
            keyboard::cycle_window_focus(state, forward, SERIAL_COUNTER.next_serial())
        }
        WindowIntent::Maximize => {
            if let Some(window) = state.active_window() {
                state.toggle_maximize(&window);
                true
            } else {
                false
            }
        }
        WindowIntent::Minimize => {
            if let Some(window) = state.active_window() {
                state.minimize_window(&window);
                true
            } else {
                false
            }
        }
        WindowIntent::Fullscreen => {
            if let Some(window) = state.active_window() {
                state.toggle_fullscreen(&window);
                true
            } else {
                false
            }
        }
        WindowIntent::Close => {
            if let Some(window) = state.active_window() {
                state.close_window(&window);
                true
            } else {
                false
            }
        }
        WindowIntent::MiddleClick => false,
    }
}

impl BabydraState {
    /// Central dispatch function for all hardware and synthetic input events.
    ///
    /// Routes incoming [`InputEvent`] variants from the backend to specialized handlers
    /// for keyboard shortcuts, pointer motion and clicks, touchpad gestures, and multi-touch interactions.
    ///
    /// # Arguments
    ///
    /// * `event` - The incoming generic [`InputEvent`] received from the input backend.
    pub(crate) fn process_input_event<I: InputBackend>(&mut self, event: InputEvent<I>)
    where
        I::Device: 'static,
    {
        match event {
            InputEvent::DeviceAdded { mut device } => {
                touchpad::handle_device_added(&mut device);
                self.reload_input_config();
            }
            InputEvent::DeviceRemoved { device } => {
                touchpad::handle_device_removed(&device);
            }
            InputEvent::Keyboard { event, .. } => {
                keyboard::handle_keyboard(self, event);
            }
            InputEvent::PointerMotion { event, .. } => {
                pointer::handle_pointer_motion(self, event);
            }
            InputEvent::PointerMotionAbsolute { event, .. } => {
                pointer::handle_pointer_motion_absolute(self, event);
            }
            InputEvent::PointerButton { event, .. } => {
                pointer::handle_pointer_button(self, event);
            }
            InputEvent::PointerAxis { event, .. } => {
                touchpad::handle_axis(self, event);
            }
            InputEvent::GestureSwipeBegin { event, .. } => {
                touchpad::handle_swipe_begin(self, event);
            }
            InputEvent::GestureSwipeUpdate { event, .. } => {
                touchpad::handle_swipe_update(self, event);
            }
            InputEvent::GestureSwipeEnd { event, .. } => {
                touchpad::handle_swipe_end(self, event);
            }
            InputEvent::GesturePinchBegin { event, .. } => {
                touchpad::handle_pinch_begin(self, event);
            }
            InputEvent::GesturePinchUpdate { event, .. } => {
                touchpad::handle_pinch_update(self, event);
            }
            InputEvent::GesturePinchEnd { event, .. } => {
                touchpad::handle_pinch_end(self, event);
            }
            InputEvent::GestureHoldBegin { event, .. } => {
                touchpad::handle_hold_begin(self, event);
            }
            InputEvent::GestureHoldEnd { event, .. } => {
                touchpad::handle_hold_end(self, event);
            }
            InputEvent::TouchDown { event, .. } => {
                touch::handle_touch_down(self, event);
            }
            InputEvent::TouchMotion { event, .. } => {
                touch::handle_touch_motion(self, event);
            }
            InputEvent::TouchUp { event, .. } => {
                touch::handle_touch_up(self, event);
            }
            InputEvent::TouchCancel { event, .. } => {
                touch::handle_touch_cancel(self, event);
            }
            InputEvent::TouchFrame { event, .. } => {
                touch::handle_touch_frame(self, event);
            }
            _ => {}
        }
    }
}
