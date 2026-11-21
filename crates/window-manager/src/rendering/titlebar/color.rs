//! Sampling the color that the client has actually rendered.

use smithay::{
    backend::allocator::{Buffer as _, Fourcc},
    desktop::Window,
    wayland::compositor::with_states,
};

const FALLBACK_COLOR: [f64; 3] = [0.0, 0.0, 0.0];
const SAMPLE_DISTANCE_SQUARED: f64 = 0.008;

/// Get the representative color of the client's currently committed buffer.
///
/// This deliberately samples on every titlebar render: a maximized client can
/// commit a new buffer immediately after its configure event.
pub(crate) fn current_color(window: &Window) -> [f64; 3] {
    sample_dmabuf_color(window)
        .or_else(|| sample_shm_color(window))
        .unwrap_or(FALLBACK_COLOR)
}

fn sample_dmabuf_color(window: &Window) -> Option<[f64; 3]> {
    let toplevel = window.toplevel()?;
    with_states(toplevel.wl_surface(), |states| {
        let surface_state = states
            .data_map
            .get::<smithay::backend::renderer::utils::RendererSurfaceStateUserData>(
        )?;
        let state = surface_state.lock().ok()?;
        let buffer = state.buffer()?;
        let dmabuf = smithay::wayland::dmabuf::get_dmabuf(buffer).ok()?;

        let size = dmabuf.size();
        if size.w <= 0 || size.h <= 0 {
            return None;
        }
        let stride = dmabuf.strides().next()? as usize;
        let format = dmabuf.format().code;

        let mapping = dmabuf
            .map_plane(
                0,
                smithay::backend::allocator::dmabuf::DmabufMappingMode::READ,
            )
            .ok()?;
        let pixels =
            unsafe { std::slice::from_raw_parts(mapping.ptr() as *const u8, mapping.length()) };
        let color = representative_color(pixels, size.w, size.h, stride, format);
        color
    })
}

fn sample_shm_color(window: &Window) -> Option<[f64; 3]> {
    let toplevel = window.toplevel()?;
    with_states(toplevel.wl_surface(), |states| {
        let surface_state = states
            .data_map
            .get::<smithay::backend::renderer::utils::RendererSurfaceStateUserData>(
        )?;
        let state = surface_state.lock().ok()?;
        let buffer = state.buffer()?;
        smithay::wayland::shm::with_buffer_contents(buffer, |ptr, len, data| {
            let format = smithay::wayland::shm::shm_format_to_fourcc(data.format)?;
            let pixels = unsafe { std::slice::from_raw_parts(ptr, len) };
            representative_color(
                pixels,
                data.width,
                data.height,
                data.stride as usize,
                format,
            )
        })
        .ok()?
    })
}

fn representative_color(
    pixels: &[u8],
    width: i32,
    height: i32,
    stride: usize,
    format: Fourcc,
) -> Option<[f64; 3]> {
    if width <= 0 || height <= 0 {
        return None;
    }

    let inset = 8.min(width / 4);
    let top = 8.min(height / 4);
    let points = [
        (inset, top),
        (width / 2, top),
        (width - 1 - inset, top),
        (width / 2, height / 2),
        (inset, height / 2),
    ];
    // At most five samples are needed, so avoid allocating on every frame.
    let mut samples = [[0.0; 3]; 5];
    let mut count = 0;
    for (x, y) in points {
        if let Some(color) = read_pixel(pixels, x, y, stride, format) {
            samples[count] = color;
            count += 1;
        }
    }
    dominant_color(&samples[..count])
}

fn read_pixel(pixels: &[u8], x: i32, y: i32, stride: usize, format: Fourcc) -> Option<[f64; 3]> {
    let offset = y as usize * stride + x as usize * 4;
    let pixel = pixels.get(offset..offset + 4)?;
    let rgb = match format {
        Fourcc::Argb8888 | Fourcc::Xrgb8888 => [pixel[2], pixel[1], pixel[0]],
        Fourcc::Abgr8888 | Fourcc::Xbgr8888 => [pixel[0], pixel[1], pixel[2]],
        _ => return None,
    };
    Some(rgb.map(|channel| channel as f64 / 255.0))
}

fn dominant_color(samples: &[[f64; 3]]) -> Option<[f64; 3]> {
    samples.iter().copied().max_by_key(|candidate| {
        samples
            .iter()
            .filter(|sample| squared_distance(candidate, sample) < SAMPLE_DISTANCE_SQUARED)
            .count()
    })
}

fn squared_distance(left: &[f64; 3], right: &[f64; 3]) -> f64 {
    left.iter()
        .zip(right)
        .map(|(left, right)| (left - right).powi(2))
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preserves_a_light_dominant_color() {
        let samples = [
            [0.95, 0.95, 0.95],
            [0.96, 0.96, 0.96],
            [0.94, 0.94, 0.94],
            [0.1, 0.1, 0.1],
        ];
        assert!(dominant_color(&samples).unwrap()[0] > 0.9);
    }
}
