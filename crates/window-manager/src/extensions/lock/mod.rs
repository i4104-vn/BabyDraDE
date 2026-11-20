//! BabyDraDE Lock Screen — Multi-monitor lock screen with PAM authentication.

pub mod auth;
pub mod render;

use gtk4::prelude::*;
use render::{
    capture_transition_images, create_lock_window, reverse_lock_transition, LockTransition,
};
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};

static LOCK_SCREEN_RUNNING: AtomicBool = AtomicBool::new(false);

/// Starts the lock screen inside the window-manager process.
pub fn spawn() {
    if LOCK_SCREEN_RUNNING
        .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
        .is_err()
    {
        return;
    }

    let _ = std::thread::Builder::new()
        .name("babydra-lock-screen".to_string())
        .spawn(|| {
            run();
            LOCK_SCREEN_RUNNING.store(false, Ordering::Release);
        });
}

fn run() {
    let args: Vec<String> = std::env::args().collect();
    let mut custom_image = None;
    if args.len() > 1 {
        let mut i = 1;
        while i < args.len() {
            if (args[i] == "--image" || args[i] == "-i") && i + 1 < args.len() {
                custom_image = Some(args[i + 1].clone());
                i += 2;
            } else if !args[i].starts_with('-')
                && !args[i].contains("lock")
                && !args[i].contains("__widget")
            {
                custom_image = Some(args[i].clone());
                i += 1;
            } else {
                i += 1;
            }
        }
    }

    let application = gtk4::Application::new(Some("org.babydra.lock"), Default::default());

    let transition_paths = Rc::new(RefCell::new(Vec::new()));
    let transition_paths_on_activate = transition_paths.clone();
    let transition_stacks = Rc::new(RefCell::new(Vec::<LockTransition>::new()));
    let transition_stacks_on_activate = transition_stacks.clone();
    let unlock_started = Rc::new(Cell::new(false));

    application.connect_activate(move |app| {
        ui_kit::init_theme();

        transition_stacks_on_activate.borrow_mut().clear();
        unlock_started.set(false);

        let transition_images = capture_transition_images();
        *transition_paths_on_activate.borrow_mut() = transition_images
            .iter()
            .filter_map(|image| image.clone())
            .collect();

        let app_on_unlock = app.clone();
        let stacks_on_unlock = transition_stacks_on_activate.clone();
        let unlock_started_on_unlock = unlock_started.clone();

        let on_unlock = Rc::new(move || {
            if unlock_started_on_unlock.replace(true) {
                return;
            }

            let app = app_on_unlock.clone();
            reverse_lock_transition(
                stacks_on_unlock.borrow().clone(),
                Rc::new(move || app.quit()),
            );
        });

        let display = gtk4::gdk::Display::default().expect("Failed to get default GDK display");
        let monitors = display.monitors();
        let num_monitors = monitors.n_items();

        if num_monitors == 0 {
            create_lock_window(
                app,
                None,
                true,
                custom_image.as_deref(),
                transition_images.first().and_then(|image| image.as_deref()),
                transition_stacks_on_activate.clone(),
                on_unlock,
            );
        } else {
            for i in 0..num_monitors {
                if let Some(monitor) = monitors
                    .item(i)
                    .and_then(|obj| obj.downcast::<gtk4::gdk::Monitor>().ok())
                {
                    let is_primary = i == 0;
                    create_lock_window(
                        app,
                        Some(&monitor),
                        is_primary,
                        custom_image.as_deref(),
                        transition_images
                            .get(i as usize)
                            .and_then(|image| image.as_deref()),
                        transition_stacks_on_activate.clone(),
                        on_unlock.clone(),
                    );
                }
            }
        }
    });

    let _exit_code = application.run_with_args::<&str>(&[]);

    for image in transition_paths.borrow().iter() {
        let _ = std::fs::remove_file(image);
    }
}
