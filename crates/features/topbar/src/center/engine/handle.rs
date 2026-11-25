//! Per-view state and the public display handle.

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

/// Monotonic counter used to break priority ties: latest request wins.
static REQUEST_SEQ: AtomicU64 = AtomicU64::new(0);

#[derive(Default)]
pub(crate) struct ViewState {
    pub(crate) requested: Cell<bool>,
    pub(crate) active: Cell<bool>,
    pub(crate) override_active: Cell<bool>,
    pub(crate) auto_hide_at: RefCell<Option<Instant>>,
    pub(crate) release_at: RefCell<Option<Instant>>,
    pub(crate) request_seq: Cell<u64>,
}

impl ViewState {
    /// Expires deadlines and reports whether any active flag remains.
    pub(crate) fn purge(&self, now: Instant) -> bool {
        if self
            .auto_hide_at
            .borrow()
            .is_some_and(|deadline| now >= deadline)
        {
            self.requested.set(false);
            self.auto_hide_at.borrow_mut().take();
        }
        if self
            .release_at
            .borrow()
            .is_some_and(|deadline| now >= deadline)
        {
            self.override_active.set(false);
            self.release_at.borrow_mut().take();
        }
        self.requested.get() || self.override_active.get()
    }

    pub(crate) fn has_timeout(&self, now: Instant) -> bool {
        self.auto_hide_at.borrow().is_some_and(|d| now < d)
            || self.release_at.borrow().is_some_and(|d| now < d)
    }

    pub(crate) fn deactivate(&self) {
        self.requested.set(false);
        self.override_active.set(false);
        self.auto_hide_at.borrow_mut().take();
        self.release_at.borrow_mut().take();
    }
}

/// Handle given to a feature through its context to request display.
#[derive(Clone)]
pub struct IslandViewHandle {
    pub(crate) state: Rc<ViewState>,
}

impl IslandViewHandle {
    pub fn show(&self) {
        if !self.state.requested.get() {
            self.state
                .request_seq
                .set(REQUEST_SEQ.fetch_add(1, Ordering::Relaxed));
            self.state.requested.set(true);
            self.state.auto_hide_at.borrow_mut().take();
        }
    }

    pub fn show_for(&self, duration: Duration) {
        self.show();
        self.state
            .auto_hide_at
            .replace(Some(Instant::now() + duration));
    }

    pub fn hide(&self) {
        self.state.deactivate();
    }

    pub fn override_show(&self) {
        self.show();
        self.state
            .request_seq
            .set(REQUEST_SEQ.fetch_add(1, Ordering::Relaxed));
        self.state.override_active.set(true);
        self.state.release_at.borrow_mut().take();
        self.state.auto_hide_at.borrow_mut().take();
    }

    pub fn override_show_for(&self, duration: Duration) {
        self.override_show();
        let deadline = Instant::now() + duration;
        self.state.release_at.replace(Some(deadline));
        self.state.auto_hide_at.replace(Some(deadline));
    }

    pub fn release_override(&self) {
        self.state.override_active.set(false);
        self.state.release_at.borrow_mut().take();
        if self.state.auto_hide_at.borrow().is_none() {
            self.state.requested.set(false);
        }
    }
}
