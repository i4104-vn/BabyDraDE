//! Directional slide-in / slide-out widget animations.

mod handler;
mod enter;
mod exit;

pub use handler::SlideDirection;
pub use enter::{slide_in, slide_in_cancelable};
pub use exit::{slide_out, slide_out_cb, slide_out_cb_cancelable};
