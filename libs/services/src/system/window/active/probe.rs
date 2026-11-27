//! One-shot `wlrctl` probing and parsing of focused-window snapshots.

use std::io;
use std::process::Command;

use crate::system::window::ActiveWindowInfo;

/// Parses the first `wlrctl window list` line into a focused-window snapshot.
pub(crate) fn parse_wlrctl_line(stdout: &str) -> Option<ActiveWindowInfo> {
    let line = stdout.lines().next()?;
    let (app_id, title) = line.split_once(':')?;
    let app_id = app_id.trim().to_string();
    let title = title.trim().to_string();
    if app_id.is_empty() && title.is_empty() {
        return None;
    }
    Some(ActiveWindowInfo {
        app_id,
        title,
        process_id: None,
    })
}

/// One-shot `wlrctl` probe. Errors mean the tool itself could not be executed.
pub(super) fn probe_wlrctl() -> io::Result<Option<ActiveWindowInfo>> {
    let output = Command::new("wlrctl")
        .args(["window", "list", "state:focused"])
        .output()?;
    if !output.status.success() {
        return Ok(None);
    }
    Ok(parse_wlrctl_line(&String::from_utf8_lossy(&output.stdout)))
}

/// Outcome of a completed background fallback probe.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum ProbeResult {
    Snapshot(Option<ActiveWindowInfo>),
    Unavailable,
}
