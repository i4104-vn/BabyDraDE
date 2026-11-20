//! Cairo Drawing helpers for BabyDraDE

pub mod battery;
pub mod chart;
pub mod ethernet;
pub mod image;

pub use battery::{draw_cairo_battery, get_battery_rgb};
pub use image::{apply_circular_mask, crop_circle, crop_square, crop_square_pixbuf};
pub use chart::draw_metric_chart;
pub use ethernet::draw_cairo_ethernet_desktop;
