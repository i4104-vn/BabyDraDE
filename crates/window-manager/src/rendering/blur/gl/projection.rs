use smithay::utils::{Physical, Rectangle};

/// Projects a destination rectangle from Wayland oriented coordinates to OpenGL physical
/// framebuffer coordinates using Smithay's exact projection matrix for the current frame.
pub fn project_rect_to_gl(
    dst: Rectangle<i32, Physical>,
    projection: &[f32; 9],
    screen_w: i32,
    screen_h: i32,
) -> (i32, i32, i32, i32) {
    let x0 = dst.loc.x as f32;
    let y0 = dst.loc.y as f32;
    let x1 = x0 + dst.size.w as f32;
    let y1 = y0 + dst.size.h as f32;

    let corners = [(x0, y0), (x1, y0), (x0, y1), (x1, y1)];

    let mut min_gx = f32::INFINITY;
    let mut max_gx = f32::NEG_INFINITY;
    let mut min_gy = f32::INFINITY;
    let mut max_gy = f32::NEG_INFINITY;

    for (x, y) in corners {
        // Column-major Matrix3 * vec3(x, y, 1.0)
        let mut rx = projection[0] * x + projection[3] * y + projection[6];
        let mut ry = projection[1] * x + projection[4] * y + projection[7];
        let rw = projection[2] * x + projection[5] * y + projection[8];

        if rw.abs() > 1e-6 {
            rx /= rw;
            ry /= rw;
        }

        // OpenGL NDC [-1.0, 1.0] -> Viewport [0, screen_size]
        let gx = (rx + 1.0) * 0.5 * screen_w as f32;
        let gy = (ry + 1.0) * 0.5 * screen_h as f32;

        min_gx = min_gx.min(gx);
        max_gx = max_gx.max(gx);
        min_gy = min_gy.min(gy);
        max_gy = max_gy.max(gy);
    }

    let gl_x = min_gx.round() as i32;
    let gl_y = min_gy.round() as i32;
    let gl_w = (max_gx - min_gx).round() as i32;
    let gl_h = (max_gy - min_gy).round() as i32;

    (gl_x, gl_y, gl_w, gl_h)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_rect_to_gl_normal() {
        // Identity projection for 1920x1080 display
        // M00=2/1920, M11=2/1080, M02=-1, M12=-1 (Y up)
        let mut proj = [0.0f32; 9];
        proj[0] = 2.0 / 1920.0;
        proj[4] = 2.0 / 1080.0;
        proj[6] = -1.0;
        proj[7] = -1.0;
        proj[8] = 1.0;

        let dst = Rectangle::new((100, 200).into(), (400, 300).into());
        let (x, y, w, h) = project_rect_to_gl(dst, &proj, 1920, 1080);
        assert_eq!((x, y, w, h), (100, 200, 400, 300));
    }

    #[test]
    fn test_project_rect_to_gl_rotated_90() {
        // Rotated projection for 1600x2560 hardware panel displayed at 2560x1600 (Transform::_270 render transform)
        // With gx = 1600 - ly, gy = lx
        let mut proj = [0.0f32; 9];
        // ndc_x = 1.0 - 2.0 * y / 1600.0 => proj[3] = -2.0/1600.0, proj[6] = 1.0 (row 0)
        proj[3] = -2.0 / 1600.0;
        proj[6] = 1.0;
        // ndc_y = 2.0 * x / 2560.0 - 1.0 => proj[1] = 2.0/2560.0, proj[7] = -1.0 (row 1)
        proj[1] = 2.0 / 2560.0;
        proj[7] = -1.0;
        proj[8] = 1.0;

        // Window at top-left: x=0, y=0, w=400, h=300
        let dst = Rectangle::new((0, 0).into(), (400, 300).into());
        let (x, y, w, h) = project_rect_to_gl(dst, &proj, 1600, 2560);
        // gx range = [1600 - 300, 1600] = [1300, 1600]
        // gy range = [0, 400]
        assert_eq!((x, y, w, h), (1300, 0, 300, 400));
    }
}
