//! Power actions shared by the shell, greeter and compositor.

/// System power operations exposed by the shell and greeter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PowerAction {
    /// Power the machine off.
    Shutdown = 0,
    /// Restart the machine.
    Reboot = 1,
    /// Suspend the machine.
    Sleep = 2,
    /// End the current user session.
    Logout = 3,
}

impl PowerAction {
    /// Resolves a power action from a zero-based menu index (0..4).
    ///
    /// # Arguments
    /// * `index` - Index of the action in the power menu.
    ///
    /// # Returns
    /// The matching action, or `None` when the index is out of range.
    pub fn from_index(index: usize) -> Option<Self> {
        match index {
            0 => Some(Self::Shutdown),
            1 => Some(Self::Reboot),
            2 => Some(Self::Sleep),
            3 => Some(Self::Logout),
            _ => None,
        }
    }

    /// Returns the `systemctl` verb for actions driven by systemd.
    ///
    /// # Returns
    /// `"poweroff"`, `"reboot"` or `"suspend"`; `None` for [`Self::Logout`].
    pub fn systemctl_arg(self) -> Option<&'static str> {
        match self {
            Self::Shutdown => Some("poweroff"),
            Self::Reboot => Some("reboot"),
            Self::Sleep => Some("suspend"),
            Self::Logout => None,
        }
    }

    /// Executes the corresponding system power command.
    ///
    /// Uses `systemctl` for shutdown, reboot and suspend. Logout terminates the
    /// current `XDG_SESSION_ID`, falling back to `pkill -f babydrade` when no
    /// session id is available.
    pub fn execute(self) {
        if let Some(arg) = self.systemctl_arg() {
            let _ = std::process::Command::new("systemctl").arg(arg).spawn();
            return;
        }

        let session = std::env::var("XDG_SESSION_ID").unwrap_or_default();
        if session.is_empty() {
            let _ = std::process::Command::new("pkill")
                .args(["-f", "babydrade"])
                .spawn();
        } else {
            let _ = std::process::Command::new("loginctl")
                .args(["terminate-session", &session])
                .spawn();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn menu_indexes_map_to_the_expected_actions() {
        assert_eq!(PowerAction::from_index(0), Some(PowerAction::Shutdown));
        assert_eq!(PowerAction::from_index(1), Some(PowerAction::Reboot));
        assert_eq!(PowerAction::from_index(2), Some(PowerAction::Sleep));
        assert_eq!(PowerAction::from_index(3), Some(PowerAction::Logout));
        assert_eq!(PowerAction::from_index(4), None);
    }

    #[test]
    fn systemctl_verbs_match_the_systemd_commands() {
        assert_eq!(PowerAction::Shutdown.systemctl_arg(), Some("poweroff"));
        assert_eq!(PowerAction::Reboot.systemctl_arg(), Some("reboot"));
        assert_eq!(PowerAction::Sleep.systemctl_arg(), Some("suspend"));
        assert_eq!(PowerAction::Logout.systemctl_arg(), None);
    }
}
