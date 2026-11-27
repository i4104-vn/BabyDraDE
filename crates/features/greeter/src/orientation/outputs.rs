//! Wayland output-manager client: discovers heads and applies transforms.

use std::{collections::HashMap, error::Error, time::Instant};

use services::config::models::{DisplayConfig, MonitorOrientation};
use wayland_client::{
    globals::{registry_queue_init, GlobalListContents},
    protocol::{wl_output::Transform, wl_registry},
    Connection, Dispatch, EventQueue, QueueHandle,
};
use wayland_protocols_wlr::output_management::v1::client::{
    zwlr_output_configuration_head_v1::{Event as ConfigHeadEvent, ZwlrOutputConfigurationHeadV1},
    zwlr_output_configuration_v1::{Event as ConfigEvent, ZwlrOutputConfigurationV1},
    zwlr_output_head_v1::{self, Event as HeadEvent, ZwlrOutputHeadV1},
    zwlr_output_manager_v1::{self, Event as ManagerEvent, ZwlrOutputManagerV1},
    zwlr_output_mode_v1::ZwlrOutputModeV1,
};

use super::PROTOCOL_TIMEOUT;

#[derive(Default)]
pub(super) struct OutputState {
    /// Outputs announced by the compositor, paired with their reported names.
    pub(super) heads: Vec<(String, ZwlrOutputHeadV1)>,
    /// Head currently being described by the compositor.
    pending_head: Option<ZwlrOutputHeadV1>,
    /// Name announced for that head (the compositor sends it before `transform`).
    pending_name: Option<String>,
    /// Transform each named head reported at session start (rotation baseline).
    pub(super) transforms: HashMap<String, Transform>,
    /// Serial of the current output configuration, used by `create_configuration`.
    pub(super) serial: Option<u32>,
    /// The compositor finished describing the current outputs.
    done: bool,
    /// Result of the applied configuration.
    pub(super) result: Option<bool>,
}

