mod handlers;
pub(crate) mod input;
mod state;

pub(crate) use state::{
    ease_in_out_cubic, AltTabEntry, AltTabSwitcher, BabydraState, CalloopData, ANIMATION_DURATION,
};
