//! Interactive button widget for the recording popover controls.

use std::cell::RefCell;
use std::ops::Deref;
use std::rc::Rc;

use gtk4::prelude::*;
use gtk4::Button;
use ui_kit::components::create_icon_button;
use ui_kit::icons::get_icon;

#[derive(Clone)]
pub struct RecordingButtonWidget {
    pub button: Button,
    current_icon: Rc<RefCell<String>>,
}

impl RecordingButtonWidget {
    pub fn new(icon_name: &str, tooltip: &str, css_class: &str) -> Self {
        let mut classes = vec!["circular"];
        if !css_class.is_empty() {
            classes.push(css_class);
        }
        let button = create_icon_button(icon_name, 18, &classes, Some(tooltip), || {});
        button.set_cursor_from_name(Some("pointer"));

        Self {
            button,
            current_icon: Rc::new(RefCell::new(icon_name.to_string())),
        }
    }

    pub fn set_icon_and_title(&self, icon_name: &str, title: &str) {
        self.button.set_tooltip_text(Some(title));
        let mut current = self.current_icon.borrow_mut();
        if *current != icon_name {
            *current = icon_name.to_string();
            self.button.set_child(Some(&get_icon(icon_name, 18)));
        }
    }

    pub fn set_alert(&self, is_alert: bool) {
        if is_alert {
            self.button.add_css_class("delete-btn");
        } else {
            self.button.remove_css_class("delete-btn");
        }
    }
}

impl Deref for RecordingButtonWidget {
    type Target = Button;

    fn deref(&self) -> &Self::Target {
        &self.button
    }
}
