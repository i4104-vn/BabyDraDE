use domain::{KeyCombo, KeyState, Modifiers};

/// Handles keyboard state, tracking pressed keys and active modifiers.
#[derive(Debug, Default)]
pub struct KeyboardDevice {
    pub modifiers: Modifiers,
}

impl KeyboardDevice {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn update_modifier(&mut self, logo: bool, alt: bool, ctrl: bool, shift: bool) {
        self.modifiers = Modifiers::new(logo, alt, ctrl, shift);
    }

    pub fn process_key(&self, key_name: &str, state: KeyState) -> (KeyCombo, KeyState) {
        let combo = KeyCombo::new(self.modifiers, key_name);
        (combo, state)
    }
}
