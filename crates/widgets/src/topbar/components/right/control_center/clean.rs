//! Clean My Linux Quick Tile & Popover UI for Control Center

use gtk4::prelude::*;
use infrastructure::i18n::trans;
use std::rc::Rc;
use ui_kit::components::{create_color_btn, create_popover};
use ui_kit::icons::get_icon_colored;

/// Sets up the Clean My Linux popover layout
fn setup_clean_popover(popover: &gtk4::Popover) {
    let popover_box = gtk4::Box::new(gtk4::Orientation::Vertical, 8);
    popover_box.add_css_class("media-popover-box");
    popover_box.set_size_request(260, -1);
    popover_box.set_margin_start(4);
    popover_box.set_margin_end(4);

    // Header
    let header = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
    header.add_css_class("media-popover-header");
    header.set_valign(gtk4::Align::Center);

    let icon = get_icon_colored("broom", 14, "#ef4444");
    let title = gtk4::Label::new(Some(&trans("control.clean_my_linux")));
    title.add_css_class("media-popover-app-name");

    header.append(&icon);
    header.append(&title);
    popover_box.append(&header);

    // Center Status
    let center_box = gtk4::Box::new(gtk4::Orientation::Vertical, 2);
    center_box.set_valign(gtk4::Align::Center);
    center_box.set_halign(gtk4::Align::Center);
    center_box.set_margin_top(12);
    center_box.set_margin_bottom(12);

    let status_label = gtk4::Label::new(Some(&trans("common.ready_upper")));
    status_label.add_css_class("clean-popover-percent");

    let size_label = gtk4::Label::new(Some("0 B"));
    size_label.add_css_class("clean-popover-size");

    center_box.append(&status_label);
    center_box.append(&size_label);
    popover_box.append(&center_box);

    // Targets List
    let target_list = gtk4::Box::new(gtk4::Orientation::Vertical, 6);
    target_list.add_css_class("clean-target-list");

    let targets = [
        ("System Cache", "0 B"),
        ("Journal Logs", "0 B"),
        ("Trash Bin", "0 B"),
    ];

    for (name, size) in targets {
        let row = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
        row.add_css_class("clean-target-row");
        row.set_hexpand(true);

        let bullet = gtk4::Label::new(Some("•"));
        bullet.add_css_class("clean-target-bullet");

        let name_lbl = gtk4::Label::new(Some(name));
        name_lbl.add_css_class("clean-target-name");
        name_lbl.set_hexpand(true);
        name_lbl.set_halign(gtk4::Align::Start);

        let size_lbl = gtk4::Label::new(Some(size));
        size_lbl.add_css_class("clean-target-size");
        size_lbl.set_halign(gtk4::Align::End);

        row.append(&bullet);
        row.append(&name_lbl);
        row.append(&size_lbl);
        target_list.append(&row);
    }
    popover_box.append(&target_list);

    // Clean Action Button
    let clean_btn = gtk4::Button::with_label(&trans("panel.clear_all"));
    clean_btn.add_css_class("btn-primary");
    clean_btn.add_css_class("clean-popover-btn");
    clean_btn.set_margin_top(6);

    let popover_c = popover.clone();
    clean_btn.connect_clicked(move |_| {
        popover_c.popdown();
    });

    popover_box.append(&clean_btn);
    popover.set_child(Some(&popover_box));
}

/// Creates a new Clean tile button.
pub fn create_clean_tile(on_popover_toggled: Option<Rc<dyn Fn(bool) + 'static>>) -> gtk4::Button {
    let btn = create_color_btn(
        "broom",
        18,
        "rgba(255, 255, 255, 0.8)",
        &["control-square-tile"],
        None,
        || {},
    );
    btn.set_size_request(56, 56);
    btn.set_halign(gtk4::Align::Center);
    btn.set_valign(gtk4::Align::Center);

    let popover = create_popover(
        &btn,
        gtk4::PositionType::Bottom,
        "clean-popover media-popover control-popover",
    );

    setup_clean_popover(&popover);

    let popover_c = popover.clone();
    let on_popover_toggled_c = on_popover_toggled.clone();
    btn.connect_clicked(move |_| {
        popover_c.popup();
        if let Some(ref cb) = on_popover_toggled_c {
            cb(true);
        }
    });

    let on_popover_toggled_c2 = on_popover_toggled;
    popover.connect_closed(move |_| {
        if let Some(ref cb) = on_popover_toggled_c2 {
            cb(false);
        }
    });

    btn
}

