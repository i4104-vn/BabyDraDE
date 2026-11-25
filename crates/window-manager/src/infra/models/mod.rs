//! Infrastructure device and configuration models.
//!
//! Backend behaviour lives in `crate::infra::udev`; the state it drives is
//! defined here and re-exported so existing paths keep resolving.

#[allow(unused_imports)]
pub(crate) mod config;
pub(crate) mod device;

pub(crate) use config::{FrameRenderContext, GpuActivation, GpuInitResult};
pub(crate) use device::GpuContext;
#[allow(unused_imports)]
pub(crate) use device::{SurfaceData, UdevDrmCompositor, UdevSession, UdevState};
