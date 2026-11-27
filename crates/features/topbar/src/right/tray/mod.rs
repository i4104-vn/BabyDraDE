//! Panel tray: a compact three-icon row that expands on hover.
//!
//! The compact row always ends with the temperature widget. Hovering the row
//! reveals a popup that grows down and to the left from the panel's right
//! corner, laying out every icon in rows of three with the temperature pinned
//! to the top-right. The popup is not click-dismissed: it stays open while the
//! pointer is on either surface or a context menu is open, and closes after a
//! short grace period once focus has fully left.

mod menu;
mod render;
mod temperature;

use gtk4::glib;
use gtk4::prelude::*;
use render::TrayVisibility;
use services::tray::{activate_item, get_tray_items, TrayItem};
use std::cell::RefCell;
use std::rc::Rc;
use std::time::{Duration, Instant};

thread_local! {
    static LAST_ACTIVE_WINDOW: RefCell<Option<(String, String)>> = const { RefCell::new(None) };
}

/// Duration of the corner reveal; shared by the popover slide and `Revealer`.
const REVEAL_DURATION_MS: u32 = 220;

/// A popover can only be presented when it still has an anchor inside a
/// realized toplevel. `gtk_popover_realize()` dereferences the parent's native
/// surface unconditionally, so presenting a detached popover segfaults GTK.
fn popover_anchored(popover: &gtk4::Popover) -> bool {
    popover.parent().is_some() && popover.root().and_then(|root| root.surface()).is_some()
}

/// Clicking another application window cannot leave a context menu open.
fn active_window_changed() -> bool {
    let current = services::system::get_active_window().map(|info| (info.app_id, info.title));
    LAST_ACTIVE_WINDOW.with(|last| {
        let mut last = last.borrow_mut();
        let changed = last.as_ref() != current.as_ref();
        *last = current;
        changed
    })
}

