//! Fluent builder for declaratively constructing and presenting context menus.

use super::items::*;
use gtk4::prelude::*;
use gtk4::{Box, Button, Popover, PositionType};
use std::cell::RefCell;
use std::rc::Rc;

thread_local! {
    /// Every top-level context menu is registered here so shells can dismiss
    /// all of them on a single outside click (layer-shell panels never see
    /// clicks that land on other surfaces).
    static OPEN_MENUS: RefCell<Vec<Popover>> = const { RefCell::new(Vec::new()) };
    /// Shells register a hook here to show/hide their outside-click catcher.
    static VISIBILITY_HOOK: RefCell<Option<std::boxed::Box<dyn Fn(bool)>>> =
        const { RefCell::new(None) };
}

/// Lets the shell show/hide a full-screen click catcher whenever any context
/// menu opens or closes (layer-shell panels cannot see clicks on other surfaces).
pub fn set_visibility_hook(hook: impl Fn(bool) + 'static) {
    VISIBILITY_HOOK.with(|slot| *slot.borrow_mut() = Some(std::boxed::Box::new(hook)));
}

fn notify_visibility(open: bool) {
    VISIBILITY_HOOK.with(|slot| {
        if let Some(hook) = slot.borrow().as_ref() {
            hook(open);
        }
    });
}

/// Closes every open context menu across the application.
pub fn dismiss_all() {
    let popovers = OPEN_MENUS.with(|menus| menus.borrow().clone());
    for popover in popovers {
        if popover.is_visible() {
            popover.popdown();
        }
    }
}

/// Adds the shared hover highlight to a menu row. The CSS class works even
/// when a system theme overrides the `:hover` pseudo-class.
fn wire_row_hover(btn: &Button) {
    let motion = gtk4::EventControllerMotion::new();
    let enter = btn.downgrade();
    let leave = btn.downgrade();
    motion.connect_enter(move |_, _, _| {
        if let Some(btn) = enter.upgrade() {
            btn.add_css_class("context-menu-row-hover");
        }
    });
    motion.connect_leave(move |_| {
        if let Some(btn) = leave.upgrade() {
            btn.remove_css_class("context-menu-row-hover");
        }
    });
    btn.add_controller(motion);
}

/// A Fluent Context Menu overlay component.
pub struct ContextMenu {
    popover: Popover,
    vbox: Box,
    footer_box: Option<Box>,
    active_subpopover: Rc<RefCell<Option<Popover>>>,
    ancestor_popovers: Vec<Popover>,
}

pub type ContextMenuBuilder = ContextMenu;

mod construct;
mod items;
mod widget;
