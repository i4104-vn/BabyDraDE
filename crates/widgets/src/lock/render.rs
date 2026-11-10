//! Lock screen UI components, window creation, and blur transitions.

use super::auth::verify_password;
use crate::common::init_layer_window;
use gtk4::prelude::*;
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};
use std::cell::{Cell, RefCell};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const BLUR_DIVISORS: [i32; 3] = [8, 16, 28];

#[derive(Clone)]
pub struct LockTransition {
    stack: gtk4::Stack,
    window: gtk4::ApplicationWindow,
    cancelled: Rc<Cell<bool>>,
    last_blur_frame: Rc<Cell<usize>>,
}

/// Spawns a lock window assigned to a specific monitor.
pub fn create_lock_window(
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
        Layer::Overlay,
        kbd_mode,
        -1,
        &[
            (Edge::Top, true),
            (Edge::Bottom, true),
            (Edge::Left, true),
            (Edge::Right, true),
        ],
        0,
        Some("babydra-lock"),
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

pub struct PrimaryCard {
    pub card_box: gtk4::Box,
    pub entry: gtk4::PasswordEntry,
    pub status_label: gtk4::Label,
}

pub fn build_wallpaper_img(custom_path: Option<&str>) -> gtk4::Picture {
    let bg_picture = gtk4::Picture::new();
    bg_picture.set_can_shrink(true);
    bg_picture.set_hexpand(true);
    bg_picture.set_vexpand(true);

    if let Some(path) = custom_path {
        bg_picture.set_filename(Some(path));
        return bg_picture;
    }

    if let Some(path) = services::wallpaper::get_wallpaper() {
        bg_picture.set_filename(Some(&path));
        return bg_picture;
    }

    bg_picture
}

pub fn build_clock_labels() -> (gtk4::Label, gtk4::Label) {
    let clock_label = gtk4::Label::new(None);
    clock_label.add_css_class("lock-clock");

    let date_label = gtk4::Label::new(None);
    date_label.add_css_class("lock-date");

    (clock_label, date_label)
}

pub fn build_avatar_widget(size: i32) -> gtk4::Widget {
    let avatar_path = services::config::get_babydra_dir().join("avatar.png");
    if avatar_path.exists() {
        let pic = gtk4::Picture::for_filename(&avatar_path);
        pic.set_size_request(size, size);
        pic.set_can_shrink(true);
        pic.add_css_class("lock-avatar");
        return pic.upcast();
    }

    let avatar_icon = ui_kit::icons::get_icon("avatar-default", size);
    avatar_icon.add_css_class("lock-avatar");
    avatar_icon.set_halign(gtk4::Align::Center);
    avatar_icon.set_valign(gtk4::Align::Center);
    avatar_icon.upcast()
}

pub fn build_primary_card(clock_label: &gtk4::Label, date_label: &gtk4::Label) -> PrimaryCard {
    let card_box = gtk4::Box::new(gtk4::Orientation::Vertical, 10);
    card_box.add_css_class("lock-card");
    card_box.set_valign(gtk4::Align::Center);
    card_box.set_halign(gtk4::Align::Center);

    let avatar_widget = build_avatar_widget(110);

    let username = std::env::var("USER").unwrap_or_else(|_| "user".to_string());
    let user_label = gtk4::Label::new(Some(&username));
    user_label.add_css_class("lock-username");

    let entry = gtk4::PasswordEntry::new();
    entry.set_placeholder_text(Some(&services::trans("lock.placeholder")));
    entry.add_css_class("lock-input");
    entry.set_halign(gtk4::Align::Center);

    let status_label = gtk4::Label::new(Some(&services::trans("lock.status")));
    status_label.add_css_class("lock-status");

    card_box.append(clock_label);
    card_box.append(date_label);
    card_box.append(&avatar_widget);
    card_box.append(&user_label);
    card_box.append(&entry);
    card_box.append(&status_label);

    PrimaryCard {
        card_box,
        entry,
        status_label,
    }
}

pub fn reverse_lock_transition(stacks: Vec<LockTransition>, on_complete: Rc<dyn Fn()>) {
    if stacks.is_empty() {
        on_complete();
        return;
    }

    let remaining = Rc::new(Cell::new(stacks.len()));
    for transition in stacks {
        let remaining = remaining.clone();
        let on_complete = on_complete.clone();
        transition.cancelled.set(true);
        transition.window.set_opacity(1.0);
        transition.stack.set_opacity(1.0);
        transition.stack.set_visible(true);
        transition
            .stack
            .set_visible_child_name(&format!("blur-{}", transition.last_blur_frame.get()));
        schedule_reverse_blur_step(
            transition,
            Rc::new(move || {
                let count = remaining.get().saturating_sub(1);
                remaining.set(count);
                if count == 0 {
                    on_complete();
                }
            }),
        );
    }
}

fn schedule_reverse_blur_step(transition: LockTransition, on_complete: Rc<dyn Fn()>) {
    let current = transition.last_blur_frame.get();
    if current == 0 {
        fade_out_reverse_transition(transition, on_complete);
        return;
    }

    glib::timeout_add_local_once(Duration::from_millis(35), move || {
        let next = current.saturating_sub(1);
        transition
            .stack
            .set_visible_child_name(&format!("blur-{next}"));
        transition.last_blur_frame.set(next);
        if next == 0 {
            fade_out_reverse_transition(transition, on_complete);
        } else {
            schedule_reverse_blur_step(transition, on_complete);
        }
    });
}

fn fade_out_reverse_transition(transition: LockTransition, on_complete: Rc<dyn Fn()>) {
    let started = Instant::now();
    glib::timeout_add_local(Duration::from_millis(16), move || {
        let progress = (started.elapsed().as_secs_f64() / 0.12).clamp(0.0, 1.0);
        transition.window.set_opacity(1.0 - progress);

        if progress >= 1.0 {
            transition.window.set_visible(false);
            on_complete();
            glib::ControlFlow::Break
        } else {
            glib::ControlFlow::Continue
        }
    });
}

fn schedule_blur_transition(transition: LockTransition, image_path: PathBuf) {
    let source = match gtk4::gdk_pixbuf::Pixbuf::from_file(&image_path) {
        Ok(pixbuf) => pixbuf,
        Err(_) => return,
    };
    schedule_blur_step(transition, source, 0);
}

fn schedule_blur_step(transition: LockTransition, source: gtk4::gdk_pixbuf::Pixbuf, step: usize) {
    let delay = if step == 0 { 10 } else { 70 };

    glib::timeout_add_local_once(Duration::from_millis(delay), move || {
        if transition.cancelled.get() {
            return;
        }

        let divisor = BLUR_DIVISORS[step];
        if let Some(frame) = blur_transition_frame(&source, divisor) {
            let texture = gtk4::gdk::Texture::for_pixbuf(&frame);
            let picture = gtk4::Picture::new();
            picture.set_paintable(Some(&texture));
            picture.set_can_shrink(true);
            picture.set_hexpand(true);
            picture.set_vexpand(true);
            transition
                .stack
                .add_named(&picture, Some(&format!("blur-{}", step + 1)));
            transition
                .stack
                .set_visible_child_name(&format!("blur-{}", step + 1));
            transition.last_blur_frame.set(step + 1);
        }

        if step + 1 < BLUR_DIVISORS.len() {
            schedule_blur_step(transition, source, step + 1);
        } else {
            glib::timeout_add_local_once(Duration::from_millis(70), move || {
                fade_out_transition(transition);
            });
        }
    });
}

fn blur_transition_frame(
    source: &gtk4::gdk_pixbuf::Pixbuf,
    divisor: i32,
) -> Option<gtk4::gdk_pixbuf::Pixbuf> {
    let width = source.width();
    let height = source.height();
    let reduced = source.scale_simple(
        (width / divisor).max(1),
        (height / divisor).max(1),
        gtk4::gdk_pixbuf::InterpType::Bilinear,
    )?;
    reduced.scale_simple(width, height, gtk4::gdk_pixbuf::InterpType::Bilinear)
}

fn fade_out_transition(transition: LockTransition) {
    let started = Instant::now();
    glib::timeout_add_local(Duration::from_millis(16), move || {
        if transition.cancelled.get() {
            return glib::ControlFlow::Break;
        }

        let progress = (started.elapsed().as_secs_f64() / 0.16).clamp(0.0, 1.0);
        transition.stack.set_opacity(1.0 - progress);

        if progress >= 1.0 {
            transition.stack.set_visible(false);
            glib::ControlFlow::Break
        } else {
            glib::ControlFlow::Continue
        }
    });
}

fn setup_clock_timer(clock_label: &gtk4::Label, date_label: &gtk4::Label) {
    let update = |cl: &gtk4::Label, dl: &gtk4::Label| {
        let now = chrono::Local::now();
        let time = now.format("%H:%M").to_string();
        let date = now.format("%A, %B %d").to_string();
        cl.set_text(&time);
        dl.set_text(&date);
    };
    update(clock_label, date_label);

    let cl = clock_label.clone();
    let dl = date_label.clone();
    glib::timeout_add_seconds_local(1, move || {
        let now = chrono::Local::now();
        let time = now.format("%H:%M").to_string();
        let date = now.format("%A, %B %d").to_string();
        cl.set_text(&time);
        dl.set_text(&date);
        glib::ControlFlow::Continue
    });
}

fn setup_auth_handler(
    on_unlock: Rc<dyn Fn()>,
    entry: &gtk4::PasswordEntry,
    status_label: &gtk4::Label,
    card_box: &gtk4::Box,
) {
    let username = std::env::var("USER").unwrap_or_else(|_| "user".to_string());
    let entry_clone = entry.clone();
    let status_label_clone = status_label.clone();
    let card_clone = card_box.clone();

    entry.connect_activate(move |_| {
        let password = entry_clone.text().to_string();
        entry_clone.set_text("");

        if verify_password(&username, &password) {
            on_unlock();
        } else {
            status_label_clone.set_text(&services::trans("lock.status_incorrect"));
            status_label_clone.add_css_class("error");
            card_clone.add_css_class("shake-error");

            let status_lbl = status_label_clone.clone();
            let card_box_ref = card_clone.clone();
            glib::timeout_add_local_once(Duration::from_millis(1600), move || {
                status_lbl.set_text(&services::trans("lock.status"));
                status_lbl.remove_css_class("error");
                card_box_ref.remove_css_class("shake-error");
            });
        }
    });
}

fn setup_focus_management(window: &gtk4::ApplicationWindow, entry: &gtk4::PasswordEntry) {
    let entry_focus = entry.clone();
    glib::timeout_add_local_once(Duration::from_millis(100), move || {
        entry_focus.grab_focus();
    });

    let entry_click = entry.clone();
    let click_gesture = gtk4::GestureClick::new();
    click_gesture.connect_pressed(move |_, _, _, _| {
        entry_click.grab_focus();
    });
    window.add_controller(click_gesture);
}

pub fn capture_transition_images() -> Vec<Option<PathBuf>> {
    let Some(display) = gtk4::gdk::Display::default() else {
        return Vec::new();
    };

    let monitors = display.monitors();
    let count = monitors.n_items();
    if count == 0 {
        return vec![capture_transition_image(None, 0)];
    }

    (0..count)
        .map(|index| {
            let monitor = monitors
                .item(index)
                .and_then(|obj| obj.downcast::<gtk4::gdk::Monitor>().ok());
            monitor
                .as_ref()
                .and_then(|monitor| capture_transition_image(monitor.connector().as_deref(), index))
        })
        .collect()
}

fn capture_transition_image(output: Option<&str>, index: u32) -> Option<PathBuf> {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    let prefix = format!("babydra-lock-{}-{index}-{nonce}", std::process::id());
    let raw_path = std::env::temp_dir().join(format!("{prefix}-raw.png"));

    let mut grim = std::process::Command::new("grim");
    if let Some(output) = output {
        grim.args(["-o", output]);
    }
    let captured = grim
        .args(["-t", "png", "-l", "1"])
        .arg(&raw_path)
        .status()
        .map(|status| status.success())
        .unwrap_or(false);

    if !captured {
        let _ = std::fs::remove_file(raw_path);
        return None;
    }

    Some(raw_path)
}
