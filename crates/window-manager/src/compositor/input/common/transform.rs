use smithay::{
    backend::input::{AbsolutePositionEvent, InputBackend},
    utils::{Logical, Point, Size, Transform},
};

use crate::compositor::state::BabydraState;

/// Transforms raw hardware absolute input coordinates into logical compositor coordinates,
/// taking into account screen geometry, orientation, and optional touchscreen override transforms.
///
/// # Arguments
///
/// * `state` - Mutable or immutable reference to [`BabydraState`] containing the current output layout.
/// * `event` - The absolute input position event providing transformed positions relative to screen mode.
/// * `touch_override` - Whether to check `BABYDRADE_TOUCH_TRANSFORM` for physical orientation override (used by touchscreens).
///
/// # Returns
///
/// Returns `Some(Point<f64, Logical>)` clamped to output geometry bounds, or `None` if no output is attached.
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
