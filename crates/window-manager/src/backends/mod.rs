pub(crate) mod udev;
pub(crate) mod winit;

use smithay::reexports::calloop::EventLoop;
use std::error::Error;
use tracing::info;

use crate::compositor::CalloopData;

pub(crate) fn init(
    event_loop: &mut EventLoop<'_, CalloopData>,
    data: &mut CalloopData,
    host_has_display: bool,
) -> Result<(), Box<dyn Error>> {
    if host_has_display {
        info!("Active graphical host display detected. Initializing nested Winit backend.");
        winit::init(event_loop, data)
    } else {
        info!("No host display detected. Initializing autonomous Udev/DRM hardware backend for direct TTY scanout.");
        udev::init(event_loop, data)
    }
}
