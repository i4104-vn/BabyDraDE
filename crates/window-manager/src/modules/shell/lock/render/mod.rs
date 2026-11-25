//! Lock screen UI components, window creation, and blur transitions.


pub(crate) use transition::{reverse_lock_transition, LockTransition};
pub(crate) use wallpaper::{build_avatar_widget, build_wallpaper_img, capture_transition_images};
pub(crate) use widgets::{build_clock_labels, build_primary_card, PrimaryCard};

// Keep the helper re-exports live without lint suppression: their call sites
// live outside this module, so a plain `use` would look unused here.
const _: fn(i32) -> gtk4::Widget = build_avatar_widget;
const _: Option<PrimaryCard> = None;

use auth_ui::setup_auth_handler;
use gtk4::prelude::*;
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};
use std::cell::{Cell, RefCell};
use std::path::Path;
use std::rc::Rc;
use transition::schedule_blur_transition;
use widgets::{setup_clock_timer, setup_focus_management};

use ui_kit::layer_shell::{init_layer_window, LayerWindowConfig};

/// Spawns a lock window assigned to a specific monitor.
pub(crate) fn create_lock_window(
    app: &gtk4::Application,
    monitor: Option<&gtk4::gdk::Monitor>,
    is_primary: bool,
    custom_wallpaper: Option<&str>,
    transition_image: Option<&Path>,
    transition_stacks: Rc<RefCell<Vec<LockTransition>>>,
    on_unlock: Rc<dyn Fn()>,
) {
    let window = gtk4::ApplicationWindow::new(app);

    let kbd_mode = if is_primary {
        KeyboardMode::Exclusive
    } else {
        KeyboardMode::None
    };

    init_layer_window(
        &window,
        &LayerWindowConfig {
            layer: Layer::Overlay,
            keyboard_mode: kbd_mode,
            exclusive_zone: -1,
            anchors: &[
                (Edge::Top, true),
                (Edge::Bottom, true),
                (Edge::Left, true),
                (Edge::Right, true),
            ],
            margin_bottom: 0,
            namespace: Some("babydra-lock"),
        },
    );
    window.set_opacity(1.0);

    if let Some(m) = monitor {
        window.set_monitor(Some(m));
    }

    window.add_css_class("lock-window");
    window.connect_close_request(|_| glib::Propagation::Stop);

    let overlay = gtk4::Overlay::new();

    let bg_picture = build_wallpaper_img(custom_wallpaper);
    overlay.set_child(Some(&bg_picture));

    let tint_box = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    tint_box.add_css_class("lock-tint");
    tint_box.set_hexpand(true);
    tint_box.set_vexpand(true);
    overlay.add_overlay(&tint_box);

    let center_box = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    center_box.set_valign(gtk4::Align::Center);
    center_box.set_halign(gtk4::Align::Center);
    center_box.set_hexpand(true);
    center_box.set_vexpand(true);

    if is_primary {
        let (clock_label, date_label) = build_clock_labels();
        setup_clock_timer(&clock_label, &date_label);

        let card_box = build_primary_card(&clock_label, &date_label);
        setup_auth_handler(
            on_unlock,
            &card_box.entry,
            &card_box.status_label,
            &card_box.card_box,
        );
        setup_focus_management(&window, &card_box.entry);
        center_box.append(&card_box.card_box);
    } else {
        let (clock_label, date_label) = build_clock_labels();
        setup_clock_timer(&clock_label, &date_label);
        center_box.append(&clock_label);
        center_box.append(&date_label);
    }

    overlay.add_overlay(&center_box);

    let transition = transition_image.map(|path| {
        let stack = gtk4::Stack::new();
        stack.set_transition_type(gtk4::StackTransitionType::Crossfade);
        stack.set_transition_duration(60);
        stack.set_hexpand(true);
        stack.set_vexpand(true);
        stack.add_css_class("lock-transition-image");

        let picture = gtk4::Picture::for_filename(path);
        picture.set_can_shrink(true);
        picture.set_hexpand(true);
        picture.set_vexpand(true);
        stack.add_named(&picture, Some("blur-0"));
        overlay.add_overlay(&stack);
        let transition = LockTransition {
            stack,
            window: window.clone(),
            cancelled: Rc::new(Cell::new(false)),
            last_blur_frame: Rc::new(Cell::new(0)),
        };
        (transition, path.to_path_buf())
    });
    if let Some((transition, _)) = transition.as_ref() {
        transition_stacks.borrow_mut().push(transition.clone());
    }

    window.set_child(Some(&overlay));
    window.present();

    if let Some((transition, image_path)) = transition {
        schedule_blur_transition(transition, image_path);
    }
}

mod auth_ui;

mod transition;

mod wallpaper;

mod widgets;
