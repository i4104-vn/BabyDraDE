pub mod keyboard;
pub mod launcher;
pub mod pointer;

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
            _ => {}
        }
    }
}
