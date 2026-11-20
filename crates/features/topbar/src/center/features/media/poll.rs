//! Background `playerctl` poller feeding the island with MPRIS metadata.

use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

const METADATA_FORMAT: &str = "{{ status }}|//|{{ title }}|//|{{ artist }}|//|{{ playerName }}|//|{{ mpris:artUrl }}|//|{{ position }}|//|{{ mpris:length }}";

/// Builds a `playerctl` command with a D-Bus fallback for sessions launched
/// from a TTY (where `DBUS_SESSION_BUS_ADDRESS` may be missing).
fn playerctl_command() -> std::process::Command {
    let mut command = std::process::Command::new("playerctl");
    if std::env::var_os("DBUS_SESSION_BUS_ADDRESS").is_none() {
        let uid = unsafe { libc::getuid() };
        command.env(
            "DBUS_SESSION_BUS_ADDRESS",
            format!("unix:path=/run/user/{uid}/bus"),
        );
    }
    command
}

/// Spawns the polling thread and returns the shared main-thread cache.
pub fn spawn_playerctl_polling() -> Rc<RefCell<Option<String>>> {
    let cache = Rc::new(RefCell::new(None));
    let (sender, mut receiver) = tokio::sync::mpsc::unbounded_channel::<Option<String>>();

    std::thread::Builder::new()
        .name("island-playerctl-poll".into())
        .spawn(move || loop {
            let metadata = playerctl_command()
                .args(["metadata", "--format", METADATA_FORMAT])
                .output()
                .ok()
                .filter(|out| out.status.success())
                .and_then(|out| {
                    let line = String::from_utf8_lossy(&out.stdout).trim().to_string();
                    (!line.is_empty()).then_some(line)
                });
            if sender.send(metadata).is_err() {
                break;
            }
            std::thread::sleep(Duration::from_millis(1000));
        })
        .ok();

    let main_cache = cache.clone();
    glib::MainContext::default().spawn_local(async move {
        while let Some(metadata) = receiver.recv().await {
            *main_cache.borrow_mut() = metadata;
        }
    });

    cache
}

/// Runs a one-off `playerctl` command (used by the popover controls).
pub fn playerctl(command: &str) {
    let _ = playerctl_command().arg(command).spawn();
}
