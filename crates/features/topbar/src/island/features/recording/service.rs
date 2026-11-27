//! Recording service: data adapter, action dispatchers and state poller.
//!
//! IPC triggers are registered centrally through `IslandCtx::on_trigger`.

pub mod actions;
pub mod polling;
pub mod recording;

pub use actions::{is_audio_muted, is_mic_muted, toggle_audio_mute, toggle_mic_mute};
pub use polling::{spawn_recording_polling, IslandRecordingState};
pub use recording::{
    get_elapsed_secs, get_recordings_dir, is_paused, is_recording, select_geometry_str_with_slurp,
    start_recording, stop_recording, toggle_pause,
};

/// Connected display names for the recording settings dropdown (`xrandr`).
pub fn display_output_names() -> Vec<String> {
    let output = std::process::Command::new("xrandr")
        .arg("--query")
        .output()
        .ok();
    let names: Vec<String> = output
        .into_iter()
        .flat_map(|output| {
            String::from_utf8_lossy(&output.stdout)
                .lines()
                .map(String::from)
                .collect::<Vec<_>>()
        })
        .filter(|line| line.contains(" connected"))
        .filter_map(|line| line.split_whitespace().next().map(str::to_string))
        .collect();
    if names.is_empty() {
        vec!["Display 1".to_string()]
    } else {
        names
    }
}