pub fn create_tray_widget(window: &gtk4::ApplicationWindow) -> gtk4::Box {
    let tray_container = render::build_tray_container();
    // The temperature item is always present, so the tray itself is always visible.
    tray_container.set_visible(true);

    // Two independent instances keep the compact row and the expanded grid
    // from fighting over a single parent; temperature.rs polling is untouched.
    let collapsed_temperature = temperature::create_temperature_widget();
    let expanded_temperature = temperature::create_temperature_widget();

    let expanded_grid = render::build_expanded_grid();
    expanded_grid.attach(
        &expanded_temperature,
        render::TRAY_ICONS_PER_ROW as i32,
        0,
        1,
        1,
    );

    let expanded_box = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    expanded_box.add_css_class("panel-tray-box");
    expanded_box.add_css_class("panel-tray-expanded");
    expanded_box.set_halign(gtk4::Align::End);
    expanded_box.set_valign(gtk4::Align::Start);
    expanded_box.append(&expanded_grid);

    // `SlideDown` grows the list from the panel edge; the popover slide below
    // adds the leftward component, so the reveal originates at the top-right.
    let revealer = gtk4::Revealer::new();
    revealer.set_transition_type(gtk4::RevealerTransitionType::SlideDown);
    revealer.set_transition_duration(REVEAL_DURATION_MS);
    revealer.set_halign(gtk4::Align::End);
    revealer.set_valign(gtk4::Align::Start);
    revealer.set_child(Some(&expanded_box));

    let popover = gtk4::Popover::new();
    popover.set_parent(&tray_container);
    popover.set_autohide(false);
    popover.set_has_arrow(false);
    popover.set_position(gtk4::PositionType::Bottom);
    popover.set_child(Some(&revealer));

    let state = Rc::new(RefCell::new(TrayVisibility::default()));
    // `None` forces the first refresh to build both views even when no SNI
    // items are registered yet (the temperature widget is still present).
    let last_snapshot: Rc<RefCell<Option<Vec<TrayItem>>>> = Rc::new(RefCell::new(None));
    let animation: Rc<RefCell<Option<gtk4::TickCallbackId>>> = Rc::new(RefCell::new(None));
    // Only these widgets are rebuilt. The popover is also a child of
    // `tray_container` (via `set_parent`) and must never be swept away by the
    // refresh below, or `popup()` would later crash GTK.
    let collapsed_children: Rc<RefCell<Vec<gtk4::Widget>>> = Rc::new(RefCell::new(Vec::new()));

    // Hover tracking for the compact row in the panel.
    let tray_motion = gtk4::EventControllerMotion::new();
    let tray_state = state.clone();
    tray_motion.connect_enter(move |_, _, _| {
        let mut state = tray_state.borrow_mut();
        state.pointer_in_tray = true;
        state.left_at = None;
    });
    let tray_state = state.clone();
    tray_motion.connect_leave(move |_| {
        let mut state = tray_state.borrow_mut();
        state.pointer_in_tray = false;
        state.left_at = Some(Instant::now());
    });
    tray_container.add_controller(tray_motion);

    // Hover tracking for the expanded surface, including its buttons. Motion
    // events from children propagate to the popover controller.
    let expanded_motion = gtk4::EventControllerMotion::new();
    let expanded_state = state.clone();
    expanded_motion.connect_enter(move |_, _, _| {
        let mut state = expanded_state.borrow_mut();
        state.pointer_in_expanded = true;
        state.left_at = None;
    });
    let expanded_state = state.clone();
    expanded_motion.connect_leave(move |_| {
        let mut state = expanded_state.borrow_mut();
        state.pointer_in_expanded = false;
        state.left_at = Some(Instant::now());
    });
    popover.add_controller(expanded_motion);

    let tray_for_refresh = tray_container.clone();
    let window_for_refresh = window.clone();
    let snapshot_for_refresh = last_snapshot.clone();
    let collapsed_temp_for_refresh = collapsed_temperature.clone();
    let grid_for_refresh = expanded_grid.clone();
    let expanded_temp_for_refresh = expanded_temperature.clone();
    let collapsed_children_for_refresh = collapsed_children.clone();

    glib::timeout_add_local(Duration::from_millis(500), move || {
        // Clicking another application window changes the compositor's active
        // window; a menu cannot stay open over a different app.
        if active_window_changed() {
            ui_kit::overlay::context_menu::dismiss_all();
        }

        let current_items = get_tray_items();
        let needs_rebuild = snapshot_for_refresh.borrow().as_ref() != Some(&current_items);

        if needs_rebuild {
            // Compact row: icons first, temperature always last on the right.
            // Remove only the widgets this refresh owns; the popover is an
            // anchor child of the same box and must stay parented.
            let mut children = collapsed_children_for_refresh.borrow_mut();
            for child in children.drain(..) {
                tray_for_refresh.remove(&child);
            }
            for item in render::collapsed_tray_items(&current_items) {
                let btn = render::build_tray_button(item);
                attach_tray_click_gesture(&btn, item, &window_for_refresh);
                tray_for_refresh.append(&btn);
                children.push(btn.upcast());
            }
            tray_for_refresh.append(&collapsed_temp_for_refresh);
            children.push(collapsed_temp_for_refresh.clone().upcast());
            drop(children);

            // Expanded grid: all icons in rows of three, temperature on top-right.
            while let Some(child) = grid_for_refresh.first_child() {
                grid_for_refresh.remove(&child);
            }
            for (row, items) in render::chunk_tray_items(&current_items)
                .into_iter()
                .enumerate()
            {
                for (col, item) in items.iter().enumerate() {
                    let btn = render::build_tray_button(item);
                    attach_tray_click_gesture(&btn, item, &window_for_refresh);
                    grid_for_refresh.attach(&btn, col as i32, row as i32, 1, 1);
                }
            }
            grid_for_refresh.attach(
                &expanded_temp_for_refresh,
                render::TRAY_ICONS_PER_ROW as i32,
                0,
                1,
                1,
            );

            *snapshot_for_refresh.borrow_mut() = Some(current_items);
        }

        glib::ControlFlow::Continue
    });

    // Visibility is polled more often than the 500 ms item refresh so that
    // menu dismissal and the close grace period are both observed promptly.
    let state_for_close = state.clone();
    let popover_for_close = popover.clone();
    let revealer_for_close = revealer.clone();
    let animation_for_close = animation.clone();
    let was_menu_open = Rc::new(RefCell::new(false));

    glib::timeout_add_local(Duration::from_millis(100), move || {
        let menu_open = ui_kit::overlay::context_menu::any_open();
        let mut state = state_for_close.borrow_mut();

        if menu_open != *was_menu_open.borrow() {
            // A menu that was keeping us open just closed: restart the grace
            // period so focus returning to the tray does not see a flicker.
            if *was_menu_open.borrow()
                && !menu_open
                && !state.pointer_in_tray
                && !state.pointer_in_expanded
            {
                state.left_at = Some(Instant::now());
            }
            *was_menu_open.borrow_mut() = menu_open;
        }
        state.menu_open = menu_open;

        let show = render::should_show_expanded(&state, Instant::now())
            && popover_anchored(&popover_for_close);
        let was_visible = popover_for_close.is_visible();
        drop(state);

        if show && !was_visible {
            // Re-fit each time: the popover is a separate surface below the
            // short panel, and End alignment keeps its right edge on the tray.
            ui_kit::overlay::popover::fit_popover_with_policy(
                &popover_for_close,
                gtk4::PositionType::Bottom,
                ui_kit::overlay::popover::CrossAlign::End,
                8,
                4,
                ui_kit::overlay::popover::FitPolicy::KeepSide,
            );
            popover_for_close.popup();
            animate_open(
                &popover_for_close,
                &revealer_for_close,
                &animation_for_close,
            );
        } else if !show && was_visible {
            stop_animation(&animation_for_close, &popover_for_close);
            revealer_for_close.set_reveal_child(false);
            popover_for_close.popdown();
            let mut state = state_for_close.borrow_mut();
            state.pointer_in_expanded = false;
        }

        glib::ControlFlow::Continue
    });

    tray_container
}

