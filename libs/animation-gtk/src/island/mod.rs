//! Dynamic Island capsule size / zoom transitions.

mod gen;
mod handler;
mod resize;
mod zoom;

pub use gen::{current_anim_gen, next_anim_gen};
pub use resize::{island_animate_size, island_animate_width};
pub use zoom::{island_zoom_in, island_zoom_out};
