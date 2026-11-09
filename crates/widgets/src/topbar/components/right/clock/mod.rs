mod render;

use gtk4::glib;

pub fn create_clock_widget() -> gtk4::Button {
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

    clock_button
}
