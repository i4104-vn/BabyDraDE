//! CPU temperature as a regular tray item on the right, with the shared
//! ui-kit tooltip listing the remaining important temperatures.

use gtk4::glib;
use gtk4::prelude::*;
use services::system::{get_cpu_temperature, get_important_temperatures};
use std::rc::Rc;
use ui_kit::overlay::popover::{TooltipPopover, TooltipRow};

fn temperature_rows() -> Vec<TooltipRow> {
    let readings = get_important_temperatures();
    if readings.is_empty() {
        return vec![TooltipRow::new("No temperature sensors", "", None)];
    }
    readings
        .iter()
        .map(|reading| TooltipRow::new(&reading.label, &format!("{:.1} °C", reading.celsius), None))
        .collect()
}

pub(crate) fn create_temperature_widget() -> gtk4::Button {
    let button = gtk4::Button::new();
    button.add_css_class("panel-tray-item-btn");
    button.set_cursor_from_name(Some("default"));

    let label = gtk4::Label::new(Some("--"));
    label.add_css_class("status-text");
    button.set_child(Some(&label));

    let tooltip = Rc::new(TooltipPopover::attach_rows(
        &button,
        "Temperatures",
        &temperature_rows(),
    ));

    let refresh = {
        let label = label.clone();
        let tooltip = tooltip.clone();
        move || {
            label.set_text(
                &get_cpu_temperature()
                    .map(|temp| format!("{temp:.0}"))
                    .unwrap_or_else(|| "--".to_string()),
            );

            if tooltip.popover.is_visible() {
                let card = TooltipPopover::build_card("Temperatures", &temperature_rows());
                tooltip.popover.set_child(Some(&card));
            }
        }
    };
    refresh();
    glib::timeout_add_seconds_local(2, move || {
        refresh();
        glib::ControlFlow::Continue
    });

    button
}
