pub mod calendar;
mod render;

use crate::control::control_center::{close_all_panels, WindowSlot};
use gtk4::glib;
use gtk4::prelude::*;

pub fn create_clock_widget(app: &gtk4::Application, cal_slot: WindowSlot) -> gtk4::Button {
    let (clock_button, clock_label, _red_dot, _bell_box) = render::build_clock_ui();

    let update_clock = {
        let clock_label = clock_label.clone();
        move || {
            let time_str = glib::DateTime::now_local()
                .and_then(|dt| dt.format("%d/%m  %I:%M %p"))
                .map(|s| s.to_string())
                .unwrap_or_else(|_| "00:00".to_string());
            clock_label.set_text(&time_str);

            glib::ControlFlow::Continue
        }
    };

    update_clock();
    glib::timeout_add_seconds_local(1, update_clock);

    let app_c = app.clone();
    let cal_inner = cal_slot;
    clock_button.connect_clicked(move |_| {
        let was_open = cal_inner.borrow().is_some();
        // Closing all panels first guarantees they never stack on top of each other.
        close_all_panels();
        if was_open {
            return;
        }
        let new_win = calendar::show_calendar_window(&app_c, cal_inner.clone());
        if let Ok(mut borrow) = cal_inner.try_borrow_mut() {
            *borrow = Some(new_win);
        }
    });

    clock_button
}
