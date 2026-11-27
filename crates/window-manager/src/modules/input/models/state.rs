//! Input event context state shared between handlers.

use smithay::backend::input::KeyState;
use smithay::input::keyboard::ModifiersState;

use common::KeyboardAction;

/// Bundles the state and event data needed by the key-family handlers.
pub(crate) struct KeyContext<'a, C> {
    pub(crate) state: &'a mut C,
    pub(crate) key_state: KeyState,
    pub(crate) modifiers: &'a ModifiersState,
    pub(crate) sym: u32,
    pub(crate) action: &'a mut Option<KeyboardAction>,
    pub(crate) alt_tab_hold: &'a mut Option<i32>,
    pub(crate) switcher_active: bool,
}

/// Pure decision about the final seat dispatch of a pointer button event.
///
/// Compositor-owned interactions (titlebars, interactive move/resize, shell
/// UI) must not be forwarded to whatever client surface happens to have
/// pointer focus, otherwise e.g. a titlebar drag also starts a selection in a
/// window below it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) struct ButtonForwarding {
    consumed: bool,
}

impl ButtonForwarding {
    /// The compositor did not handle the event: forward it to the client.
    pub(crate) fn unhandled() -> Self {
        Self::default()
    }

    /// The compositor handled the event: suppress the client delivery.
    #[cfg(test)]
    pub(crate) fn consumed() -> Self {
        Self { consumed: true }
    }

    /// Marks the event as handled by the compositor.
    pub(crate) fn consume(&mut self) {
        self.consumed = true;
    }

    /// Whether the event should be sent to the pointer-focused client.
    pub(crate) fn should_forward(&self) -> bool {
        !self.consumed
    }
}

/// Pointer button event context.
pub(crate) struct ButtonContext<'a, C> {
    pub(crate) state: &'a mut C,
    pub(crate) serial: smithay::utils::Serial,
    pub(crate) button_code: u32,
    pub(crate) time: u32,
    forwarding: ButtonForwarding,
}

impl<'a, C> ButtonContext<'a, C> {
    /// Creates a context for an event the compositor has not consumed yet.
    pub(crate) fn new(
        state: &'a mut C,
        serial: smithay::utils::Serial,
        button_code: u32,
        time: u32,
    ) -> Self {
        Self {
            state,
            serial,
            button_code,
            time,
            forwarding: ButtonForwarding::unhandled(),
        }
    }

    /// Records that the compositor handled this button event itself.
    pub(crate) fn consume(&mut self) {
        self.forwarding.consume();
    }

    /// Whether this event still needs to reach a client surface.
    pub(crate) fn should_forward(&self) -> bool {
        self.forwarding.should_forward()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unhandled_events_are_forwarded() {
        let decision = ButtonForwarding::unhandled();
        assert!(decision.should_forward());
    }

    #[test]
    fn consumed_events_are_not_forwarded() {
        let decision = ButtonForwarding::consumed();
        assert!(!decision.should_forward());
    }

    #[test]
    fn consuming_an_event_is_idempotent() {
        let mut decision = ButtonForwarding::unhandled();
        decision.consume();
        assert!(!decision.should_forward());
        decision.consume();
        assert!(!decision.should_forward());
    }

    #[test]
    fn button_context_starts_unconsumed_and_can_be_consumed() {
        let mut state = ();
        let mut ctx = ButtonContext::new(&mut state, smithay::utils::Serial::from(7), 0x110, 42);
        assert_eq!(ctx.button_code, 0x110);
        assert!(ctx.should_forward());
        ctx.consume();
        assert!(!ctx.should_forward());
    }
}
