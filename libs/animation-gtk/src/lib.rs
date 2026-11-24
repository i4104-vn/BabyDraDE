//! GTK widget animation helpers for BabyDraDE desktop features.
//!
//! These helpers need a GTK4 frame clock, so they live in a separate crate from
//! the framework-free [`animation`] primitives. Compositor code can depend on
//! the timing primitives without pulling in the GTK toolkit.

mod island;
mod slide;

pub use island::{
    current_anim_gen, island_animate_size, island_animate_width, island_zoom_in, island_zoom_out,
    next_anim_gen,
};
pub use slide::{
    slide_in, slide_in_cancelable, slide_out, slide_out_cb, slide_out_cb_cancelable, SlideDirection,
};
