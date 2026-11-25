//! Glassmorphic power menu popover anchored down below the Dynamic Island capsule.

use std::ops::Deref;

use gtk4::prelude::*;
use gtk4::{Align, Box as GtkBox, Label, Orientation};

use super::button::PowerButtonWidget;
use crate::center::ui::IslandPopover;

#[derive(Clone)]
pub struct PowerPopover {
    pub base: IslandPopover,
    pub buttons: [PowerButtonWidget; 4],
}

impl Deref for PowerPopover {
    type Target = IslandPopover;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl PowerPopover {
    /// Builds the power popover anchored down below the notch capsule.
    pub fn new(capsule: &GtkBox) -> Self {
        let base = IslandPopover::new_modal(
            capsule,
            "power-popover control-popover",
            "power-popover-box",
            400,
        );

        let header = GtkBox::new(Orientation::Horizontal, 8);
        header.add_css_class("power-popover-header");
        header.set_valign(Align::Center);

        let title_lbl = Label::new(Some(&services::i18n::trans("island.power")));
        title_lbl.add_css_class("power-header-title");
        title_lbl.set_valign(Align::Center);
        header.append(&title_lbl);

        let shortcut_hint = Label::new(Some("Win+F4"));
        shortcut_hint.add_css_class("power-header-shortcut");
        shortcut_hint.set_halign(Align::End);
        shortcut_hint.set_hexpand(true);
        shortcut_hint.set_valign(Align::Center);
        header.append(&shortcut_hint);

        base.popover_box.append(&header);

        let buttons_box = GtkBox::new(Orientation::Horizontal, 8);
        buttons_box.add_css_class("power-popover-buttons");
        buttons_box.set_halign(Align::Center);
        buttons_box.set_valign(Align::Center);
        buttons_box.set_hexpand(true);
        buttons_box.set_vexpand(false);
        buttons_box.set_homogeneous(true);

        let shutdown = PowerButtonWidget::new(
            "power",
            "1",
            &services::i18n::trans("island.power_shutdown"),
            "power-shutdown",
        );
        let reboot = PowerButtonWidget::new(
            "restart",
            "2",
            &services::i18n::trans("island.power_restart"),
            "power-reboot",
        );
        let suspend = PowerButtonWidget::new(
            "sleep",
            "3",
            &services::i18n::trans("island.power_suspend"),
            "power-suspend",
        );
        let logout = PowerButtonWidget::new(
            "logout",
            "4",
            &services::i18n::trans("island.power_logout"),
            "power-logout",
        );

        buttons_box.append(&shutdown.container);
        buttons_box.append(&reboot.container);
        buttons_box.append(&suspend.container);
        buttons_box.append(&logout.container);

        base.popover_box.append(&buttons_box);

        let hint_lbl = Label::new(Some(&services::i18n::trans("island.power_hint")));
        hint_lbl.add_css_class("power-popover-hint");
        hint_lbl.set_halign(Align::Center);
        base.popover_box.append(&hint_lbl);

        Self {
            base,
            buttons: [shutdown, reboot, suspend, logout],
        }
    }
}
