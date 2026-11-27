//! Context menu component state model.

use super::*;

thread_local! {
    /// Every top-level context menu is registered here so shells can dismiss
    /// all of them on a single outside click (layer-shell panels never see
    /// clicks that land on other surfaces).
    pub(super) static OPEN_MENUS: RefCell<Vec<Popover>> = const { RefCell::new(Vec::new()) };
    /// Shells register a hook here to show/hide their outside-click catcher.
    pub(super) static VISIBILITY_HOOK: RefCell<Option<Rc<dyn Fn(bool)>>> =
        const { RefCell::new(None) };
}

/// Lets the shell show/hide a full-screen click catcher whenever any context
/// menu opens or closes (layer-shell panels cannot see clicks on other surfaces).
pub fn set_visibility_hook(hook: impl Fn(bool) + 'static) {
    VISIBILITY_HOOK.with(|slot| *slot.borrow_mut() = Some(Rc::new(hook)));
}

pub(super) fn notify_visibility(open: bool) {
    // Clone the hook out first: a hook may re-enter and change the registry
    // (for example by closing a menu), which would panic on a held borrow.
    let hook = VISIBILITY_HOOK.with(|slot| slot.borrow().clone());
    if let Some(hook) = hook {
        hook(open);
    }
}

/// Closes every open context menu across the application.
pub fn dismiss_all() {
    let popovers = OPEN_MENUS.with(|menus| {
        let mut menus = menus.borrow_mut();
        // Drop registrations for popovers that were never opened or are gone.
        menus.retain(|menu| menu.is_visible());
        menus.clone()
    });
    for popover in popovers {
        if popover.is_visible() {
            popover.popdown();
        }
    }
}

/// Returns whether any registered context menu is currently visible.
///
/// Read-only and side-effect free: consumers (for example an expanded tray)
/// can keep their own surface open while a menu is active. This never pops
/// menus down or mutates the registry.
pub fn any_open() -> bool {
    OPEN_MENUS.with(|menus| menus.borrow().iter().any(|menu| menu.is_visible()))
}

/// Adds the shared hover highlight to a menu row. The CSS class works even
/// when a system theme overrides the `:hover` pseudo-class.
pub(super) fn wire_row_hover(btn: &Button) {
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
    pub(super) popover: Popover,
    pub(super) vbox: Box,
    pub(super) footer_box: Option<Box>,
    pub(super) active_subpopover: Rc<RefCell<Option<Popover>>>,
    pub(super) ancestor_popovers: Vec<Popover>,
    /// Pointer coordinates (parent-widget space) for menus opened at `(x, y)`
    /// instead of at a whole widget. `popup()` fits around this point when set.
    pub(super) pointer_point: Option<(f64, f64)>,
}

pub type ContextMenuBuilder = ContextMenu;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn any_open_reads_registry_without_mutating_it() {
        let before = OPEN_MENUS.with(|menus| menus.borrow().len());
        let expected = OPEN_MENUS.with(|menus| menus.borrow().iter().any(|m| m.is_visible()));
        assert_eq!(any_open(), expected);
        assert_eq!(any_open(), expected);
        assert_eq!(OPEN_MENUS.with(|menus| menus.borrow().len()), before);
    }
}
