//! Multi-touch input event handlers: touch down, motion, up, cancel, and frame.

mod dispatch;
mod gesture_state;
mod overlay;

pub(crate) use dispatch::{
    handle_touch_cancel, handle_touch_down, handle_touch_frame, handle_touch_motion,
    handle_touch_up,
};

#[cfg(test)]
mod tests {
    use smithay::utils::{Logical, Point, Size, Transform};

    #[test]
    fn test_touch_coordinate_rotation_90() {
        let raw_size = Size::<f64, Logical>::from((1600.0, 2560.0));
        let transform = Transform::_90;

        // Physical center (800, 1280) -> Landscape center (1280, 800)
        let pt_center = Point::<f64, Logical>::from((800.0, 1280.0));
        let oriented_center = transform.transform_point_in(pt_center, &raw_size);
        assert_eq!(oriented_center.x, 1280.0);
        assert_eq!(oriented_center.y, 800.0);

        // Physical bottom-left (0, 2560) -> Landscape top-left (0, 0)
        let pt_bl = Point::<f64, Logical>::from((0.0, 2560.0));
        let oriented_bl = transform.transform_point_in(pt_bl, &raw_size);
        assert_eq!(oriented_bl.x, 0.0);
        assert_eq!(oriented_bl.y, 0.0);

        // Physical top-left (0, 0) -> Landscape top-right (2560, 0)
        let pt_tl = Point::<f64, Logical>::from((0.0, 0.0));
        let oriented_tl = transform.transform_point_in(pt_tl, &raw_size);
        assert_eq!(oriented_tl.x, 2560.0);
        assert_eq!(oriented_tl.y, 0.0);
    }
}
