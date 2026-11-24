//! Directional slide-in / slide-out widget animations.

mod enter;
mod exit;
mod handler;

pub use enter::{slide_in, slide_in_cancelable};
pub use exit::{slide_out, slide_out_cb, slide_out_cb_cancelable};
pub use handler::SlideDirection;