impl Dispatch<wl_registry::WlRegistry, GlobalListContents> for OutputState {
    fn event(
        _state: &mut OutputState,
        _registry: &wl_registry::WlRegistry,
        _event: wl_registry::Event,
        _data: &GlobalListContents,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<ZwlrOutputManagerV1, ()> for OutputState {
    fn event(
        state: &mut OutputState,
        _manager: &ZwlrOutputManagerV1,
        event: ManagerEvent,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        match event {
            ManagerEvent::Head { head } => state.pending_head = Some(head),
            ManagerEvent::Done { serial } => {
                state.serial = Some(serial);
                state.done = true;
            }
            ManagerEvent::Finished => state.done = true,
            _ => {}
        }
    }

    wayland_client::event_created_child!(OutputState, ZwlrOutputManagerV1, [
        zwlr_output_manager_v1::EVT_HEAD_OPCODE => (ZwlrOutputHeadV1, ())
    ]);
}

impl Dispatch<ZwlrOutputHeadV1, ()> for OutputState {
    fn event(
        state: &mut OutputState,
        _head: &ZwlrOutputHeadV1,
        event: HeadEvent,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        match event {
            HeadEvent::Name { name } => {
                if let Some(head) = state.pending_head.take() {
                    state.pending_name = Some(name.clone());
                    state.heads.push((name, head));
                }
            }
            HeadEvent::Transform { transform } => {
                // The transform arrives after the name in wlr's event order,
                // so record the baseline here instead of at `Name`.
                let transform = transform.into_result().unwrap_or(Transform::Normal);
                if let Some(name) = state.pending_name.take() {
                    state.transforms.insert(name, transform);
                }
            }
            _ => {}
        }
    }

    wayland_client::event_created_child!(OutputState, ZwlrOutputHeadV1, [
        zwlr_output_head_v1::EVT_MODE_OPCODE => (ZwlrOutputModeV1, ())
    ]);
}

impl Dispatch<ZwlrOutputModeV1, ()> for OutputState {
    fn event(
        _state: &mut OutputState,
        _mode: &ZwlrOutputModeV1,
        _event: wayland_protocols_wlr::output_management::v1::client::zwlr_output_mode_v1::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<ZwlrOutputConfigurationHeadV1, ()> for OutputState {
    fn event(
        _state: &mut OutputState,
        _head: &ZwlrOutputConfigurationHeadV1,
        _event: ConfigHeadEvent,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<ZwlrOutputConfigurationV1, ()> for OutputState {
    fn event(
        state: &mut OutputState,
        _config: &ZwlrOutputConfigurationV1,
        event: ConfigEvent,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        state.result = Some(match event {
            ConfigEvent::Succeeded => true,
            ConfigEvent::Failed | ConfigEvent::Cancelled => false,
            _ => return,
        });
    }
}

/// A connected output manager with its initial head descriptions.
pub(super) struct OutputConnection {
    pub(super) connection: Connection,
    pub(super) queue: EventQueue<OutputState>,
    pub(super) qh: QueueHandle<OutputState>,
    pub(super) manager: ZwlrOutputManagerV1,
    pub(super) state: OutputState,
    pub(super) serial: u32,
}

/// Connects to the compositor and waits for its output descriptions.
///
/// # Returns
/// The connected protocol objects and the serial describing current outputs.
///
/// # Errors
/// Returns an error when the connection, manager or output announcement fails.
pub(super) fn connect_outputs() -> Result<OutputConnection, Box<dyn Error>> {
    let connection = Connection::connect_to_env()?;
    let (globals, mut queue) = registry_queue_init::<OutputState>(&connection)?;
    let qh = queue.handle();
    let manager: ZwlrOutputManagerV1 = globals.bind(&qh, 1..=2, ())?;

    let mut state = OutputState::default();
    let deadline = Instant::now() + PROTOCOL_TIMEOUT;
    while !state.done && Instant::now() < deadline {
        dispatch(&connection, &mut queue, &mut state)?;
    }
    if !state.done {
        return Err("compositor did not announce its outputs".into());
    }
    let serial = state.serial.ok_or("missing output configuration serial")?;
    let names: Vec<&str> = state.heads.iter().map(|(name, _)| name.as_str()).collect();
    tracing::info!("greeter: compositor outputs: {names:?}");

    Ok(OutputConnection {
        connection,
        queue,
        qh,
        manager,
        state,
        serial,
    })
}

/// Connects to the compositor and applies the configured rotations.
///
/// # Returns
/// The number of outputs whose rotation was changed.
///
/// # Errors
/// Returns an error when the Wayland connection or output manager is
/// unavailable, or when the compositor rejects the configuration.
pub(super) fn apply_protocol(config: &DisplayConfig) -> Result<usize, Box<dyn Error>> {
    let mut outputs = connect_outputs()?;
    let serial = outputs.serial;
    let configuration = outputs
        .manager
        .create_configuration(serial, &outputs.qh, ());
    let mut applied = 0;
    for (name, head) in &outputs.state.heads {
        // A configuration must mention every head, even ones left untouched.
        let head_config = configuration.enable_head(head, &outputs.qh, ());
        if let Some(orientation) = config.orientation_for(name) {
            head_config.set_transform(to_wl_transform(orientation));
            applied += 1;
        }
    }
    configuration.apply();

    let deadline = Instant::now() + PROTOCOL_TIMEOUT;
    while outputs.state.result.is_none() && Instant::now() < deadline {
        dispatch(&outputs.connection, &mut outputs.queue, &mut outputs.state)?;
    }
    match outputs.state.result {
        Some(true) => Ok(applied),
        Some(false) => Err("compositor rejected the output configuration".into()),
        None => Err("no answer from the compositor".into()),
    }
}

/// Dispatches at most one non-blocking batch of Wayland events.
///
/// The caller owns the deadline loop, so a stalled compositor can never keep
/// the greeter's login UI from appearing.
fn dispatch(
    connection: &Connection,
    queue: &mut EventQueue<OutputState>,
    state: &mut OutputState,
) -> Result<(), Box<dyn Error>> {
    connection.flush()?;
    if let Some(guard) = queue.prepare_read() {
        match guard.read() {
            Ok(_) => {}
            // A non-blocking socket has nothing available right now.
            Err(wayland_client::backend::WaylandError::Io(err))
                if err.kind() == std::io::ErrorKind::WouldBlock => {}
            Err(err) => return Err(Box::new(err)),
        }
    }
    queue.dispatch_pending(state)?;
    std::thread::sleep(std::time::Duration::from_millis(10));
    Ok(())
}

/// Converts a configured orientation to the protocol's transform value.
fn to_wl_transform(orientation: MonitorOrientation) -> Transform {
    match orientation {
        MonitorOrientation::Normal => Transform::Normal,
        MonitorOrientation::Rotate90 => Transform::_90,
        MonitorOrientation::Rotate180 => Transform::_180,
        MonitorOrientation::Rotate270 => Transform::_270,
        MonitorOrientation::Flipped => Transform::Flipped,
        MonitorOrientation::Flipped90 => Transform::Flipped90,
        MonitorOrientation::Flipped180 => Transform::Flipped180,
        MonitorOrientation::Flipped270 => Transform::Flipped270,
    }
}
