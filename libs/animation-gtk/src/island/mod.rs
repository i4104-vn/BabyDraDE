//! Dynamic Island capsule size / zoom transitions.

mod handler;
mod render;

pub use render::{
    current_anim_gen, island_animate_size, island_animate_width, island_zoom_in, island_zoom_out,
    next_anim_gen,
};
