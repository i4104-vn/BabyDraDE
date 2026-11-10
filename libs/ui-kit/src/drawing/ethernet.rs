//! Cairo drawing for Desktop Ethernet status icon with TX/RX activity LED indicators.

use gtk4::cairo;
use std::f64::consts::PI;

pub fn draw_cairo_ethernet_desktop(
    cr: &cairo::Context,
    width: f64,
    _height: f64,
    is_dark: bool,
    is_connected: bool,
    tx_lit: bool,
    rx_lit: bool,
) {
    let (fg_r, fg_g, fg_b) = if is_dark {
        (1.0, 1.0, 1.0)
    } else {
        (0.12, 0.12, 0.14)
    };

    let base_alpha = if is_connected { 0.85 } else { 0.35 };

    let draw_rr = |x: f64, y: f64, rw: f64, rh: f64, r: f64| {
        let deg = PI / 180.0;
        cr.new_sub_path();
        cr.arc(x + rw - r, y + r, r, -90.0 * deg, 0.0 * deg);
        cr.arc(x + rw - r, y + rh - r, r, 0.0 * deg, 90.0 * deg);
        cr.arc(x + r, y + rh - r, r, 90.0 * deg, 180.0 * deg);
        cr.arc(x + r, y + r, r, 180.0 * deg, 270.0 * deg);
        cr.close_path();
    };

    // Desktop Monitor Geometry
    let screen_x = 0.8;
    let screen_y = 1.0;
    let screen_w = 11.4;
    let screen_h = 8.0;
    let screen_r = 1.2;

    draw_rr(
        screen_x + 1.0,
        screen_y + 1.0,
        screen_w - 2.0,
        screen_h - 2.0,
        0.8,
    );
    let inner_alpha = if is_dark { 0.08 } else { 0.05 };
    cr.set_source_rgba(fg_r, fg_g, fg_b, inner_alpha);
    let _ = cr.fill();

    draw_rr(screen_x, screen_y, screen_w, screen_h, screen_r);
    cr.set_line_width(1.1);
    cr.set_source_rgba(fg_r, fg_g, fg_b, base_alpha);
    let _ = cr.stroke();

    let neck_x = screen_x + screen_w / 2.0;
    let neck_top_y = screen_y + screen_h;
    let neck_bot_y = neck_top_y + 2.2;
    cr.set_line_width(1.1);
    cr.move_to(neck_x, neck_top_y);
    cr.line_to(neck_x, neck_bot_y);
    cr.set_source_rgba(fg_r, fg_g, fg_b, base_alpha);
    let _ = cr.stroke();

    let base_y = neck_bot_y + 0.3;
    let base_half_w = 2.6;
    cr.set_line_cap(cairo::LineCap::Round);
    cr.move_to(neck_x - base_half_w, base_y);
    cr.line_to(neck_x + base_half_w, base_y);
    cr.set_source_rgba(fg_r, fg_g, fg_b, base_alpha);
    let _ = cr.stroke();
    cr.set_line_cap(cairo::LineCap::Butt);

    // Activity Indicator Dots (LEDs)
    let dot_r = 1.25;
    let dot_cx = (width - 2.0).max(14.8);
    let bot_dot_cy = base_y + 0.1;
    let top_dot_cy = bot_dot_cy - 3.4;

    let draw_led = |cx: f64, cy: f64, (r, g, b): (f64, f64, f64), is_lit: bool| {
        let alpha = if !is_connected {
            0.15
        } else if is_lit {
            0.95
        } else {
            0.18
        };

        if is_connected && is_lit {
            cr.new_sub_path();
            cr.arc(cx, cy, dot_r + 0.7, 0.0, 2.0 * PI);
            cr.set_source_rgba(r, g, b, 0.28);
            let _ = cr.fill();
        }

        cr.new_sub_path();
        cr.arc(cx, cy, dot_r, 0.0, 2.0 * PI);
        cr.set_source_rgba(r, g, b, alpha);
        let _ = cr.fill();
    };

    draw_led(dot_cx, top_dot_cy, (0.96, 0.62, 0.04), tx_lit);
    draw_led(dot_cx, bot_dot_cy, (0.13, 0.77, 0.36), rx_lit);
}

