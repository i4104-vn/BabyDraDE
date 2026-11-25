//! Power action model for the power island feature.

/// Supported power management actions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PowerAction {
    Shutdown = 0,
    Reboot = 1,
    Sleep = 2,
    Logout = 3,
}

impl PowerAction {
    /// Resolves a power action from a zero-based menu index (0..4).
    pub fn from_index(index: usize) -> Option<Self> {
        match index {
            0 => Some(Self::Shutdown),
            1 => Some(Self::Reboot),
            2 => Some(Self::Sleep),
            3 => Some(Self::Logout),
            _ => None,
        }
    }

    /// Executes the corresponding system power command.
    ///
    /// Upstream delegated this to `babydra_core`; the topbar invokes the
    /// equivalent `systemctl` / `loginctl` commands directly.
    pub fn execute(self) {
        let (program, args): (&str, Vec<&str>) = match self {
            Self::Shutdown => ("systemctl", vec!["poweroff"]),
            Self::Reboot => ("systemctl", vec!["reboot"]),
            Self::Sleep => ("systemctl", vec!["suspend"]),
            Self::Logout => {
                let session = std::env::var("XDG_SESSION_ID").unwrap_or_default();
                if session.is_empty() {
                    ("pkill", vec!["-f", "babydrade"])
                } else {
                    // Owned String must live until the command is spawned.
                    return execute_logout(&session);
                }
            }
        };
        let _ = std::process::Command::new(program).args(args).spawn();
    }
}

fn execute_logout(session: &str) {
    let _ = std::process::Command::new("loginctl")
        .args(["terminate-session", session])
        .spawn();
}
