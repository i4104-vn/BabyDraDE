//! Standardized Fluent Popover overlay component.

use gtk4::prelude::*;
use gtk4::{Box, Button, Orientation, Popover, PositionType, Widget};
use std::ops::Deref;

/// Creates a standardized Popover widget with fluent styling.
pub fn create_popover(
    parent: &impl IsA<Widget>,
    position: PositionType,
    css_class: &str,
) -> Popover {
    let popover = Popover::builder()
        .has_arrow(false)
        .autohide(true)
        .position(position)
        .build();
    popover.set_parent(parent.as_ref());
    popover.add_css_class("fluent-popover");
    if !css_class.is_empty() {
        for cls in css_class.split_whitespace() {
            popover.add_css_class(cls);
        }
    }
    popover
}

/// A Fluent Popover overlay component with an internal vertical container.
pub struct FluentPopover {
    popover: Popover,
    container: Box,
}

impl Deref for FluentPopover {
    type Target = Popover;

    fn deref(&self) -> &Self::Target {
        &self.popover
    }
}

impl FluentPopover {
    /// Creates a new styled Popover anchored to `parent` at `position`.
    pub fn new(parent: &impl IsA<Widget>, position: PositionType) -> Self {
        let popover = create_popover(parent, position, "");
        let container = Box::new(Orientation::Vertical, 4);
        popover.set_child(Some(&container));

        Self { popover, container }
    }

    /// Creates a new styled Popover with custom CSS classes.
    pub fn with_css_class(
        parent: &impl IsA<Widget>,
        position: PositionType,
        css_class: &str,
    ) -> Self {
        let popover = create_popover(parent, position, css_class);
        let container = Box::new(Orientation::Vertical, 4);
        popover.set_child(Some(&container));

        Self { popover, container }
    }

    /// Sets the position of the popover relative to its parent.
    pub fn set_position(&self, position: PositionType) {
        self.popover.set_position(position);
    }

    /// Adds a CSS class to the popover.
    pub fn add_css_class(&self, class: &str) {
        self.popover.add_css_class(class);
    }

    /// Appends a child widget into the internal container.
    pub fn append<W: IsA<Widget>>(&self, widget: &W) {
        self.container.append(widget);
    }

    /// Sets custom child widget, replacing the internal container.
    pub fn set_child<W: IsA<Widget>>(&self, child: Option<&W>) {
        self.popover.set_child(child);
    }

    /// Appends a clickable action button inside the popover that dismisses the popover upon click.
    pub fn add_item(&self, label: &str, on_click: impl Fn() + 'static) -> Button {
        let btn = Button::builder()
            .label(label)
            .css_classes(["fluent-btn"])
            .halign(gtk4::Align::Fill)
            .build();

        let pop = self.popover.clone();
        btn.connect_clicked(move |_| {
            pop.popdown();
            on_click();
        });

        self.container.append(&btn);
        btn
    }

    /// Displays the popover.
    pub fn popup(&self) {
        self.popover.popup();
    }

    /// Dismisses the popover.
    pub fn popdown(&self) {
        self.popover.popdown();
    }

    /// Returns a reference to the GTK4 Popover widget.
    pub fn widget(&self) -> &Popover {
        &self.popover
    }

    /// Returns a reference to the internal Box container.
    pub fn container(&self) -> &Box {
        &self.container
    }
}
