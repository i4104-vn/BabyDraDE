//! System-action and text-editing port implementations for `BabydraState`.

use common::ShortcutTarget;

use crate::core::shortcuts::Mods;
use crate::shared::ports::{GesturePolicy, QueryText, SystemActions};
use crate::modules::shell::launcher::search::SearchText;
use crate::app::models::runtime::BabydraState;

impl QueryText for SearchText {
    fn insert(&mut self, text: &str) {
        SearchText::insert(self, text);
    }
    fn backspace(&mut self) {
        SearchText::backspace(self);
    }
    fn delete(&mut self) {
        SearchText::delete(self);
    }
    fn delete_word(&mut self) {
        SearchText::delete_word(self);
    }
    fn move_caret(&mut self, direction: i32, word: bool, extend: bool) {
        SearchText::move_caret(self, direction, word, extend);
    }
    fn home(&mut self, extend: bool) {
        SearchText::home(self, extend);
    }
    fn end(&mut self, extend: bool) {
        SearchText::end(self, extend);
    }
    fn select_all(&mut self) {
        SearchText::select_all(self);
    }
}

impl SystemActions for BabydraState {
    fn gesture_config(&self) -> &GesturePolicy {
        self.config.gesture_config()
    }

    fn touchpad_natural_scroll(&self) -> bool {
        self.config.touchpad_natural_scroll()
    }

    fn mouse_natural_scroll(&self) -> bool {
        self.config.mouse_natural_scroll()
    }

    fn reload_input_config(&mut self) {
        BabydraState::reload_input_config(self);
    }

    fn reload_config(&mut self) {
        BabydraState::reload_config(self);
    }

    fn shortcut_command(&self, mods: Mods, keysym: u32) -> Option<String> {
        self.config.shortcut_command(mods, keysym)
    }

    fn registered_shortcut(&self, mods: Mods, keysym: u32) -> Option<ShortcutTarget> {
        self.config.registered_target(mods, keysym)
    }

    fn reload_shortcuts(&self) {
        self.config.reload_shortcuts();
    }

    fn launch_command(&self, command: String) {
        self.config.launch_command(command);
    }

    fn lock_screen(&self) {
        crate::modules::shell::lock::spawn();
    }

    fn trigger_screenshot(&self) {
        crate::modules::shell::screenshot::trigger();
    }
}
