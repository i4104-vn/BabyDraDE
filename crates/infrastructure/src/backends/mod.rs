pub(crate) mod udev;
pub(crate) mod winit;

use std::error::Error;
use smithay::reexports::calloop::EventLoop;
use tracing::info;

use crate::compositor::CalloopData;

pub(crate) fn init(
    event_loop: &mut EventLoop<CalloopData>,
    data: &mut CalloopData,
) -> Result<(), Box<dyn Error>> {
    if std::env::var_os("WAYLAND_DISPLAY").is_some() || std::env::var_os("DISPLAY").is_some() {
        info!("Active graphical host display detected ($WAYLAND_DISPLAY or $DISPLAY is set). Initializing nested Winit backend.");
        winit::init(event_loop, data)
    } else {
        info!("No host display detected. Initializing autonomous Udev/DRM hardware backend for direct TTY scanout.");
        udev::init(event_loop, data)
    }
}
