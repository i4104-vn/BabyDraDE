//! Selection highlight and action dispatch for the power popover.

use gtk4::prelude::*;

use super::popover::PowerPopover;
use crate::center::features::power::models::PowerAction;

/// Highlights one of the four power buttons.
pub fn highlight_selection(popover: &PowerPopover, selected_index: usize) {
    for (index, button) in popover.buttons.iter().enumerate() {
        if index == selected_index {
            button.container.add_css_class("selected");
        } else {
            button.container.remove_css_class("selected");
        }
    }
}

/// Executes the power action at `index` and closes the popover.
pub fn execute_power_action(index: usize, popover: &PowerPopover) {
    popover.popdown();
    if let Some(action) = PowerAction::from_index(index) {
        action.execute();
    }
}
