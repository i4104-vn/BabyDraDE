//! Cairo battery drawing helper.

use gtk4::cairo;

/// Charging / healthy fill: green (#22c55e).
const GREEN: (f64, f64, f64) = (0.13, 0.77, 0.36);
/// Mid-level warning fill: amber (#f59e0b).
const AMBER: (f64, f64, f64) = (0.96, 0.62, 0.04);
/// Low-level warning fill: red (#ef4444).
const RED: (f64, f64, f64) = (0.94, 0.27, 0.27);
/// Normal (discharging) fill: white (#ffffff).
const WHITE: (f64, f64, f64) = (1.0, 1.0, 1.0);
/// Neutral foreground used on light themes (also used for the shell and tip).
const LIGHT_INK: (f64, f64, f64) = (0.15, 0.15, 0.15);

/// Battery fill color.
///
/// * Charging: always green, regardless of percentage (the lightning bolt is
///   drawn separately in [`draw_cairo_battery`]).
/// * Discharging: white when normal/high (`> 30%`), amber for mid
///   (`16..=30%`), red when low (`<= 15%`).
pub fn get_battery_rgb(percentage: u32, is_charging: bool) -> (f64, f64, f64) {
    if is_charging {
        return GREEN;
    }

    match percentage {
        0..=15 => RED,
        16..=30 => AMBER,
        _ => WHITE,
    }
}

/// Fill color actually painted, accounting for the theme.
///
/// Pure white would be invisible on a light panel, so the "normal" level uses
/// the theme's neutral foreground instead (dark ink on light themes, white on
/// dark themes). Charging green and the amber/red warnings stay untouched.
fn fill_rgb(percentage: u32, is_charging: bool, is_dark: bool) -> (f64, f64, f64) {
    let rgb = get_battery_rgb(percentage, is_charging);
    if rgb == WHITE && !is_dark {
        LIGHT_INK
    } else {
        rgb
    }
}

pub fn draw_cairo_battery(
    cr: &cairo::Context,
    width: f64,
    height: f64,
    percentage: u32,
    is_charging: bool,
    is_dark: bool,
) {
    let (fg_r, fg_g, fg_b) = if is_dark { WHITE } else { LIGHT_INK };

    let (fill_r, fill_g, fill_b) = fill_rgb(percentage, is_charging, is_dark);

    let shell_x = 1.5;
    let shell_y = 1.5;
    let tip_w = (width * 0.10).clamp(2.0, 6.0);
    let shell_w = width - tip_w - 3.0;
    let shell_h = height - 3.0;
    let corner_r = (shell_h * 0.2).clamp(1.5, 6.0);

    let draw_rr = |x: f64, y: f64, rw: f64, rh: f64, r: f64| {
        let deg = std::f64::consts::PI / 180.0;
        cr.new_sub_path();
        cr.arc(x + rw - r, y + r, r, -90.0 * deg, 0.0 * deg);
        cr.arc(x + rw - r, y + rh - r, r, 0.0 * deg, 90.0 * deg);
        cr.arc(x + r, y + rh - r, r, 90.0 * deg, 180.0 * deg);
        cr.arc(x + r, y + r, r, 180.0 * deg, 270.0 * deg);
        cr.close_path();
    };

    // 1. Draw outer shell border
    draw_rr(shell_x, shell_y, shell_w, shell_h, corner_r);
    cr.set_line_width((height * 0.08).clamp(1.5, 3.0));
    cr.set_source_rgba(fg_r, fg_g, fg_b, 0.85);
    let _ = cr.stroke();

    // 2. Draw terminal tip on right
    let tip_x = shell_x + shell_w + 1.5;
    let tip_h = shell_h * 0.45;
    let tip_y = shell_y + (shell_h - tip_h) / 2.0;
    draw_rr(tip_x, tip_y, tip_w, tip_h, 1.5);
    cr.set_source_rgba(fg_r, fg_g, fg_b, 0.85);
    let _ = cr.fill();

    // 3. Draw inner fill bar
    let pad = (height * 0.10).clamp(2.0, 5.0);
    let fill_max_w = shell_w - (pad * 2.0);
    let fill_h = shell_h - (pad * 2.0);
    let fill_w = (fill_max_w * (percentage as f64 / 100.0)).clamp(2.0, fill_max_w);

    draw_rr(
        shell_x + pad,
        shell_y + pad,
        fill_w,
        fill_h,
        (corner_r - 1.0).max(1.0),
    );
    cr.set_source_rgba(fill_r, fill_g, fill_b, 0.95);
    let _ = cr.fill();

    // 4. Draw lightning bolt if charging
    if is_charging {
        let cx = shell_x + shell_w / 2.0;
        let cy = shell_y + shell_h / 2.0;
        let scale = (height / 24.0).clamp(0.5, 2.5);
        cr.new_path();
        cr.move_to(cx + 1.0 * scale, cy - 5.0 * scale);
        cr.line_to(cx - 3.0 * scale, cy + 1.0 * scale);
        cr.line_to(cx - 0.3 * scale, cy + 1.0 * scale);
        cr.line_to(cx - 1.0 * scale, cy + 5.0 * scale);
        cr.line_to(cx + 3.0 * scale, cy - 1.0 * scale);
        cr.line_to(cx + 0.3 * scale, cy - 1.0 * scale);
        cr.close_path();

        cr.set_source_rgba(1.0, 1.0, 1.0, 0.95);
        let _ = cr.fill();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn charging_is_always_green() {
        for pct in [0, 15, 16, 30, 31, 50, 100, 120] {
            assert_eq!(get_battery_rgb(pct, true), GREEN, "pct = {}", pct);
        }
    }

    #[test]
    fn discharging_colors_follow_thresholds() {
        // Low: 0..=15% -> red.
        for pct in [0, 1, 14, 15] {
            assert_eq!(get_battery_rgb(pct, false), RED, "pct = {}", pct);
        }
        // Mid: 16..=30% -> amber.
        for pct in [16, 17, 29, 30] {
            assert_eq!(get_battery_rgb(pct, false), AMBER, "pct = {}", pct);
        }
        // Normal/high: > 30% -> white.
        for pct in [31, 32, 50, 100, 120] {
            assert_eq!(get_battery_rgb(pct, false), WHITE, "pct = {}", pct);
        }
    }

    #[test]
    fn threshold_boundaries_are_exact() {
        assert_eq!(get_battery_rgb(15, false), RED);
        assert_eq!(get_battery_rgb(16, false), AMBER);
        assert_eq!(get_battery_rgb(30, false), AMBER);
        assert_eq!(get_battery_rgb(31, false), WHITE);
    }

    #[test]
    fn theme_only_adjusts_the_white_normal_level() {
        assert_eq!(fill_rgb(80, false, true), WHITE);
        assert_eq!(fill_rgb(80, false, false), LIGHT_INK);

        for dark in [true, false] {
            assert_eq!(fill_rgb(10, false, dark), RED);
            assert_eq!(fill_rgb(20, false, dark), AMBER);
            assert_eq!(fill_rgb(60, true, dark), GREEN);
        }
    }

    #[test]
    fn battery_colors_are_distinct_and_bounded() {
        for color in [GREEN, AMBER, RED, WHITE] {
            for channel in [color.0, color.1, color.2] {
                assert!((0.0..=1.0).contains(&channel));
            }
        }
        assert_ne!(GREEN, WHITE);
        assert_ne!(AMBER, RED);
        assert_ne!(AMBER, WHITE);
        assert_ne!(RED, WHITE);
    }
}
