//! Compositor input processing pipeline.
//!
//! Dispatches events from the backend (keyboard, pointer, touchpad, touch)
//! to the appropriate specialized input sub-modules.

use smithay::{
    backend::input::{InputBackend, InputEvent},
    utils::SERIAL_COUNTER,
};

use crate::adapters::window::InputContext;
use crate::application::gestures::WindowIntent;

use super::{keyboard, pointer, touch, touchpad};

/// Performs the action chosen by the gesture use case against compositor state.
///
/// Returns whether the event was consumed (i.e. must not be replayed to
/// clients). `MiddleClick` is performed by the touch adapter, which owns the
/// pointer event synthesis.
pub(crate) fn execute_window_intent<C: InputContext>(state: &mut C, intent: WindowIntent) -> bool {
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

/// Central dispatch function for all hardware and synthetic input events.
///
/// Routes incoming [`InputEvent`] variants from the backend to specialized handlers
/// for keyboard shortcuts, pointer motion and clicks, touchpad gestures, and multi-touch interactions.
pub(crate) fn process_input_event<C: InputContext, I: InputBackend>(
    state: &mut C,
    event: InputEvent<I>,
) where
    I::Device: 'static,
{
    match event {
        InputEvent::DeviceAdded { mut device } => {
            touchpad::handle_device_added(&mut device);
            state.reload_input_config();
        }
        InputEvent::DeviceRemoved { device } => {
            touchpad::handle_device_removed(&device);
        }
        InputEvent::Keyboard { event, .. } => {
            keyboard::handle_keyboard(state, event);
        }
        InputEvent::PointerMotion { event, .. } => {
            pointer::handle_pointer_motion(state, event);
        }
        InputEvent::PointerMotionAbsolute { event, .. } => {
            pointer::handle_absolute_motion(state, event);
        }
        InputEvent::PointerButton { event, .. } => {
            pointer::handle_pointer_button(state, event);
        }
        InputEvent::PointerAxis { event, .. } => {
            touchpad::handle_axis(state, event);
        }
        InputEvent::GestureSwipeBegin { event, .. } => {
            touchpad::handle_swipe_begin(state, event);
        }
        InputEvent::GestureSwipeUpdate { event, .. } => {
            touchpad::handle_swipe_update(state, event);
        }
        InputEvent::GestureSwipeEnd { event, .. } => {
            touchpad::handle_swipe_end(state, event);
        }
        InputEvent::GesturePinchBegin { event, .. } => {
            touchpad::handle_pinch_begin(state, event);
        }
        InputEvent::GesturePinchUpdate { event, .. } => {
            touchpad::handle_pinch_update(state, event);
        }
        InputEvent::GesturePinchEnd { event, .. } => {
            touchpad::handle_pinch_end(state, event);
        }
        InputEvent::GestureHoldBegin { event, .. } => {
            touchpad::handle_hold_begin(state, event);
        }
        InputEvent::GestureHoldEnd { event, .. } => {
            touchpad::handle_hold_end(state, event);
        }
        InputEvent::TouchDown { event, .. } => {
            touch::handle_touch_down(state, event);
        }
        InputEvent::TouchMotion { event, .. } => {
            touch::handle_touch_motion(state, event);
        }
        InputEvent::TouchUp { event, .. } => {
            touch::handle_touch_up(state, event);
        }
        InputEvent::TouchCancel { event, .. } => {
            touch::handle_touch_cancel(state, event);
        }
        InputEvent::TouchFrame { event, .. } => {
            touch::handle_touch_frame(state, event);
        }
        _ => {}
    }
}
