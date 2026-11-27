//! Active-window tracking over the protocol bus, with a `wlrctl` fallback.

mod probe;
mod state;
mod watcher;

#[cfg(test)]
pub(crate) use probe::parse_wlrctl_line;
pub use state::get_active_window;
#[cfg(test)]
pub(crate) use state::{fallback_probe_due, throttle_due};
pub(super) use watcher::send_request;
