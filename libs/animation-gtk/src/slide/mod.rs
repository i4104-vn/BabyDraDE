//! Directional slide-in / slide-out widget animations.

mod handler;
mod render;

pub use handler::SlideDirection;
pub use render::{
    slide_in, slide_in_cancelable, slide_out, slide_out_cb, slide_out_cb_cancelable,
};
