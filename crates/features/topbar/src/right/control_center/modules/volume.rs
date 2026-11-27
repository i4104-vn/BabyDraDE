//! Volume Slider UI item for Control Center

use gtk4::prelude::*;
use services::i18n::trans;
use services::system::volume::{get_active_output_name, get_volume_state, set_muted, set_volume};
use std::cell::Cell;
use std::rc::Rc;
use ui_kit::components::overlay::{
    fit_popover_with_policy, max_popover_height, max_popover_width, CrossAlign, FitPolicy,
    ANCHOR_GAP, EDGE_MARGIN,
};
use ui_kit::components::{create_popover, create_slider_header, PillSlider};
use ui_kit::icons::{get_icon, get_icon_colored, set_image_from_icon};

pub fn update_topbar_volume_state(vol_icon: &gtk4::Image, vol: f64, muted: bool) {
    let name = match true {
        _ if muted => "volume-mute",
        _ if vol > 66.0 => "volume",
        _ if vol > 33.0 => "volume-medium",
        _ => "volume-low",
    };
    set_image_from_icon(vol_icon, name, 14);
}

/// Creates a new volume control row.
pub fn create_volume_row(
    on_popover_toggled: Option<Rc<dyn Fn(bool) + 'static>>,
    vol_icon: gtk4::Image,
) -> (gtk4::Box, Rc<dyn Fn(f64, bool)>) {
    let main_box = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
    main_box.add_css_class("control-slider-card");

    let (initial_val, initial_muted) = get_volume_state();
    let current_val = Rc::new(Cell::new(initial_val));
    let (header_box, value_label) = create_slider_header(&trans("volume.title"), initial_val);

    let row_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);

    let icon_container = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
    icon_container.set_valign(gtk4::Align::Center);

    let muted_state = Rc::new(Cell::new(initial_muted));
    let update_mute_icon = {
        let icon_container = icon_container.clone();
        Rc::new(move |is_muted_val: bool| {
            if let Some(old) = icon_container.first_child() {
                icon_container.remove(&old);
            }
            let icon_widget = if is_muted_val {
                get_icon_colored("volume-mute", 16, "#ffffff")
            } else {
                get_icon_colored("volume", 16, "#ffffff")
            };
            icon_widget.add_css_class("slider-icon");
            icon_container.append(&icon_widget);
        })
    };

    let mute_btn = gtk4::Button::new();
    mute_btn.add_css_class("slider-overlay-mute-btn");
    mute_btn.set_child(Some(&icon_container));
    mute_btn.set_halign(gtk4::Align::Start);
    mute_btn.set_valign(gtk4::Align::Center);
    mute_btn.set_margin_start(10);
    mute_btn.set_can_focus(false);
    mute_btn.set_focus_on_click(false);

    {
        let update_mute_icon_clone = update_mute_icon.clone();
        let muted_state_clone = muted_state.clone();
        let vol_icon_c = vol_icon.clone();
        let current_val_c = current_val.clone();
        mute_btn.connect_clicked(move |_| {
            let new_mute = !muted_state_clone.get();
            muted_state_clone.set(new_mute);
            set_muted(new_mute);
            update_mute_icon_clone(new_mute);
            update_topbar_volume_state(&vol_icon_c, current_val_c.get(), new_mute);
        });
    }
    update_mute_icon(muted_state.get());

    let slider = PillSlider::new(initial_val, |_| {});
    slider.bind_label(&value_label);
    slider.add_scroll_to(&row_box);

    let vol_icon_c2 = vol_icon.clone();
    let update_mute_icon_c = update_mute_icon.clone();
    let muted_state_c = muted_state.clone();
    let current_val_c2 = current_val.clone();
    slider.connect_debounced(80, move |val| {
        current_val_c2.set(val);
        set_volume(val);
        if val > 0.0 && muted_state_c.get() {
            // set_volume() does not unmute the sink, so a muted slider would
            // otherwise look like it does nothing.
            set_muted(false);
            muted_state_c.set(false);
            update_mute_icon_c(false);
        }
        update_topbar_volume_state(&vol_icon_c2, val, muted_state_c.get());
    });

    let menu_btn = gtk4::Button::new();
    menu_btn.add_css_class("slider-popover-btn");
    menu_btn.set_valign(gtk4::Align::Center);
    let menu_icon = get_icon("go-up-symbolic", 12);
    menu_btn.set_child(Some(&menu_icon));

    // Audio devices popover. The button sits at the right edge of the panel,
    // so start with a right-side preference; the fitter flips it to the left
    // when there is not enough room. `End` alignment makes it grow upwards
    // from the button, leaving the sliders below uncovered. The fitter's
    // pointing-rect math keeps the popup beside the button (never on top of
    // its icon) and clamps it to the monitor on either side.
    let popover = create_popover(&menu_btn, gtk4::PositionType::Right, "audio-menu-popover");
    let side_padding = 8;
    let max_width = max_popover_width(menu_btn.upcast_ref::<gtk4::Widget>(), EDGE_MARGIN)
        .unwrap_or(248);
    let content_width = max_width.saturating_sub(side_padding).min(240).max(1);
    let dev_box = gtk4::Box::new(gtk4::Orientation::Vertical, 6);
    dev_box.set_size_request(content_width, -1);
    dev_box.set_margin_start(4);
    dev_box.set_margin_end(4);

    let title = gtk4::Label::new(Some(&trans("volume.output_devices")));
    title.add_css_class("audio-menu-section-title");
    title.set_xalign(0.0);
    dev_box.append(&title);

    let dev_name = get_active_output_name().unwrap_or_else(|| "Default Audio Sink".to_string());
    let dev_item = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
    let dev_icon = get_icon_colored("speaker", 14, "#ffffff");
    let dev_lbl = gtk4::Label::new(Some(&dev_name));
    dev_lbl.set_ellipsize(gtk4::pango::EllipsizeMode::End);
    dev_lbl.set_halign(gtk4::Align::Start);
    dev_lbl.set_hexpand(true);
    dev_lbl.set_width_request(1);
    let check = gtk4::Label::new(Some("✓"));
    check.add_css_class("audio-menu-item-check");

    dev_item.append(&dev_icon);
    dev_item.append(&dev_lbl);
    dev_item.append(&check);

    let dev_row = gtk4::Button::new();
    dev_row.add_css_class("audio-menu-item-btn");
    dev_row.add_css_class("active");
    dev_row.set_child(Some(&dev_item));
    dev_box.append(&dev_row);

    let scrolled = gtk4::ScrolledWindow::new();
    scrolled.set_hscrollbar_policy(gtk4::PolicyType::Never);
    scrolled.set_vscrollbar_policy(gtk4::PolicyType::Automatic);
    scrolled.set_propagate_natural_width(true);
    scrolled.set_propagate_natural_height(true);
    scrolled.set_max_content_width(max_width);
    if let Some(max_height) =
        max_popover_height(menu_btn.upcast_ref::<gtk4::Widget>(), EDGE_MARGIN)
    {
        scrolled.set_max_content_height(max_height);
    }
    scrolled.set_child(Some(&dev_box));
    popover.set_child(Some(&scrolled));

    let popover_c = popover.clone();
    let on_popover_toggled_c = on_popover_toggled.clone();
    let menu_btn_c = menu_btn.clone();
    let dev_box_c = dev_box.clone();
    let scrolled_c = scrolled.clone();
    menu_btn.connect_clicked(move |_| {
        // At creation time the panel window may not be realized yet; refresh
        // the width cap from the actual monitor right before presenting.
        if let Some(width) =
            max_popover_width(menu_btn_c.upcast_ref::<gtk4::Widget>(), EDGE_MARGIN)
        {
            scrolled_c.set_max_content_width(width.min(max_width));
            dev_box_c.set_size_request(width.saturating_sub(side_padding).min(240).max(1), -1);
        }
        fit_popover_with_policy(
            &popover_c,
            gtk4::PositionType::Right,
            CrossAlign::End,
            EDGE_MARGIN,
            ANCHOR_GAP,
            FitPolicy::Flip,
        );
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

    let overlay = gtk4::Overlay::new();
    overlay.set_hexpand(true);
    overlay.set_valign(gtk4::Align::Center);
    overlay.set_child(Some(&slider.container));
    overlay.add_overlay(&mute_btn);

    row_box.append(&overlay);
    row_box.append(&menu_btn);

    main_box.append(&header_box);
    main_box.append(&row_box);

    let slider_sync = slider.clone();
    let update_mute_icon_sync = update_mute_icon;
    let muted_state_sync = muted_state;
    let current_val_sync = current_val;
    let value_label_sync = value_label.clone();
    let vol_icon_sync = vol_icon.clone();
    let sync_callback: Rc<dyn Fn(f64, bool)> = Rc::new(move |val: f64, is_muted_val: bool| {
        current_val_sync.set(val);
        muted_state_sync.set(is_muted_val);
        slider_sync.set_value_silent(val);
        value_label_sync.set_text(&format!("{:.0}%", val));
        update_mute_icon_sync(is_muted_val);
        update_topbar_volume_state(&vol_icon_sync, val, is_muted_val);
    });

    (main_box, sync_callback)
}
