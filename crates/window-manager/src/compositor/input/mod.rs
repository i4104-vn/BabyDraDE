pub mod keyboard;
pub mod launcher;
pub mod pointer;
pub mod touchpad;

use smithay::backend::input::{InputBackend, InputEvent};

use super::state::BabydraState;

impl BabydraState {
    pub(crate) fn process_input_event<I: InputBackend>(&mut self, event: InputEvent<I>) {
        match event {
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
            _ => {}
        }
    }
}
