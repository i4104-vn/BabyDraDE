//! Pointer press handling.

use smithay::{desktop::{layer_map_for_output, PopupManager}, input::pointer::MotionEvent};

use common::Action;
use crate::adapters::input::{InputContext, MoveState, ResizeState};

use super::models::ButtonContext;

pub(super) fn press<C: InputContext>(ctx: &mut ButtonContext<'_, C>) {
            ctx.state.pointer_pressed_buttons_mut().insert(ctx.button_code);

            if ctx.state.switcher_is_active() {
                let px = ctx.state.pointer_location().x;
                let py = ctx.state.pointer_location().y;

                if let Some(clicked_idx) = ctx.state.switcher_index_at((px, py), 0.0) {
                    if let Some(window) = ctx.state.switcher_window_at(clicked_idx) {
                        ctx.state.unminimize_window(&window);
                        crate::adapters::input::keyboard::focus::focus_window(
                            ctx.state, window, ctx.serial,
                        );
                    }
                    ctx.state.switcher_begin_close();
                    return;
                }
            }

            // While the search overlay is expanded it owns the whole screen:
            // clicking a result activates it, clicking outside returns to the
            // docked launcher.
            if ctx.button_code == common::BTN_LEFT
                && !ctx.state.pointer_is_grabbed()
                && ctx.state.launcher_search_click(ctx.state.pointer_location())
            {
                return;
            }

            // The launcher is compositor-owned UI: a click on one of its icons
            // launches that app and must not reach (or raise) any window.
            if !ctx.state.switcher_is_active()
                && ctx.button_code == common::BTN_LEFT
                && !ctx.state.pointer_is_grabbed()
            {
                if let Some(index) = ctx.state.launcher_index_at(ctx.state.pointer_location()) {
                    ctx.state.launcher_launch(index);
                    return;
                }
                // Clicking elsewhere dismisses the launcher.
                if ctx.state.launcher_is_visible() {
                    ctx.state.launcher_toggle();
                }
            }

            if !ctx.state.pointer_is_grabbed() {
                // 1. Check window border for interactive resize
                if ctx.button_code == common::BTN_LEFT {
                    if let Some((window, loc, edge)) =
                        ctx.state.window_border_under(ctx.state.pointer_location())
                    {
                        ctx.state.space_mut().raise_element(&window, true);
                        ctx.state.raise_window(&window);
                        ctx.state.set_keyboard_focus(
                            Some(window.toplevel().unwrap().wl_surface().clone()),
                            ctx.serial,
                        );
                        ctx.state.space().elements().for_each(|w| {
                            let active = w == &window;
                            w.set_activated(active);
                            if let Some(top) = w.toplevel() {
                                top.send_pending_configure();
                            }
                        });
                        ctx.state.update_active_window(Some(&window));
                        let size = window.geometry().size;
                        *ctx.state.resize_state_mut() = Some(ResizeState {
                            window,
                            initial_window_location: loc,
                            initial_window_size: size,
                            initial_pointer_location: ctx.state.pointer_location(),
                            edges: edge,
                            button: ctx.button_code,
                        });
                        return;
                    }
                }

                if let Some((window, location)) = ctx.state.window_under(ctx.state.pointer_location()) {
                    // Clicking an application window dismisses any open layer popups (e.g. context menus)
                    for output in ctx.state.space().outputs() {
                        let layer_map = layer_map_for_output(output);
                        for layer in layer_map.layers() {
                            let surface = layer.wl_surface();
                            for (popup, _) in PopupManager::popups_for_surface(surface) {
                                let _ = PopupManager::dismiss_popup(surface, &popup);
                            }
                        }
                    }

                    ctx.state.space_mut().raise_element(&window, true);
                    ctx.state.raise_window(&window);
                    ctx.state.set_keyboard_focus(
                        Some(window.toplevel().unwrap().wl_surface().clone()),
                        ctx.serial,
                    );
                    ctx.state.space().elements().for_each(|w| {
                        let active = w == &window;
                        w.set_activated(active);
                        if let Some(top) = w.toplevel() {
                            top.send_pending_configure();
                        }
                    });
                    ctx.state.update_active_window(Some(&window));

                    if let Some(action) =
                        ctx.state.titlebar_action(&window, location, ctx.state.pointer_location())
                    {
                        match action {
                            Action::Close => {
                                *ctx.state.last_titlebar_click_mut() = None;
                                ctx.state.close_window(&window);
                            }
                            Action::Minimize => {
                                *ctx.state.last_titlebar_click_mut() = None;
                                ctx.state.minimize_window(&window);
                            }
                            Action::Maximize => {
                                *ctx.state.last_titlebar_click_mut() = None;
                                ctx.state.toggle_maximize(&window);
                            }
                            Action::Move => {
                                let now = std::time::Instant::now();
                                let is_double_click =
                                    if let Some((last_time, last_pos, ref last_win)) =
                                        ctx.state.last_titlebar_click_mut().take()
                                    {
                                        let elapsed = now.duration_since(last_time).as_millis();
                                        let dx = ctx.state.pointer_location().x - last_pos.x;
                                        let dy = ctx.state.pointer_location().y - last_pos.y;
                                        last_win == &window
                                            && elapsed <= common::DOUBLE_CLICK_TIMEOUT_MS
                                            && (dx * dx + dy * dy)
                                                <= common::DOUBLE_CLICK_DISTANCE_SQ
                                    } else {
                                        false
                                    };

                                if is_double_click {
                                    ctx.state.toggle_maximize(&window);
                                } else {
                                    *ctx.state.last_titlebar_click_mut() =
                                        Some((now, ctx.state.pointer_location(), window.clone()));
                                    *ctx.state.move_state_mut() = Some(MoveState {
                                        window: window.clone(),
                                        initial_window_location: location,
                                        initial_pointer_location: ctx.state.pointer_location(),
                                        has_dragged: false,
                                        button: ctx.button_code,
                                    });
                                }
                            }
                        }
                        return;
                    }

                    *ctx.state.last_titlebar_click_mut() = None;

                    // Ensure pointer focus is updated to the raised window surface
                    let under = ctx.state.surface_under(ctx.state.pointer_location());
                    ctx.state.pointer_motion(
                        under,
                        &MotionEvent {
                            location: ctx.state.pointer_location(),
                            serial: ctx.serial,
                            time: ctx.time,
                        },
                    );
                    ctx.state.pointer_frame();

                    // Drag or resize window from anywhere if holding Super (Win) key
                    let mods = ctx.state.keyboard_modifiers();
                    if mods.logo {
                        if ctx.button_code == common::BTN_RIGHT {
                            let size = window.geometry().size;
                            *ctx.state.resize_state_mut() = Some(ResizeState {
                                window: window.clone(),
                                initial_window_location: location,
                                initial_window_size: size,
                                initial_pointer_location: ctx.state.pointer_location(),
                                edges: smithay::reexports::wayland_protocols::xdg::shell::server::xdg_toplevel::ResizeEdge::BottomRight,
                                button: ctx.button_code,
                            });
                            return;
                        } else {
                            *ctx.state.move_state_mut() = Some(MoveState {
                                window: window.clone(),
                                initial_window_location: location,
                                initial_pointer_location: ctx.state.pointer_location(),
                                has_dragged: false,
                                button: ctx.button_code,
                            });
                            return;
                        }
                    }
                } else {
                    let clicked_layer = ctx.state.layer_under(ctx.state.pointer_location());
                    let is_top_or_overlay = matches!(
                        clicked_layer,
                        Some(smithay::wayland::shell::wlr_layer::Layer::Top)
                            | Some(smithay::wayland::shell::wlr_layer::Layer::Overlay)
                    );

                    // Ensure pointer focus is updated to the clicked surface
                    let under = ctx.state.surface_under(ctx.state.pointer_location());
                    ctx.state.pointer_motion(
                        under,
                        &MotionEvent {
                            location: ctx.state.pointer_location(),
                            serial: ctx.serial,
                            time: ctx.time,
                        },
                    );
                    ctx.state.pointer_frame();

                    // When interacting with panels, topbar, or overlays, do NOT lose active app focus
                    if !is_top_or_overlay {
                        for output in ctx.state.space().outputs() {
                            let layer_map = layer_map_for_output(output);
                            for layer in layer_map.layers() {
                                let surface = layer.wl_surface();
                                for (popup, _) in PopupManager::popups_for_surface(surface) {
                                    let _ = PopupManager::dismiss_popup(surface, &popup);
                                }
                            }
                        }

                        ctx.state.space().elements().for_each(|window| {
                            window.set_activated(false);
                            window.toplevel().unwrap().send_pending_configure();
                        });
                        ctx.state.set_keyboard_focus(None, ctx.serial);
                        ctx.state.update_active_window(None);
                    }
                }
            }
}
