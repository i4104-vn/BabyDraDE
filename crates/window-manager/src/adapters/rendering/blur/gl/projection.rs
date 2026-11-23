use smithay::utils::{Physical, Rectangle};

/// Projects a destination rectangle from Wayland oriented coordinates to OpenGL physical
/// framebuffer coordinates using Smithay's exact projection matrix for the current frame.
pub(crate) fn project_rect_to_gl(
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
#[path = "../../../../tests/adapters/rendering/blur/gl/projection.rs"]
mod tests;
