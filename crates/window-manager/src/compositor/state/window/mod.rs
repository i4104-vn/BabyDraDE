pub(crate) mod actions;
pub(crate) mod animation;
pub(crate) mod sizing;
pub(crate) mod state_data;

#[allow(unused_imports)]
pub(crate) use animation::WindowAnimation;
pub(crate) use animation::{is_window_closing, start_window_opening, window_animation_factors};
pub(crate) use state_data::window_state;
#[allow(unused_imports)]
pub(crate) use state_data::WindowStateData;
