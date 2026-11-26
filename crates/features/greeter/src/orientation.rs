//! Applies fixed monitor orientations to the greetd/cage session.
//!
//! The greeter runs before the user logs in, so it discovers the display config
//! on disk and sends each rotation to cage over `zwlr_output_manager_v1`. No
//! external tool such as `wlr-randr` is needed.

use std::{
    error::Error,
    path::{Path, PathBuf},
    time::{Duration, Instant, SystemTime},
};

use services::config::{load_display_config_from, models::{DisplayConfig, MonitorOrientation}};
use wayland_client::{
    globals::{registry_queue_init, GlobalListContents},
    protocol::{wl_output::Transform, wl_registry},
    Connection, Dispatch, EventQueue, QueueHandle,
};
use wayland_protocols_wlr::output_management::v1::client::{
    zwlr_output_configuration_head_v1::{
        Event as ConfigHeadEvent, ZwlrOutputConfigurationHeadV1,
    },
    zwlr_output_configuration_v1::{Event as ConfigEvent, ZwlrOutputConfigurationV1},
    zwlr_output_head_v1::{self, Event as HeadEvent, ZwlrOutputHeadV1},
    zwlr_output_mode_v1::ZwlrOutputModeV1,
    zwlr_output_manager_v1::{self, Event as ManagerEvent, ZwlrOutputManagerV1},
};

/// Upper bound for each protocol round-trip, so a stalled compositor cannot
/// keep the login screen from appearing.
const PROTOCOL_TIMEOUT: Duration = Duration::from_secs(2);

/// Applies every configured monitor rotation to the current Wayland session.
///
/// Errors are logged and ignored: orientation must never block the login UI.
pub(crate) fn apply() {
    let Some(path) = find_config() else {
        tracing::warn!("greeter: no readable display.toml found; keeping default orientation");
        return;
    };
    let config = load_config(&path);
    match apply_protocol(&config) {
        Ok(applied) => {
            tracing::info!("greeter: applied orientation to {applied} output(s)");
        }
        Err(error) => {
            tracing::warn!("greeter: cannot set orientation over Wayland: {error}");
        }
    }
}

// Wayland protocol client state.

#[derive(Default)]
struct OutputState {
    /// Outputs announced by the compositor, paired with their reported names.
    heads: Vec<(String, ZwlrOutputHeadV1)>,
    /// Head currently being described by the compositor.
    pending_head: Option<ZwlrOutputHeadV1>,
    /// Serial of the current output configuration, used by `create_configuration`.
    serial: Option<u32>,
    /// The compositor finished describing the current outputs.
    done: bool,
    /// Result of the applied configuration.
    result: Option<bool>,
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
        if let HeadEvent::Name { name } = event {
            if let Some(head) = state.pending_head.take() {
                state.heads.push((name, head));
            }
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

/// Connects to the compositor and applies the configured rotations.
///
/// # Returns
/// The number of outputs whose rotation was changed.
///
/// # Errors
/// Returns an error when the Wayland connection or output manager is
/// unavailable, or when the compositor rejects the configuration.
fn apply_protocol(config: &DisplayConfig) -> Result<usize, Box<dyn Error>> {
    let connection = Connection::connect_to_env()?;
    let (globals, mut queue) = registry_queue_init::<OutputState>(&connection)?;
    let qh = queue.handle();
    let manager: ZwlrOutputManagerV1 = globals.bind(&qh, 1..=2, ())?;

    let mut state = OutputState::default();
    let deadline = Instant::now() + PROTOCOL_TIMEOUT;
    while !state.done && Instant::now() < deadline {
        dispatch(&mut queue, &mut state)?;
    }
    if !state.done {
        return Err("compositor did not announce its outputs".into());
    }

    let names: Vec<&str> = state.heads.iter().map(|(name, _)| name.as_str()).collect();
    tracing::info!("greeter: compositor outputs: {names:?}");

    let serial = state.serial.ok_or("missing output configuration serial")?;
    let configuration = manager.create_configuration(serial, &qh, ());
    let mut applied = 0;
    for (name, head) in &state.heads {
        // A configuration must mention every head, even ones left untouched.
        let head_config = configuration.enable_head(head, &qh, ());
        if let Some(orientation) = config.orientation_for(name) {
            head_config.set_transform(to_wl_transform(orientation));
            applied += 1;
        }
    }
    configuration.apply();

    let deadline = Instant::now() + PROTOCOL_TIMEOUT;
    while state.result.is_none() && Instant::now() < deadline {
        dispatch(&mut queue, &mut state)?;
    }
    match state.result {
        Some(true) => Ok(applied),
        Some(false) => Err("compositor rejected the output configuration".into()),
        None => Err("no answer from the compositor".into()),
    }
}

/// Dispatches one batch of Wayland events.
fn dispatch(queue: &mut EventQueue<OutputState>, state: &mut OutputState) -> Result<(), Box<dyn Error>> {
    queue.blocking_dispatch(state)?;
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

// Display config discovery.

/// Locates the most recently modified readable display config.
///
/// The greeter's home is usually `/`, so the last logged-in user's config is
/// preferred, with any user under `/home` as a fallback.
fn find_config() -> Option<PathBuf> {
    let mut candidates = Vec::new();
    if let Some(user) = crate::widgets::get_last_user() {
        if valid_username(&user) {
            if let Some(home) = user_home(&user, &std::fs::read_to_string("/etc/passwd").unwrap_or_default()) {
                candidates.push(home);
            }
        }
    }
    candidates.push(services::config::get_home_dir());
    if let Ok(entries) = std::fs::read_dir("/home") {
        candidates.extend(entries.flatten().map(|entry| entry.path()).filter(|path| path.is_dir()));
    }

    let mut best: Option<(SystemTime, PathBuf)> = None;
    for home in candidates {
        let path = home.join(".babydra/configs/display.toml");
        let Ok(metadata) = std::fs::metadata(&path) else {
            continue;
        };
        let Ok(modified) = metadata.modified() else {
            continue;
        };
        if best.as_ref().is_none_or(|(time, _)| modified > *time) {
            best = Some((modified, path));
        }
    }
    best.map(|(_, path)| path).map(|path| {
        tracing::info!("greeter: using display config {path:?}");
        path
    })
}

/// Loads the display config from a resolved path.
fn load_config(path: &Path) -> DisplayConfig {
    load_display_config_from(path)
}

/// Returns true for names that are safe to match in the passwd database.
fn valid_username(user: &str) -> bool {
    !user.is_empty()
        && user
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
}

/// Resolves a user's home directory from passwd content.
fn user_home(user: &str, passwd: &str) -> Option<PathBuf> {
    for line in passwd.lines() {
        let fields: Vec<&str> = line.split(':').collect();
        if fields.first() == Some(&user) && fields.len() > 5 {
            return Some(PathBuf::from(fields[5]));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::user_home;

    #[test]
    fn user_home_reads_the_sixth_passwd_field() {
        let passwd = "root:x:0:0:root:/root:/bin/bash\nuser:x:1000:1000::/home/user:/bin/zsh\n";
        assert_eq!(user_home("user", passwd), Some(std::path::PathBuf::from("/home/user")));
        assert_eq!(user_home("missing", passwd), None);
    }
}
