//! Backend device models: libseat session, surfaces and GPU context.

use std::{error::Error, path::PathBuf};

use smithay::{
    backend::{
        allocator::{
            gbm::{GbmAllocator, GbmDevice},
            Format,
        },
        drm::{
            compositor::DrmCompositor, exporter::gbm::GbmFramebufferExporter, DrmDevice,
            DrmDeviceFd, DrmDeviceNotifier,
        },
        renderer::glow::GlowRenderer,
        session::libseat::{LibSeatSession, LibSeatSessionNotifier},
    },
    reexports::calloop::EventLoop,
};

use services::theme::DesktopTheme;

use crate::{app::state::CalloopData, modules::window::cursor_impl::Cursor};

/// Claimed LibSeat session with its notifier and libinput setup applied.
pub(crate) struct UdevSession {
    pub(crate) session: LibSeatSession,
    pub(crate) notifier: LibSeatSessionNotifier,
    pub(crate) seat_name: String,
}

impl UdevSession {
    pub(crate) fn new(event_loop: &mut EventLoop<'_, CalloopData>) -> Result<Self, Box<dyn Error>> {
        let (session, notifier, seat_name) = crate::infra::udev::context::claim_seat(event_loop)?;
        Ok(Self {
            session,
            notifier,
            seat_name,
        })
    }
}

/// Compositor type used by a Udev/DRM output.
pub(crate) type UdevDrmCompositor =
    DrmCompositor<GbmAllocator<DrmDeviceFd>, GbmFramebufferExporter<DrmDeviceFd>, (), DrmDeviceFd>;

/// One scanned output and its DRM compositor.
pub(crate) struct SurfaceData {
    pub(crate) compositor: UdevDrmCompositor,
    pub(crate) output: smithay::output::Output,
}

/// Shared Udev/DRM backend state borrowable by frame renderers.
pub(crate) struct UdevState {
    pub(crate) surfaces: Vec<SurfaceData>,
    pub(crate) renderer: GlowRenderer,
    pub(crate) theme: DesktopTheme,
    pub(crate) cursor: Cursor,
    pub(crate) active: bool,
    pub(crate) _drm_device: DrmDevice,
    pub(crate) _gbm_device: GbmDevice<DrmDeviceFd>,
    pub(crate) _session: LibSeatSession,
}

impl Drop for UdevState {
    fn drop(&mut self) {
        // Explicitly pause DRM device before dropping to avoid AtomicDrmDevice trying to
        // commit/restore previous state on an invalid or unprivileged FD on teardown.
        self._drm_device.pause();
    }
}

/// State shared across the GPU initialization stages.
pub(crate) struct GpuContext {
    pub(crate) path: PathBuf,
    pub(crate) drm_device: DrmDevice,
    pub(crate) drm_notifier: DrmDeviceNotifier,
    pub(crate) gbm_device: GbmDevice<DrmDeviceFd>,
    pub(crate) allocator: GbmAllocator<DrmDeviceFd>,
    pub(crate) exporter: GbmFramebufferExporter<DrmDeviceFd>,
    pub(crate) renderer: GlowRenderer,
    pub(crate) renderer_formats: Vec<Format>,
}
