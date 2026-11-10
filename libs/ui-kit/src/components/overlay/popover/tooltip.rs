//! Rich Card-style Tooltip Popover component.

use gtk4::prelude::*;
use gtk4::{Box, Label, Orientation, Popover, PositionType, Widget};
use std::cell::RefCell;
use std::ops::Deref;
use std::rc::Rc;

/// A row in a structured tooltip card (Key -> Value).
#[derive(Debug, Clone)]
pub struct TooltipRow {
    pub key: String,
    pub val: String,
    pub css_class: Option<String>,
}

impl TooltipRow {
    pub fn new(key: &str, val: &str, css_class: Option<&str>) -> Self {
        Self {
            key: key.to_string(),
            val: val.to_string(),
            css_class: css_class.map(|s| s.to_string()),
        }
    }
}

type SuppressCallback = Rc<dyn Fn() -> bool>;
type SuppressFnCell = Rc<RefCell<Option<SuppressCallback>>>;

/// A rich tooltip popover displaying key-value structured data cards.
#[derive(Clone)]
pub struct TooltipPopover {
    pub popover: Popover,
    suppress_fn: SuppressFnCell,
}

pub type CardTooltip = TooltipPopover;

impl Deref for TooltipPopover {
    type Target = Popover;

    fn deref(&self) -> &Self::Target {
        &self.popover
    }
}

impl TooltipPopover {
    /// Creates a new `TooltipPopover` anchored to the specified parent widget.
    pub fn new(parent: &impl IsA<Widget>, position: PositionType) -> Self {
        let popover = Popover::builder()
            .has_arrow(false)
            .autohide(false)
            .position(position)
            .css_classes(["status-popover", "tooltip-popover"])
            .build();
        popover.set_parent(parent.as_ref());

        Self {
            popover,
            suppress_fn: Rc::new(RefCell::new(None)),
        }
    }

    /// Convenience helper to create and attach a card tooltip from multi-line "Key: Value" text.
    pub fn attach(anchor: &impl IsA<Widget>, title: &str, text: &str) -> Self {
        let rows = Self::parse_rows(text);
        let card = Self::build_card(title, &rows);
        let tooltip = Self::new(anchor, PositionType::Bottom);
        tooltip.popover.set_child(Some(&card));
        tooltip.attach_hover(anchor, None);
        tooltip
    }

    /// Convenience helper to create and attach a card tooltip with explicit `TooltipRow` items.
    pub fn attach_rows(anchor: &impl IsA<Widget>, title: &str, rows: &[TooltipRow]) -> Self {
        let card = Self::build_card(title, rows);
        let tooltip = Self::new(anchor, PositionType::Bottom);
        tooltip.popover.set_child(Some(&card));
        tooltip.attach_hover(anchor, None);
        tooltip
    }

    /// Configures a predicate callback that conditionally suppresses tooltip display (e.g. during drag or active menus).
    pub fn set_suppress_fn(&self, f: impl Fn() -> bool + 'static) {
        *self.suppress_fn.borrow_mut() = Some(Rc::new(f));
    }

    /// Returns whether the tooltip is currently suppressed.
    pub fn is_suppressed(&self) -> bool {
        if let Some(ref f) = *self.suppress_fn.borrow() {
            f()
        } else {
            false
        }
    }

    /// Displays the tooltip popover.
    pub fn popup(&self) {
        self.popover.popup();
    }

    /// Dismisses the tooltip popover.
    pub fn popdown(&self) {
        self.popover.popdown();
    }

    /// Returns a reference to the underlying GTK4 Popover widget.
    pub fn widget(&self) -> &Popover {
        &self.popover
    }

    /// Builds a structured card widget with an optional title and key-value rows.
    pub fn build_card(title: &str, rows: &[TooltipRow]) -> Box {
        let card = Box::new(Orientation::Vertical, 6);
        card.add_css_class("status-popover-card");
        card.add_css_class("tooltip-popover-card");
        card.set_margin_top(0);
        card.set_margin_bottom(0);
        card.set_margin_start(0);
        card.set_margin_end(0);

        if !title.is_empty() {
            let title_lbl = Label::new(Some(title));
            title_lbl.add_css_class("status-popover-header");
            title_lbl.add_css_class("tooltip-popover-header");
            title_lbl.set_halign(gtk4::Align::Start);
            title_lbl.set_ellipsize(gtk4::pango::EllipsizeMode::End);
            card.append(&title_lbl);
        }

        for row in rows {
            let row_box = Box::new(Orientation::Horizontal, 16);
            row_box.add_css_class("status-popover-row");
            row_box.add_css_class("tooltip-popover-row");

            let key_lbl = Label::new(Some(&row.key));
            key_lbl.add_css_class("status-popover-key");
            key_lbl.add_css_class("tooltip-popover-key");
            key_lbl.set_halign(gtk4::Align::Start);

            if row.val.is_empty() {
                key_lbl.set_hexpand(true);
                row_box.append(&key_lbl);
            } else {
                key_lbl.set_hexpand(true);

                let val_lbl = Label::new(Some(&row.val));
                val_lbl.add_css_class("status-popover-val");
                val_lbl.add_css_class("tooltip-popover-val");
                if let Some(ref cls) = row.css_class {
                    val_lbl.add_css_class(cls);
                }
                val_lbl.set_halign(gtk4::Align::End);

                row_box.append(&key_lbl);
                row_box.append(&val_lbl);
            }

            card.append(&row_box);
        }

        card
    }