/// Slides the popover in from the top-right and simultaneously reveals its
/// contents downward, so the visible area grows down and to the left.
fn animate_open(
    popover: &gtk4::Popover,
    revealer: &gtk4::Revealer,
    animation: &Rc<RefCell<Option<gtk4::TickCallbackId>>>,
) {
    stop_animation(animation, popover);
    revealer.set_reveal_child(false);

    let started = Instant::now();
    let popover_for_tick = popover.clone();
    let animation_for_tick = animation.clone();
    let id = popover.add_tick_callback(move |_, _| {
        let seconds = started.elapsed().as_secs_f64();
        let progress = (seconds / (REVEAL_DURATION_MS as f64 / 1000.0)).min(1.0);
        let eased = 1.0 - (1.0 - progress).powi(3);
        let remaining = 1.0 - eased;

        // Positive x / negative y: start up and to the right of the final
        // position, then settle down-left into place.
        popover_for_tick.set_offset(
            (remaining * 56.0).round() as i32,
            (remaining * -28.0).round() as i32,
        );

        if progress >= 1.0 {
            popover_for_tick.set_offset(0, 0);
            if let Some(id) = animation_for_tick.borrow_mut().take() {
                id.remove();
            }
            glib::ControlFlow::Break
        } else {
            glib::ControlFlow::Continue
        }
    });

    *animation.borrow_mut() = Some(id);
    revealer.set_reveal_child(true);
}

/// Cancels an in-flight reveal and restores the fitted popover position.
fn stop_animation(animation: &Rc<RefCell<Option<gtk4::TickCallbackId>>>, popover: &gtk4::Popover) {
    if let Some(id) = animation.borrow_mut().take() {
        id.remove();
    }
    popover.set_offset(0, 0);
}

fn attach_tray_click_gesture(
    btn: &gtk4::Button,
    item: &TrayItem,
    window: &gtk4::ApplicationWindow,
) {
    let item = item.clone();
    let btn_c = btn.clone();
    let win_c = window.clone();

    let gesture = gtk4::GestureClick::new();
    gesture.set_button(0);

    gesture.connect_pressed(move |g, _, click_x, click_y| {
        let button_num = g.current_button();
        if button_num == 0 {
            return;
        }

        let (window_x, window_y) = btn_c
            .translate_coordinates(&win_c, click_x, click_y)
            .unwrap_or((click_x, click_y));
        let x = window_x as i32 + crate::PANEL_MARGIN_X;
        let y = window_y as i32 + crate::PANEL_MARGIN_Y;

        if !item.menu_path.is_empty() && button_num != 2 {
            menu::show_menu(&item, &btn_c, x, y, button_num);
        } else {
            activate_item(&item, x, y, button_num);
        }
    });

    btn.add_controller(gesture);
}
