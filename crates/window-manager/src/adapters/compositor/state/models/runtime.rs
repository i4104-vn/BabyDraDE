//! Shared runtime resources owned by the compositor state.

use std::{ffi::OsString, time::Instant};

use smithay::{desktop::PopupManager, reexports::calloop::LoopSignal};

use crate::adapters::compositor::state::RenderState;

/// Long-lived resources shared by the event loop, backends and renderers.
pub(crate) struct RuntimeResources {
    pub(crate) start_time: Instant,
    pub(crate) socket_name: OsString,
    pub(crate) loop_signal: LoopSignal,
    pub(crate) popups: PopupManager,
    pub(crate) render: RenderState,
}
