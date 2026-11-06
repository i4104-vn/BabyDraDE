use domain::{AppRegistry, DesktopEntry};

/// Service for discovering and launching applications in BabyDraDE.
#[derive(Debug, Default)]
pub struct AppLauncherService;

impl AppLauncherService {
    pub fn new() -> Self {
        Self
    }

    pub fn search(&self, registry: &dyn AppRegistry, query: &str) -> Vec<DesktopEntry> {
        registry.search(query)
    }

    pub fn find_by_id(&self, registry: &dyn AppRegistry, id: &str) -> Option<DesktopEntry> {
        registry.find_by_id(id)
    }

    /// Prepares process environment for spawned clients (setting WAYLAND_DISPLAY).
    pub fn build_spawn_command(&self, exec: &str, wayland_socket: &str) -> std::process::Command {
        let clean_exec = exec
            .split_whitespace()
            .take_while(|arg| !arg.starts_with('%'))
            .collect::<Vec<_>>()
            .join(" ");

        let mut cmd = if cfg!(windows) {
            let mut c = std::process::Command::new("cmd");
            c.args(["/C", &clean_exec]);
            c
        } else {
            let mut c = std::process::Command::new("sh");
            c.args(["-c", &clean_exec]);
            c
        };

        cmd.env("WAYLAND_DISPLAY", wayland_socket);
        cmd
    }
}
