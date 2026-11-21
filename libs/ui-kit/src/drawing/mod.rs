//! Cairo Drawing helpers for BabyDraDE

pub mod battery;
pub mod cairo;
pub mod chart;
pub mod ethernet;
pub mod image;

pub use battery::{draw_cairo_battery, get_battery_rgb};
pub use cairo::{
    app_color, docked_panel_path, draw_docked_panel, draw_text, fit_text, paint_rounded,
    rounded_rect, set_bold_font,
};
pub use chart::draw_metric_chart;
pub use common::DockSide;
pub use ethernet::draw_cairo_ethernet_desktop;
pub use image::{apply_circular_mask, crop_circle, crop_square, crop_square_pixbuf};
