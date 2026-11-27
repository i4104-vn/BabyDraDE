//! Ports — the boundaries through which use cases reach the outside world.
//!
//! Adapters and infrastructure implement these; the application layer itself
//! only ever sees the plain data and function types declared here.

pub(crate) mod input_ports;
pub(crate) mod usecase;

pub(crate) use crate::core::models::policy::GesturePolicy;
pub(crate) use crate::shared::models::ports::{ConfigError, LaunchError};

pub(crate) use input_ports::{
    CaretSelection, CaretUnit, HitTestQuery, InputContext, InteractionBuffer, OverlayBridge,
    QueryText, SeatSink, SystemActions, WindowControl,
};
pub(crate) use usecase::{
    AutostartLauncher, CommandLauncher, LoadGesturePolicy, RotationTarget, ShortcutRepository,
};
