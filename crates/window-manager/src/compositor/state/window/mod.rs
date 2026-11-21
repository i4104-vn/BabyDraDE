pub(crate) mod actions;
pub(crate) mod animation;
pub(crate) mod sizing;
pub(crate) mod state_data;

#[allow(unused_imports)]
pub(crate) use animation::WindowAnimation;
pub(crate) use animation::{
    cancel_minimizing_animation, is_window_exiting, start_window_minimizing, start_window_opening,
    start_window_restoring, start_window_transition, window_animation_frame,
};
pub(crate) use state_data::window_state;
#[allow(unused_imports)]
pub(crate) use state_data::WindowStateData;
