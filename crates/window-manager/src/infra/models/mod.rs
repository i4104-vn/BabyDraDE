//! Infrastructure device and configuration models.
//!
//! Backend behaviour lives in `crate::infra::udev`; the state it drives is
//! defined here and re-exported so existing paths keep resolving.

pub(crate) mod config;
pub(crate) mod device;

pub(crate) use config::{FrameRenderContext, GpuActivation, GpuInitResult};
pub(crate) use device::GpuContext;
pub(crate) use device::{SurfaceData, UdevSession, UdevState};
