//! Greeter top bar: clock/date labels and power action buttons.

use gtk4::prelude::*;
use gtk4::{Align, Box as GtkBox, Button, Label, Orientation};

pub(crate) struct TopBarWidget {
    pub(crate) container: GtkBox,
    pub(crate) power_btn: Button,
    pub(crate) reboot_btn: Button,
    pub(crate) suspend_btn: Button,
    pub(crate) clock_label: Label,
    pub(crate) date_label: Label,
}

fn power_button(icon_name: &str, extra_class: Option<&str>, tooltip_key: &str) -> Button {
    let button = Button::new();
    button.add_css_class("power-btn");
    if let Some(class) = extra_class {
        button.add_css_class(class);
    }
    button.set_tooltip_text(Some(&services::trans(tooltip_key)));
    button.set_cursor_from_name(Some("pointer"));
    let icon = ui_kit::icons::get_icon_colored(icon_name, 18, "#ffffff");
    button.set_child(Some(&icon));
    button
}

pub(crate) fn build() -> TopBarWidget {
    let container = GtkBox::new(Orientation::Horizontal, 0);
    container.add_css_class("top-bar");
    container.set_valign(Align::Start);

    let time_box = GtkBox::new(Orientation::Vertical, 0);
    time_box.set_halign(Align::Start);

    let clock_label = Label::new(None);
    clock_label.add_css_class("clock-time");
    let date_label = Label::new(None);
    date_label.add_css_class("clock-date");
    time_box.append(&clock_label);
    time_box.append(&date_label);

    let actions = GtkBox::new(Orientation::Horizontal, 10);
    actions.set_halign(Align::End);
    actions.set_hexpand(true);
    actions.set_valign(Align::Center);

    let suspend_btn = power_button("sleep", Some("action-btn-suspend"), "greeter.suspend");
    let reboot_btn = power_button("restart", Some("action-btn-reboot"), "greeter.reboot");
    let power_btn = power_button("power", None, "greeter.power_off");
    actions.append(&suspend_btn);
    actions.append(&reboot_btn);
    actions.append(&power_btn);

    container.append(&time_box);
    container.append(&actions);

    TopBarWidget {
        container,
        power_btn,
        reboot_btn,
        suspend_btn,
        clock_label,
        date_label,
    }
}
