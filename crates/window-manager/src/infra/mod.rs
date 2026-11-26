//! Infrastructure adapters: backends, config loaders and process drivers.

pub(crate) mod frame_pacing;
pub(crate) mod models;
pub(crate) mod process;
pub(crate) mod udev;
pub(crate) mod winit;

use smithay::reexports::calloop::EventLoop;
use std::error::Error;
use tracing::info;

use crate::app::state::CalloopData;

/// Starts the backend that matches the host's display setup.
///
/// # Arguments
/// * `event_loop` - Compositor event loop that receives backend sources.
/// * `data` - Shared loop data handed to the backend.
/// * `host_has_display` - Selects the nested Winit backend over Udev/DRM.
///
/// # Returns
/// `Ok(())` once the backend sources are registered.
///
/// # Errors
/// Returns the chosen backend's initialisation error.
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
