//! Compatibility facade for the former `input::touch::events::*` paths.
//! The handlers themselves live in this file.

pub(crate) use super as dispatch;
pub(crate) use super::gesture_state;
pub(crate) use super::overlay;
pub(crate) use super::{
    handle_touch_cancel, handle_touch_down, handle_touch_frame, handle_touch_motion,
    handle_touch_up,
};
