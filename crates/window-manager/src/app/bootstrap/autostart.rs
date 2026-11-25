//! Autostart use case.
//!
//! Decides which configured commands to run and delegates the actual process
//! spawning to an injected [`AutostartLauncher`]. No process, framework or
//! filesystem APIs are used here, so the rules can be tested with a fake port.

use std::ffi::OsStr;
use tracing::{info, warn};

use crate::shared::ports::AutostartLauncher;

/// Raw config lines that describe a command to start.
///
/// Blank lines and lines whose first non-whitespace character is `#` are
/// comments and are skipped.
pub(crate) fn enabled_commands(lines: &[String]) -> impl Iterator<Item = &str> + '_ {
    lines
        .iter()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
}

/// Starts every enabled command and returns the handles of those that started
/// successfully, so the composition root can terminate them on shutdown.
pub(crate) fn start<L: AutostartLauncher>(
    commands: &[String],
    launcher: &L,
    wayland_display: &OsStr,
) -> Vec<L::Child> {
    let mut children = Vec::new();
    for command in enabled_commands(commands) {
        info!("Spawning autostart command: {command}");
        match launcher.spawn(command, wayland_display) {
            Ok(child) => children.push(child),
            Err(err) => warn!("Failed to spawn autostart command '{command}': {err}"),
        }
    }
    children
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::ffi::OsString;

    use super::*;

    /// Fake port: records every launch and can be told to reject a command.
    #[derive(Default)]
    struct FakeLauncher {
        calls: RefCell<Vec<(String, OsString)>>,
        fail_on: Option<&'static str>,
    }

    impl AutostartLauncher for FakeLauncher {
        type Child = &'static str;

        fn spawn(&self, command: &str, wayland_display: &OsStr) -> Result<Self::Child, String> {
            self.calls
                .borrow_mut()
                .push((command.to_owned(), wayland_display.to_owned()));
            if self.fail_on == Some(command) {
                Err("fake spawn failure".to_owned())
            } else {
                Ok("started")
            }
        }
    }

    fn lines(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| (*value).to_owned()).collect()
    }

    #[test]
    fn blank_lines_are_filtered() {
        let commands = lines(&["", "   ", "\t\n", "real-command"]);
        let enabled: Vec<_> = enabled_commands(&commands).collect();
        assert_eq!(enabled, ["real-command"]);
    }

    #[test]
    fn comment_lines_are_filtered() {
        let commands = lines(&["# comment", "   # indented comment", "real-command"]);
        let enabled: Vec<_> = enabled_commands(&commands).collect();
        assert_eq!(enabled, ["real-command"]);
    }

    #[test]
    fn commands_are_trimmed() {
        let commands = lines(&["  spaced command \t", "  \t# comment  ", "  "]);
        let enabled: Vec<_> = enabled_commands(&commands).collect();
        assert_eq!(enabled, ["spaced command"]);
    }

    #[test]
    fn mixed_input_keeps_only_valid_commands_in_order() {
        let commands = lines(&[
            "# banner",
            "first",
            "",
            "  # note",
            "  second  ",
            "\t",
            "third # not a comment",
        ]);
        let enabled: Vec<_> = enabled_commands(&commands).collect();
        assert_eq!(enabled, ["first", "second", "third # not a comment"]);
    }

    #[test]
    fn launcher_is_called_only_for_valid_commands() {
        let commands = lines(&["# comment", " one ", "", "two"]);
        let launcher = FakeLauncher::default();
        let display = OsStr::new("babydra-0");

        let children = start(&commands, &launcher, display);

        assert_eq!(children, ["started", "started"]);
        let calls = launcher.calls.borrow();
        assert_eq!(
            *calls,
            vec![
                ("one".to_owned(), OsString::from(display)),
                ("two".to_owned(), OsString::from(display)),
            ]
        );
    }

    #[test]
    fn launch_failures_are_skipped() {
        let commands = lines(&["ok", "bad", "also ok"]);
        let launcher = FakeLauncher {
            calls: RefCell::new(Vec::new()),
            fail_on: Some("bad"),
        };

        let children = start(&commands, &launcher, OsStr::new("babydra-0"));

        assert_eq!(children.len(), 2);
        assert_eq!(launcher.calls.borrow().len(), 3);
    }
}
