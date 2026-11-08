use smithay::{
    backend::input::{
        AbsolutePositionEvent, ButtonState, Event, InputBackend, InputEvent, KeyState,
        KeyboardKeyEvent, PointerButtonEvent, PointerMotionEvent,
    },
    input::{
        keyboard::{keysyms, FilterResult},
        pointer::{ButtonEvent, MotionEvent},
    },
    reexports::wayland_server::protocol::wl_surface::WlSurface,
    utils::{Point, SERIAL_COUNTER},
};
use tracing::{info, warn};

use super::state::{BabydraState, MoveState};

pub(crate) fn spawn_new_terminal(socket_name: &std::ffi::OsStr) {
    let term = std::env::var("BABYDRADE_TEST_APP").ok().unwrap_or_else(|| {
        for candidate in [
            "foot",
            "alacritty",
            "kitty",
            "weston-terminal",
            "gnome-terminal",
            "xfce4-terminal",
            "konsole",
            "xterm",
            "urxvt",
            "st",
        ] {
            if let Ok(paths) = std::env::var("PATH") {
                for dir in std::env::split_paths(&paths) {
                    if dir.join(candidate).is_file() {
                        return candidate.to_string();
                    }
                }
            }
        }
        "foot".to_string()
    });

    info!("Super+T pressed: Spawning new terminal window: {term} (WAYLAND_DISPLAY={:?})", socket_name);
    if let Err(err) = std::process::Command::new(&term)
        .env("WAYLAND_DISPLAY", socket_name)
        .spawn()
    {
        warn!("Failed to spawn terminal '{term}': {err}");
    }
}

impl BabydraState {
    pub(crate) fn process_input_event<I: InputBackend>(&mut self, event: InputEvent<I>) {
        match event {
            InputEvent::Keyboard { event, .. } => {
                let serial = SERIAL_COUNTER.next_serial();
                let time = Event::time_msec(&event);
                let keyboard = self.seat.get_keyboard().unwrap();

                let mut spawn_terminal = false;
                keyboard.input::<(), _>(
                    self,
                    event.key_code(),
                    event.state(),
                    serial,
                    time,
                    |_state, modifiers, handle| {
                        // Check for Super (Win) + T key combination to spawn terminal
                        if event.state() == KeyState::Pressed && modifiers.logo {
                            let sym = handle.modified_sym();
                            if sym == keysyms::KEY_t.into() || sym == keysyms::KEY_T.into() {
                                spawn_terminal = true;
                                return FilterResult::Intercept(());
                            }
                        }
                        FilterResult::Forward
                    },
                );

                if spawn_terminal {
                    spawn_new_terminal(&self.socket_name);
                }
            }
            InputEvent::PointerMotion { event, .. } => {
                let pointer = self.seat.get_pointer().unwrap();
                let delta = event.delta();
                let current_pos = pointer.current_location();
                let mut position = current_pos + delta;

                if let Some(output) = self.space.outputs().next() {
                    if let Some(geo) = self.space.output_geometry(output) {
                        let max_x = (geo.loc.x + geo.size.w) as f64;
                        let max_y = (geo.loc.y + geo.size.h) as f64;
                        position.x = position.x.clamp(geo.loc.x as f64, max_x);
                        position.y = position.y.clamp(geo.loc.y as f64, max_y);
                    }
                }

                self.pointer_location = position;

                // Handle window dragging/moving
                if let Some(ref move_state) = self.move_state {
                    let delta = position - move_state.initial_pointer_location;
                    let new_loc = move_state.initial_window_location
                        + Point::from((delta.x as i32, delta.y as i32));
                    self.space.map_element(move_state.window.clone(), new_loc, false);
                }

                let under = self.surface_under(position);
                pointer.motion(
                    self,
                    under,
                    &MotionEvent {
                        location: position,
                        serial: SERIAL_COUNTER.next_serial(),
                        time: event.time_msec(),
                    },
                );
                pointer.frame(self);
            }
            InputEvent::PointerMotionAbsolute { event, .. } => {
                let output = self.space.outputs().next().unwrap();
                let output_geometry = self.space.output_geometry(output).unwrap();
                let position =
                    event.position_transformed(output_geometry.size) + output_geometry.loc.to_f64();
                let pointer = self.seat.get_pointer().unwrap();

                self.pointer_location = position;

                // Handle window dragging/moving
                if let Some(ref move_state) = self.move_state {
                    let delta = position - move_state.initial_pointer_location;
                    let new_loc = move_state.initial_window_location
                        + Point::from((delta.x as i32, delta.y as i32));
                    self.space.map_element(move_state.window.clone(), new_loc, false);
                }

                let under = self.surface_under(position);
                pointer.motion(
                    self,
                    under,
                    &MotionEvent {
                        location: position,
                        serial: SERIAL_COUNTER.next_serial(),
                        time: event.time_msec(),
                    },
                );
                pointer.frame(self);
            }
            InputEvent::PointerButton { event, .. } => {
                let pointer = self.seat.get_pointer().unwrap();
                let keyboard = self.seat.get_keyboard().unwrap();
                let serial = SERIAL_COUNTER.next_serial();

                if event.state() == ButtonState::Pressed && !pointer.is_grabbed() {
                    if let Some((window, location)) = self
                        .space
                        .element_under(pointer.current_location())
                        .map(|(window, location)| (window.clone(), location))
                    {
                        self.space.raise_element(&window, true);
                        keyboard.set_focus(
                            self,
                            Some(window.toplevel().unwrap().wl_surface().clone()),
                            serial,
                        );
                        self.space.elements().for_each(|window| {
                            window.toplevel().unwrap().send_pending_configure();
                        });

                        // Start dragging window if holding Super (Win) key OR clicking the top 36px header
                        let mods = keyboard.modifier_state();
                        let click_rel_y = pointer.current_location().y - location.y as f64;
                        if mods.logo || click_rel_y <= 36.0 {
                            self.move_state = Some(MoveState {
                                window: window.clone(),
                                initial_window_location: location,
                                initial_pointer_location: pointer.current_location(),
                            });
                        }
                    } else {
                        self.space.elements().for_each(|window| {
                            window.set_activated(false);
                            window.toplevel().unwrap().send_pending_configure();
                        });
                        keyboard.set_focus(self, Option::<WlSurface>::None, serial);
                    }
                } else if event.state() == ButtonState::Released && event.button_code() == 0x110 {
                    // Release window dragging on Left Mouse Button release
                    self.move_state = None;
                }

                pointer.button(
                    self,
                    &ButtonEvent {
                        button: event.button_code(),
                        state: event.state(),
                        serial,
                        time: event.time_msec(),
                    },
                );
                pointer.frame(self);
            }
            _ => {}
        }
    }
}