    /// Parses a multi-line string with "Key: Value" lines into `TooltipRow` items.
    pub fn parse_rows(text: &str) -> Vec<TooltipRow> {
        text.lines()
            .filter_map(|line| {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    return None;
                }
                if let Some((k, v)) = trimmed.split_once(':') {
                    Some(TooltipRow::new(k.trim(), v.trim(), None))
                } else {
                    Some(TooltipRow::new(trimmed, "", None))
                }
            })
            .collect()
    }

    /// Attaches hover motion controllers to the anchor widget and popover content.
    pub fn attach_hover(&self, anchor: &impl IsA<Widget>, update_fn: Option<Rc<dyn Fn()>>) {
        let is_hovered = Rc::new(RefCell::new(false));
        let suppress_fn = self.suppress_fn.clone();

        // 1. Motion controller on anchor widget
        let motion_anchor = gtk4::EventControllerMotion::new();
        let is_h_enter = is_hovered.clone();
        let pop_enter = self.popover.clone();
        let update_c = update_fn.clone();
        let sup_enter = suppress_fn.clone();
        motion_anchor.connect_enter(move |_, _, _| {
            if pop_enter.parent().is_none() || pop_enter.root().is_none() {
                return;
            }
            *is_h_enter.borrow_mut() = true;
            if let Some(ref f) = *sup_enter.borrow() {
                if f() {
                    *is_h_enter.borrow_mut() = false;
                    if pop_enter.is_visible() {
                        pop_enter.popdown();
                    }
                    return;
                }
            }
            if let Some(ref update) = update_c {
                update();
            }
            pop_enter.popup();
        });

        let is_h_motion = is_hovered.clone();
        let pop_motion = self.popover.clone();
        let sup_motion = suppress_fn.clone();
        motion_anchor.connect_motion(move |_, _, _| {
            if let Some(ref f) = *sup_motion.borrow() {
                if f() {
                    *is_h_motion.borrow_mut() = false;
                    if pop_motion.is_visible() {
                        pop_motion.popdown();
                    }
                }
            }
        });

        let is_h_leave = is_hovered.clone();
        let pop_leave = self.popover.clone();
        motion_anchor.connect_leave(move |_| {
            *is_h_leave.borrow_mut() = false;
            let is_h = is_h_leave.clone();
            let pop = pop_leave.clone();
            gtk4::glib::timeout_add_local(std::time::Duration::from_millis(150), move || {
                if !*is_h.borrow() && pop.parent().is_some() && pop.root().is_some() {
                    pop.popdown();
                }
                gtk4::glib::ControlFlow::Break
            });
        });
        anchor.add_controller(motion_anchor);

        // 2. Motion controller on popover card
        let motion_pop = gtk4::EventControllerMotion::new();
        let is_h_pop_enter = is_hovered.clone();
        let pop_pop = self.popover.clone();
        let sup_pop = suppress_fn.clone();
        motion_pop.connect_enter(move |_, _, _| {
            if let Some(ref f) = *sup_pop.borrow() {
                if f() {
                    *is_h_pop_enter.borrow_mut() = false;
                    if pop_pop.is_visible() {
                        pop_pop.popdown();
                    }
                    return;
                }
            }
            *is_h_pop_enter.borrow_mut() = true;
        });

        let pop_pop_motion = self.popover.clone();
        let sup_pop_motion = suppress_fn.clone();
        motion_pop.connect_motion(move |_, _, _| {
            if let Some(ref f) = *sup_pop_motion.borrow() {
                if f() && pop_pop_motion.is_visible() {
                    pop_pop_motion.popdown();
                }
            }
        });

        let is_h_pop_leave = is_hovered.clone();
        let pop_pop_leave = self.popover.clone();
        motion_pop.connect_leave(move |_| {
            *is_h_pop_leave.borrow_mut() = false;
            let is_h = is_h_pop_leave.clone();
            let pop = pop_pop_leave.clone();
            gtk4::glib::timeout_add_local(std::time::Duration::from_millis(150), move || {
                if !*is_h.borrow() && pop.parent().is_some() && pop.root().is_some() {
                    pop.popdown();
                }
                gtk4::glib::ControlFlow::Break
            });
        });
        self.popover.add_controller(motion_pop);
    }
}
