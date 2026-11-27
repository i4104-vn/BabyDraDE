//! Dynamic Island capsule layout for recording indicator.

use gtk4::prelude::*;
use gtk4::{Align, Box as GtkBox, CenterBox, DrawingArea, Label, Orientation};
use ui_kit::icons::get_icon;

use crate::island::features::recording::service;

#[derive(Clone)]
pub struct RecordingCapsuleWidget {
    pub container: GtkBox,
    pub timer_label: Label,
    pub indicator: DrawingArea,
}

impl RecordingCapsuleWidget {
    pub fn new() -> Self {
        let container = GtkBox::new(Orientation::Horizontal, 0);
        container.add_css_class("notch-content");
        container.set_valign(Align::Center);
        container.set_halign(Align::Fill);
        container.set_hexpand(true);
        container.set_vexpand(true);
        container.set_focusable(false);
        container.set_can_focus(false);

        let center_box = CenterBox::new();
        center_box.set_hexpand(true);
        center_box.set_valign(Align::Center);

        let icon_container = GtkBox::new(Orientation::Horizontal, 0);
        icon_container.set_valign(Align::Center);
        icon_container.set_halign(Align::Start);
        icon_container.set_margin_start(6);
        let icon = get_icon("camera", 14);
        icon.add_css_class("recording-capsule-icon");
        icon.set_valign(Align::Center);
        icon.set_halign(Align::Start);
        icon_container.append(&icon);
        center_box.set_start_widget(Some(&icon_container));

        let timer_label = Label::new(Some("00:00"));
        timer_label.add_css_class("recording-capsule-timer");
        timer_label.set_valign(Align::Center);
        timer_label.set_halign(Align::Center);
        center_box.set_center_widget(Some(&timer_label));

        let indicator = DrawingArea::new();
        indicator.set_content_width(18);
        indicator.set_content_height(18);
        indicator.set_valign(Align::Center);
        indicator.set_halign(Align::End);
        indicator.set_margin_end(6);

        indicator.set_draw_func(|_, context, width, height| {
            let center_x = width as f64 / 2.0;
            let center_y = height as f64 / 2.0;
            let pi = std::f64::consts::PI;

            context.set_source_rgba(0.937, 0.267, 0.267, 0.90);
            context.set_line_width(1.8);
            context.arc(center_x, center_y, 6.8, 0.0, 2.0 * pi);
            let _ = context.stroke();

            context.set_source_rgba(0.937, 0.267, 0.267, 1.0);
            let size = 6.0;
            let x = center_x - size / 2.0;
            let y = center_y - size / 2.0;
            let radius = 1.2;
            let half_pi = std::f64::consts::FRAC_PI_2;
            context.new_sub_path();
            context.arc(x + size - radius, y + radius, radius, -half_pi, 0.0);
            context.arc(x + size - radius, y + size - radius, radius, 0.0, half_pi);
            context.arc(x + radius, y + size - radius, radius, half_pi, pi);
            context.arc(x + radius, y + radius, radius, pi, 3.0 * half_pi);
            context.close_path();
            let _ = context.fill();
        });

        let stop_click = gtk4::GestureClick::new();
        stop_click.connect_pressed(|gesture, _, _, _| {
            if service::is_recording() {
                std::thread::spawn(|| {
                    let _ = service::stop_recording();
                });
                gesture.set_state(gtk4::EventSequenceState::Claimed);
            }
        });
        indicator.add_controller(stop_click);
        indicator.set_cursor_from_name(Some("pointer"));

        center_box.set_end_widget(Some(&indicator));
        container.append(&center_box);

        Self {
            container,
            timer_label,
            indicator,
        }
    }

    pub fn update_timer(&self, text: &str) {
        self.timer_label.set_text(text);
    }
}

impl Default for RecordingCapsuleWidget {
    fn default() -> Self {
        Self::new()
    }
}
