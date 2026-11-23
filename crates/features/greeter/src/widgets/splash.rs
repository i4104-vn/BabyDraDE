//! Greeter splash screen: logo, title and loading spinner.

use gtk4::prelude::*;
use gtk4::{Align, Box as GtkBox, Label, Orientation, Spinner};

pub(crate) struct SplashWidget {
    pub(crate) container: GtkBox,
}

pub(crate) fn build() -> SplashWidget {
    let container = GtkBox::new(Orientation::Vertical, 16);
    container.set_valign(Align::Center);
    container.set_halign(Align::Center);
    container.add_css_class("splash-box");

    let content = GtkBox::new(Orientation::Vertical, 12);
    content.set_halign(Align::Center);

    let logo_wrapper = GtkBox::new(Orientation::Vertical, 0);
    logo_wrapper.add_css_class("splash-logo-wrapper");
    logo_wrapper.set_halign(Align::Center);
    logo_wrapper.append(&ui_kit::icons::get_logo_png(110));

    let title = Label::new(Some(&services::trans("greeter.os_name")));
    title.add_css_class("splash-title");
    let subtitle = Label::new(Some(&services::trans("greeter.initializing")));
    subtitle.add_css_class("splash-subtitle");

    let spinner = Spinner::new();
    spinner.add_css_class("splash-spinner");
    spinner.set_size_request(32, 32);
    spinner.start();

    content.append(&logo_wrapper);
    content.append(&title);
    content.append(&subtitle);
    content.append(&spinner);
    container.append(&content);

    SplashWidget { container }
}
