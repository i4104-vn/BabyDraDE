//! Control Center Modal Window UI and Lifecycle

use gtk4::prelude::*;
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};
use std::cell::RefCell;
use std::rc::Rc;

use super::backlight;
use super::header::create_header_row;
use super::storage;
use super::toggle_grid::create_cc_grid;
use super::volume;

pub type WindowSlot = Rc<RefCell<Option<gtk4::ApplicationWindow>>>;

thread_local! {
    static ACTIVE_CC_SYNC: RefCell<Option<ActiveCcSync>> = const { RefCell::new(None) };
}

struct ActiveCcSync {
    update_volume: Rc<dyn Fn(f64, bool)>,
    update_brightness: Rc<dyn Fn(f64)>,
}

pub fn sync_cc_volume(vol: f64, muted: bool) {
    ACTIVE_CC_SYNC.with(|s| {
        if let Some(ref sync) = *s.borrow() {
            (sync.update_volume)(vol, muted);
        }
    });
}

pub fn sync_cc_brightness(val: f64) {
    ACTIVE_CC_SYNC.with(|s| {
        if let Some(ref sync) = *s.borrow() {
            (sync.update_brightness)(val);
        }
    });
}

fn register_cc_sync(update_vol: Rc<dyn Fn(f64, bool)>, update_bright: Rc<dyn Fn(f64)>) {
    ACTIVE_CC_SYNC.with(|s| {
        *s.borrow_mut() = Some(ActiveCcSync {
            update_volume: update_vol,
            update_brightness: update_bright,
        });
    });
}

fn clear_cc_sync() {
    ACTIVE_CC_SYNC.with(|s| {
        *s.borrow_mut() = None;
    });
}

/// Sets up dismissal when clicking anywhere outside the main panel container.
pub fn setup_click_outside_dismiss(win: &gtk4::ApplicationWindow, target: &gtk4::Box) {
    let click = gtk4::GestureClick::new();
    let win_c = win.clone();
    let target_c = target.clone();
    click.connect_pressed(move |_, _, x, y| {
        let (bx, by) = target_c
            .translate_coordinates(&win_c, 0.0, 0.0)
            .unwrap_or((0.0, 0.0));
        let w = target_c.width() as f64;
        let h = target_c.height() as f64;
        if x < bx || x > bx + w || y < by || y > by + h {
            win_c.close();
        }
    });
    win.add_controller(click);
}

/// Builds the control center transparent layer window.
fn build_control_center(
    app: &gtk4::Application,
) -> (gtk4::ApplicationWindow, gtk4::Box, gtk4::Box) {
    let q_win = gtk4::ApplicationWindow::builder()
        .application(app)
        .title("BabyDraDE Control Center")
        .decorated(false)
        .build();

    q_win.init_layer_shell();
    q_win.set_layer(Layer::Overlay);
    q_win.set_namespace(Some("babydra-control-center"));
    q_win.set_anchor(Edge::Top, true);
    q_win.set_anchor(Edge::Bottom, true);
    q_win.set_anchor(Edge::Left, true);
    q_win.set_anchor(Edge::Right, true);
    q_win.set_keyboard_mode(KeyboardMode::OnDemand);

    q_win.add_css_class("control-center-window");

    let blur_host = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    blur_host.add_css_class("control-center-blur-host");
    blur_host.set_halign(gtk4::Align::End);
    blur_host.set_valign(gtk4::Align::Start);
    blur_host.set_size_request(360, 480);
    blur_host.set_margin_top(6);
    blur_host.set_margin_end(12);

    let main_box = gtk4::Box::new(gtk4::Orientation::Vertical, 14);
    main_box.add_css_class("control-center-box");
    main_box.set_vexpand(true);
    main_box.set_hexpand(true);

    blur_host.append(&main_box);
    q_win.set_child(Some(&blur_host));

    (q_win, blur_host, main_box)
}

/// Rebuilds control center internal rows and sliders.
fn rebuild_cc_contents(
    main_box: &gtk4::Box,
    on_popover_toggled: Option<Rc<dyn Fn(bool) + 'static>>,
    vol_icon: gtk4::Image,
) {
    while let Some(child) = main_box.first_child() {
        main_box.remove(&child);
    }

    // 1. Header
    main_box.append(&create_header_row());

    // 2. Toggle Grid
    main_box.append(&create_cc_grid(on_popover_toggled.clone()));

    // 3. Volume Slider
    let (volume_row, vol_sync) = volume::create_volume_row(on_popover_toggled, vol_icon);
    main_box.append(&volume_row);

    // 4. Brightness Slider
    let (brightness_row, bright_sync) = backlight::create_brightness();
    main_box.append(&brightness_row);

    register_cc_sync(vol_sync, bright_sync);

    // 5. Storage Monitor Card
    main_box.append(&storage::create_disk_list_box());
}

/// Creates, populates, and presents the Control Center window anchored to the top-right.
pub fn create_cc_window(
    app: &gtk4::Application,
    control_center_window: WindowSlot,
    vol_icon: gtk4::Image,
) -> gtk4::ApplicationWindow {
    let (q_win, blur_host, main_box) = build_control_center(app);

    let popover_active = Rc::new(std::cell::Cell::new(false));
    let popover_active_clone = popover_active.clone();
    let q_win_weak = q_win.downgrade();

    let motion_controller = gtk4::EventControllerMotion::new();
    main_box.add_controller(motion_controller.clone());
    let motion_c = motion_controller;

    let on_popover_toggled = Rc::new(move |is_open: bool| {
        popover_active_clone.set(is_open);
        if !is_open && !motion_c.contains_pointer() {
            if let Some(win) = q_win_weak.upgrade() {
                win.close();
            }
        }
    }) as Rc<dyn Fn(bool)>;

    let on_popover_toggled_opt = Some(on_popover_toggled.clone());
    rebuild_cc_contents(&main_box, on_popover_toggled_opt, vol_icon);

    setup_click_outside_dismiss(&q_win, &blur_host);

    let popover_active_notify = popover_active;
    q_win.connect_is_active_notify(move |win| {
        if !win.is_active() && !popover_active_notify.get() {
            win.close();
        }
    });

    let ccw_inner = control_center_window;
    q_win.connect_close_request(move |_| {
        clear_cc_sync();
        if let Ok(mut borrow) = ccw_inner.try_borrow_mut() {
            *borrow = None;
        }
        glib::Propagation::Proceed
    });

    q_win.connect_destroy(|_| {
        clear_cc_sync();
    });

    q_win.present();
    q_win
}
