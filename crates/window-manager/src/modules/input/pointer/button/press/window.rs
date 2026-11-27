//! Window interaction press handling.

use smithay::reexports::wayland_protocols::xdg::shell::server::xdg_toplevel::ResizeEdge;

use crate::modules::input::{InputContext, MoveState, ResizeState};
use common::Action;

use crate::modules::input::pointer::button::ButtonContext;

use super::super::focus;
use super::{layer, shared};

/// Handles a press against an application window: border resize, title-bar
/// actions, and Super-modified move/resize.
///
/// # Arguments
/// * `ctx` - Mutable button context.
pub(super) fn pointer<C: InputContext>(ctx: &mut ButtonContext<'_, C>) {
    // 1. Check window border for interactive resize
    if ctx.button_code == common::BTN_LEFT {
        if let Some((window, loc, edge)) =
            ctx.state.window_border_under(ctx.state.pointer_location())
        {
            shared::raise_and_focus(ctx, &window);
            ctx.state.update_active_window(Some(&window));
            // The border is drawn by the compositor. Clear pointer focus so a
            // window behind the border does not receive this press.
            focus::clear(ctx);
            ctx.consume();
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
        shared::dismiss_popups(ctx);
        shared::raise_and_focus(ctx, &window);
        ctx.state.update_active_window(Some(&window));

        if let Some(action) =
            ctx.state
                .titlebar_action(&window, location, ctx.state.pointer_location())
        {
            // The titlebar is a compositor-drawn frame, not a client surface.
            // Keep the button away from any window sitting behind it while
            // still allowing the titlebar action to run below.
            focus::clear(ctx);
            ctx.consume();
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
                    let is_double_click = if let Some((last_time, last_pos, ref last_win)) =
                        ctx.state.last_titlebar_click_mut().take()
                    {
                        let elapsed = now.duration_since(last_time).as_millis();
                        let dx = ctx.state.pointer_location().x - last_pos.x;
                        let dy = ctx.state.pointer_location().y - last_pos.y;
                        last_win == &window
                            && elapsed <= common::DOUBLE_CLICK_TIMEOUT_MS
                            && (dx * dx + dy * dy) <= common::DOUBLE_CLICK_DISTANCE_SQ
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

        // Drag or resize window from anywhere if holding Super (Win) key.
        // The compositor owns that gesture, so the press must not reach the
        // client surface either.
        let mods = ctx.state.keyboard_modifiers();
        if mods.logo {
            focus::clear(ctx);
            ctx.consume();
            if ctx.button_code == common::BTN_RIGHT {
                let size = window.geometry().size;
                *ctx.state.resize_state_mut() = Some(ResizeState {
                    window: window.clone(),
                    initial_window_location: location,
                    initial_window_size: size,
                    initial_pointer_location: ctx.state.pointer_location(),
                    edges: ResizeEdge::BottomRight,
                    button: ctx.button_code,
                });
            } else {
                *ctx.state.move_state_mut() = Some(MoveState {
                    window: window.clone(),
                    initial_window_location: location,
                    initial_pointer_location: ctx.state.pointer_location(),
                    has_dragged: false,
                    button: ctx.button_code,
                });
            }
        } else {
            // Normal client-surface click: make sure the raised window owns
            // pointer focus before the button is forwarded.
            focus::refresh(ctx);
        }
    } else {
        layer::handle(ctx);
    }
}
