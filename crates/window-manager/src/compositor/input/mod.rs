pub mod keyboard;
pub mod launcher;
pub mod pointer;
pub mod touch;
pub mod touchpad;

use smithay::{
    backend::input::{AbsolutePositionEvent, InputBackend, InputEvent},
    utils::{Logical, Point, Size, Transform},
};

use super::state::BabydraState;

pub(crate) fn transform_absolute_position<B: InputBackend, T: AbsolutePositionEvent<B>>(
    state: &BabydraState,
    event: &T,
    touch_override: bool,
) -> Option<Point<f64, Logical>> {
    let output = state.space.outputs().next()?;
    let output_geometry = state.space.output_geometry(output)?;
    let transform = if touch_override {
        match std::env::var("BABYDRADE_TOUCH_TRANSFORM").as_deref() {
            Ok("90") => Transform::_90,
            Ok("180") => Transform::_180,
            Ok("270") => Transform::_270,
            Ok("flipped") => Transform::Flipped,
            Ok("flipped-90") => Transform::Flipped90,
            Ok("flipped-180") => Transform::Flipped180,
            Ok("flipped-270") => Transform::Flipped270,
            Ok("normal") | Ok("0") => Transform::Normal,
            _ => output.current_transform(),
        }
    } else {
        output.current_transform()
    };

    let raw_logical_size: Size<i32, Logical> = output
        .current_mode()
        .map(|m| Size::from((m.size.w, m.size.h)))
        .unwrap_or_else(|| transform.transform_size(output_geometry.size));

    let raw_point = event.position_transformed(raw_logical_size);
    let area_f64 = Size::<f64, Logical>::from((raw_logical_size.w as f64, raw_logical_size.h as f64));
    let oriented_point = transform.transform_point_in(raw_point, &area_f64);

    let mut position = oriented_point + output_geometry.loc.to_f64();
    let max_x = (output_geometry.loc.x + output_geometry.size.w) as f64;
    let max_y = (output_geometry.loc.y + output_geometry.size.h) as f64;
    position.x = position.x.clamp(output_geometry.loc.x as f64, max_x);
    position.y = position.y.clamp(output_geometry.loc.y as f64, max_y);

    Some(position)
}

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
