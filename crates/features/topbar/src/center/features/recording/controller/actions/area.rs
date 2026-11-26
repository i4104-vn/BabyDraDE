//! Area selection and folder navigation action handlers.

use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use gtk4::Label;

use crate::center::features::recording::models::{RecordingArea, RecordingConfig, RecordingMode};
use crate::center::features::recording::service;
use crate::center::ui::IslandPopover;

/// Opens the recordings directory in the system file manager.
pub fn open_recordings_dir_action() {
    let _ = services::filedialog::open::open_target(
        &service::get_recordings_dir().to_string_lossy(),
    );
}

/// Triggers `slurp` area selection while temporarily closing the popover,
/// then reopens it and updates the configuration with the chosen geometry.
pub fn select_recording_area_action(
    popover: &IslandPopover,
    area_label: &Label,
    config: &Rc<RefCell<RecordingConfig>>,
) {
    popover.popdown();

    let (sender, receiver) = tokio::sync::oneshot::channel::<Option<String>>();
    std::thread::spawn(move || {
        // Wait for the compositor to unmap the popover before slurp snapshots.
        std::thread::sleep(Duration::from_millis(150));
        let result = service::select_geometry_str_with_slurp();
        let _ = sender.send(result);
    });

    let popover = popover.clone();
    let area_label = area_label.clone();
    let config = config.clone();

    glib::MainContext::default().spawn_local(async move {
        if let Ok(Some(geometry)) = receiver.await {
            area_label.set_text(&geometry);
            let numbers: Vec<i32> = geometry
                .split_whitespace()
                .flat_map(|part| part.split(','))
                .filter_map(|value| value.parse::<i32>().ok())
                .collect();
            if numbers.len() >= 4 {
                config.borrow_mut().mode = RecordingMode::Area(RecordingArea {
                    x: numbers[0],
                    y: numbers[1],
                    width: numbers[2],
                    height: numbers[3],
                });
            } else {
                config.borrow_mut().mode = RecordingMode::Window(geometry);
            }
        }
        popover.popup();
    });
}
