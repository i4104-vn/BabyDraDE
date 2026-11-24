//! Shared state for udev frame rendering.

use std::cell::RefMut;

use super::super::surface::UdevState;
use crate::adapters::{compositor::CalloopData, desktop::SwitcherOverlay};

/// Borrowed state required to render a single udev frame.
pub(super) struct FrameRenderContext<'a> {
    pub(super) data: &'a mut CalloopData,
    pub(super) udev: RefMut<'a, UdevState>,
    pub(super) switcher: &'a mut SwitcherOverlay,
}
