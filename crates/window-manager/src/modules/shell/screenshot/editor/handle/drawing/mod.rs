//! Drawing tools and rendering.

mod handler;
mod render;

pub(crate) use handler::{save_cropped_surface, trigger_save};
pub(crate) use render::{draw_pixelated_rect, render_shape};
