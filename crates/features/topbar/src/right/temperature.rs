//! Tray indicator that always shows the CPU temperature and lists every
//! temperature sensor the system exposes in a shared ui-kit tooltip popover.

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

pub fn create_temperature_widget() -> gtk4::Box {
    let container = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
    container.add_css_class("sys-monitor-box");
    container.set_valign(gtk4::Align::Center);

    let label = gtk4::Label::new(Some("--°C"));
    label.add_css_class("status-text");
    container.append(&label);

    let tooltip = Rc::new(TooltipPopover::attach_rows(
        &container,
        "Temperatures",
        &temperature_rows(),
    ));

    let refresh = {
        let label = label.clone();
        let tooltip = tooltip.clone();
        move || {
            label.set_text(
                &get_cpu_temperature()
                    .map(|temp| format!("{temp:.0}°C"))
                    .unwrap_or_else(|| "--°C".to_string()),
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

    container
}
