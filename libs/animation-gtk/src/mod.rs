//! GTK widget animation helpers (frame-clock driven).
//!
//! Copied verbatim from the old `babydra-ui-kit` animation module so the
//! Dynamic Island and other GTK features can share the same implementation.

pub mod easing;
pub mod island;
pub mod slide;

pub use island::{
    current_anim_gen, island_animate_size, island_animate_width, island_zoom_in, island_zoom_out,
    next_anim_gen,
};
pub use slide::{
    slide_in, slide_in_cancelable, slide_out, slide_out_cb, slide_out_cb_cancelable, SlideDirection,
};
