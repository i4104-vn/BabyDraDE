//! Compositor input processing pipeline.
//!
//! Dispatches events from the backend (keyboard, pointer, touchpad, touch)
//! to the appropriate specialized input sub-modules.

pub mod common;
pub mod keyboard;
pub mod pointer;
pub mod touch;
pub mod touchpad;

// Backwards compatibility re-exports
#[allow(unused_imports)]
pub use keyboard::launcher;

use smithay::backend::input::{InputBackend, InputEvent};

use super::state::BabydraState;

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
