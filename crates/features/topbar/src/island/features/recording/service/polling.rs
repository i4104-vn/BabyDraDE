//! Background poller monitoring the recording service.

use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use super::recording;
pub use crate::island::features::recording::models::IslandRecordingState;

/// Spawns a background thread that polls the recording service and mirrors its
/// state into an `Rc<RefCell<_>>` on the GTK main context.
pub fn spawn_recording_polling() -> Rc<RefCell<IslandRecordingState>> {
    let state_rc = Rc::new(RefCell::new(IslandRecordingState::default()));
    let (sender, mut receiver) = tokio::sync::mpsc::unbounded_channel::<IslandRecordingState>();

    std::thread::Builder::new()
        .name("island-recorder-poll".into())
        .spawn(move || loop {
            let mut state = IslandRecordingState::default();

            if recording::is_recording() {
                state.is_recording = true;
                state.is_paused = recording::is_paused();
                state.elapsed_secs = recording::get_elapsed_secs();
                state.is_audio_muted = recording::is_recording_audio_muted();
                state.is_mic_muted = recording::is_recording_mic_muted();
            }

            if sender.send(state).is_err() {
                break;
            }
            std::thread::sleep(Duration::from_millis(500));
        })
        .ok();

    let cache = state_rc.clone();
    glib::MainContext::default().spawn_local(async move {
        while let Some(state) = receiver.recv().await {
            *cache.borrow_mut() = state;
        }
    });

    state_rc
}
