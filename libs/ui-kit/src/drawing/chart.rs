//! Custom Cairo Chart Drawing for metrics history (CPU, RAM, GPU, etc.)

use gtk4::cairo;

fn hex_to_rgb(hex: &str) -> (f64, f64, f64) {
    let clean = hex.trim_start_matches('#');
    if let Ok(val) = u32::from_str_radix(clean, 16) {
        let r = ((val >> 16) & 0xFF) as f64 / 255.0;
        let g = ((val >> 8) & 0xFF) as f64 / 255.0;
        let b = (val & 0xFF) as f64 / 255.0;
        (r, g, b)
    } else {
        (0.5, 0.5, 0.5)
    }
}

pub fn draw_metric_chart(cr: &cairo::Context, w: f64, h: f64, history: &[f64], color_hex: &str) {
    if history.is_empty() {
        return;
    }

    // 1. Background grid lines (4 levels)
    cr.set_source_rgba(1.0, 1.0, 1.0, 0.08);
    cr.set_line_width(1.0);
    for i in 1..4 {
        let y = h * (i as f64) / 4.0;
        cr.move_to(0.0, y);
        cr.line_to(w, y);
        let _ = cr.stroke();
    }

    let len = history.len();
    if len < 2 {
        return;
    }
    let step = w / ((len - 1) as f64);

    // 2. Filled area below line
    cr.move_to(0.0, h);
    for (i, &val) in history.iter().enumerate() {
        let x = i as f64 * step;
        let y = h - (val / 100.0) * h;
        cr.line_to(x, y.max(0.0).min(h));
    }
    cr.line_to(w, h);
    cr.close_path();

    let (r, g, b) = hex_to_rgb(color_hex);
    cr.set_source_rgba(r, g, b, 0.15);
    let _ = cr.fill();

    // 3. Line stroke
    let mut first = true;
    for (i, &val) in history.iter().enumerate() {
        let x = i as f64 * step;
        let y = h - (val / 100.0) * h;
        if first {
            cr.move_to(x, y.max(0.0).min(h));
            first = false;
        } else {
            cr.line_to(x, y.max(0.0).min(h));
        }
    }

    cr.set_source_rgba(r, g, b, 0.85);
    cr.set_line_width(2.0);
    let _ = cr.stroke();
}
