use crate::value_objects::{KeyCombo, Point};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum KeyState {
    Pressed,
    Released,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ButtonState {
    Pressed,
    Released,
}

/// Generic input event emitted from system or backend.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum InputEvent {
    Key { combo: KeyCombo, state: KeyState },
    PointerMotion { position: Point, delta: (i32, i32) },
    PointerButton { button: u32, state: ButtonState },
    PointerAxis { horizontal: f64, vertical: f64 },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputError {
    DeviceError(String),
    Disconnected,
}

impl std::fmt::Display for InputError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DeviceError(msg) => write!(f, "Input device error: {msg}"),
            Self::Disconnected => write!(f, "Input source disconnected"),
        }
    }
}

impl std::error::Error for InputError {}

/// Contract for keyboard and pointer input event sources.
pub trait InputSource {
    /// Polls for the next pending input event.
    fn poll_event(&mut self) -> Result<Option<InputEvent>, InputError>;
}
