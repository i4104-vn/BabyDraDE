//! Background `playerctl` polling and time formatting for the media feature.

use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

const METADATA_FORMAT: &str = "{{ status }}|//|{{ title }}|//|{{ artist }}|//|{{ playerName }}|//|{{ mpris:artUrl }}|//|{{ position }}|//|{{ mpris:length }}";

/// Formats a duration in seconds as `H:MM:SS` or `M:SS`.
pub fn format_time(secs: f64) -> String {
    if secs <= 0.0 || secs.is_nan() || secs.is_infinite() {
        return "0:00".to_string();
    }
    let total = secs as u64;
    let hours = total / 3600;
    let minutes = (total % 3600) / 60;
    let seconds = total % 60;
    if hours > 0 {
        format!("{}:{:02}:{:02}", hours, minutes, seconds)
    } else {
        format!("{}:{:02}", minutes, seconds)
    }
}

/// Spawns the polling thread and returns the shared main-thread cache.
pub fn spawn_playerctl_polling() -> Rc<RefCell<Option<String>>> {
    let cache = Rc::new(RefCell::new(None));
    // Bounded: only one pending sample, so a stalled main loop cannot accumulate
    // one `playerctl` result per second indefinitely.
    let (sender, mut receiver) = tokio::sync::mpsc::channel::<Option<String>>(1);

    std::thread::Builder::new()
        .name("island-playerctl-poll".into())
        .spawn(move || loop {
            let metadata = std::process::Command::new("playerctl")
                .args(["metadata", "--format", METADATA_FORMAT])
                .output()
                .ok()
                .filter(|out| out.status.success())
                .and_then(|out| {
                    let line = String::from_utf8_lossy(&out.stdout).trim().to_string();
                    (!line.is_empty()).then_some(line)
                });
            if sender.try_send(metadata).is_err() && sender.is_closed() {
                break;
            }
            std::thread::sleep(Duration::from_millis(1000));
        })
        .ok();

    let main_cache = cache.clone();
    glib::MainContext::default().spawn_local(async move {
        while let Some(metadata) = receiver.recv().await {
            let mut latest = metadata;
            while let Ok(newer) = receiver.try_recv() {
                latest = newer;
            }
            *main_cache.borrow_mut() = latest;
        }
    });

    cache
}

/// Runs a one-off `playerctl` command (used by the popover controls).
pub fn playerctl(command: &str) {
    let _ = std::process::Command::new("playerctl").arg(command).spawn();
}
