//! Bridges locale changes from core headless watcher to the GTK main loop.

use gtk4::glib::{self, ControlFlow};
use std::time::Duration;

pub fn watch_locale_change<F: Fn(&str) + 'static>(callback: F) {
    let receiver = services::i18n::watch_locale_changes();
    glib::timeout_add_local(Duration::from_millis(300), move || loop {
        match receiver.try_recv() {
            Ok(locale) => callback(&locale),
            Err(std::sync::mpsc::TryRecvError::Empty) => return ControlFlow::Continue,
            Err(std::sync::mpsc::TryRecvError::Disconnected) => return ControlFlow::Break,
        }
    });
}
