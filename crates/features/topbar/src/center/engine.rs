//! Island arbitration engine: handles, deadlines and priority-based view switching.
//!
//! Ported from the `babydra-island` framework: every feature reports whether it
//! is alive, the 150 ms tick selects a winner (open popover > override >
//! priority), and only the winning view is attached to the capsule.

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use gtk4::prelude::*;

static REQUEST_SEQ: AtomicU64 = AtomicU64::new(0);

/// Capsule size used by regular features.
pub const CAPSULE_WIDTH: i32 = 180;
/// Expanded capsule size used by the media player.
pub const PLAYER_CAPSULE_WIDTH: i32 = 200;
pub const CAPSULE_HEIGHT: i32 = 30;

#[derive(Default)]
pub(crate) struct ViewState {
    requested: Cell<bool>,
    pub(crate) active: Cell<bool>,
    override_active: Cell<bool>,
    auto_hide_at: RefCell<Option<Instant>>,
    release_at: RefCell<Option<Instant>>,
    request_seq: Cell<u64>,
}

impl ViewState {
    fn purge(&self, now: Instant) -> bool {
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

    fn has_timeout(&self, now: Instant) -> bool {
        self.auto_hide_at.borrow().is_some_and(|d| now < d)
            || self.release_at.borrow().is_some_and(|d| now < d)
    }

    fn deactivate(&self) {
        self.requested.set(false);
        self.override_active.set(false);
        self.auto_hide_at.borrow_mut().take();
        self.release_at.borrow_mut().take();
    }
}

/// Handle given to features to request display of their view.
#[derive(Clone)]
pub struct IslandViewHandle {
    state: Rc<ViewState>,
}

impl IslandViewHandle {
    pub fn show(&self) {
        if !self.state.requested.get() {
            self.state.request_seq.set(REQUEST_SEQ.fetch_add(1, Ordering::Relaxed));
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
        self.state.request_seq.set(REQUEST_SEQ.fetch_add(1, Ordering::Relaxed));
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

    pub fn is_requested(&self) -> bool {
        self.state.requested.get()
    }
}

/// Context passed to feature ticks.
#[derive(Clone)]
pub struct IslandCtx {
    capsule: gtk4::Box,
    current: bool,
    hovered: bool,
}

impl IslandCtx {
    pub fn capsule(&self) -> gtk4::Box {
        self.capsule.clone()
    }
    pub fn is_current(&self) -> bool {
        self.current
    }
    pub fn is_hovered(&self) -> bool {
        self.hovered
    }
}

/// A pluggable feature of the Dynamic Island.
pub trait IslandFeature {
    fn id(&self) -> &'static str;

    /// Higher priority wins arbitration.
    fn priority(&self) -> u32 {
        50
    }

    fn size(&self) -> (i32, i32) {
        (CAPSULE_WIDTH, CAPSULE_HEIGHT)
    }

    fn hover_keep(&self) -> bool {
        false
    }

    fn capsule_class(&self) -> Option<&'static str> {
        None
    }

    /// Persistent feature state (media playing, recording, ...).
    fn is_alive(&self) -> bool {
        false
    }

    /// Legacy active-state hook (kept for simple features).
    fn is_active(&self) -> bool {
        self.is_alive()
    }

    /// Legacy details popover hook.
    fn details_popover_content(&self) -> Option<gtk4::Widget> {
        None
    }

    /// A feature with an open popover locks the capsule until it closes.
    fn is_popover_open(&self) -> bool {
        false
    }

    /// The widget displayed inside the capsule.
    fn widget(&self) -> gtk4::Widget;

    fn init(&mut self, _handle: IslandViewHandle) {}
    fn attach(&mut self, _capsule: &gtk4::Box) {}
    fn tick(&mut self, _ctx: &IslandCtx) {}
    fn on_show(&mut self) {}
    fn on_hide(&mut self) {}
    fn on_click(&mut self) {}
}

pub(crate) struct ViewRecord {
    pub(crate) feature: Rc<RefCell<dyn IslandFeature>>,
    pub(crate) container: gtk4::Box,
    pub(crate) state: Rc<ViewState>,
}

/// Owns the island widget tree and drives arbitration.
pub struct IslandEngine {
    pub(crate) root: gtk4::Box,
    pub(crate) capsule: gtk4::Box,
    content: gtk4::Box,
    left_bracket: gtk4::Label,
    right_bracket: gtk4::Label,
    pub(crate) views: Vec<ViewRecord>,
    pub(crate) displayed: Option<usize>,
    pub(crate) hovered: Cell<bool>,
    pub(crate) animating: Cell<bool>,
}

impl IslandEngine {
    pub fn new() -> Self {
        let root = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .valign(gtk4::Align::Center)
            .halign(gtk4::Align::Center)
            .css_classes(["island-wrapper"])
            .build();

        let left_bracket = gtk4::Label::builder()
            .label("(")
            .css_classes(["island-bracket", "left"])
            .visible(false)
            .build();

        let capsule = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .valign(gtk4::Align::Center)
            .halign(gtk4::Align::Center)
            .css_classes(["panel-notch"])
            .visible(false)
            .build();
        if let Some(cursor) = gtk4::gdk::Cursor::from_name("pointer", None) {
            capsule.set_cursor(Some(&cursor));
        }

        let content = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .valign(gtk4::Align::Center)
            .halign(gtk4::Align::Center)
            .css_classes(["notch-content"])
            .build();
        capsule.append(&content);

        let right_bracket = gtk4::Label::builder()
            .label(")")
            .css_classes(["island-bracket", "right"])
            .visible(false)
            .build();

        root.append(&left_bracket);
        root.append(&capsule);
        root.append(&right_bracket);

        let hover = Cell::new(false);
        let enter = gtk4::EventControllerMotion::new();
        let hover_enter = hover.clone();
        enter.connect_enter(move |_, _, _| hover_enter.set(true));
        let leave = gtk4::EventControllerMotion::new();
        let hover_leave = hover.clone();
        leave.connect_leave(move |_| hover_leave.set(false));
        capsule.add_controller(enter);
        capsule.add_controller(leave);

        Self {
            root,
            capsule,
            content,
            left_bracket,
            right_bracket,
            views: Vec::new(),
            displayed: None,
            hovered: hover,
            animating: Cell::new(false),
        }
    }

    pub fn root(&self) -> gtk4::Box {
        self.root.clone()
    }

    pub fn capsule(&self) -> gtk4::Box {
        self.capsule.clone()
    }

    /// Registers a feature and returns its handle.
    pub fn register_feature<F: IslandFeature + 'static>(
        &mut self,
        mut feature: F,
    ) -> IslandViewHandle {
        let widget = feature.widget();
        let container = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .visible(false)
            .build();
        container.append(&widget);
        self.content.append(&container);

        let state = Rc::new(ViewState::default());
        let handle = IslandViewHandle {
            state: state.clone(),
        };
        feature.init(handle.clone());
        feature.attach(&self.capsule);

        self.views.push(ViewRecord {
            feature: Rc::new(RefCell::new(feature)),
            container,
            state,
        });
        handle
    }

    /// Dispatches a click on the capsule to the currently displayed feature.
    pub fn handle_click(&mut self) {
        if let Some(idx) = self.displayed {
            if let Some(view) = self.views.get_mut(idx) {
                view.feature.borrow_mut().on_click();
            }
        }
    }

    /// One arbitration tick. Called every 150 ms by the capsule.
    pub fn tick(&mut self, engine_rc: &Rc<RefCell<IslandEngine>>) {
        let now = Instant::now();
        let hovered = self.hovered.get();

        let mut alive_count = 0usize;
        for view in &self.views {
            let alive = {
                let feature = view.feature.borrow();
                feature.is_alive()
                    || feature.is_active()
                    || feature.is_popover_open()
                    || (hovered && feature.hover_keep() && view.state.requested.get())
            };
            if !view.state.purge(now) && alive && !view.state.has_timeout(now) && !view.state.override_active.get() {
                view.state.deactivate();
            }
            if alive {
                view.state.requested.set(true);
                alive_count += 1;
            }
        }

        // Feature ticks (all features, every tick).
        for (idx, view) in self.views.iter().enumerate() {
            let ctx = IslandCtx {
                capsule: self.capsule.clone(),
                current: self.displayed == Some(idx),
                hovered,
            };
            if let Ok(mut feature) = view.feature.try_borrow_mut() {
                feature.tick(&ctx);
            }
        }

        let show_brackets = alive_count > 1;
        if self.left_bracket.is_visible() != show_brackets {
            self.left_bracket.set_visible(show_brackets);
            self.right_bracket.set_visible(show_brackets);
        }

        let winner = self.select_winner(now);
        if winner == self.displayed {
            return;
        }

        // Cancel a pending transition if the target changed mid-animation.
        if winner != self.displayed {
            self.animating.set(false);
        }
        super::ui::transition::apply_transition(self, winner, engine_rc);
    }

    fn select_winner(&self, now: Instant) -> Option<usize> {
        // 1. An open popover must never be hijacked.
        for (idx, view) in self.views.iter().enumerate() {
            if view.feature.borrow().is_popover_open() {
                return Some(idx);
            }
        }

        // 2. Most recent explicit override.
        let mut best_override: Option<(u64, usize)> = None;
        for (idx, view) in self.views.iter().enumerate() {
            if view.state.override_active.get() && view.state.purge(now) {
                let seq = view.state.request_seq.get();
                if best_override.is_none_or(|(s, _)| seq > s) {
                    best_override = Some((seq, idx));
                }
            }
        }
        if let Some((_, idx)) = best_override {
            return Some(idx);
        }

        // 3. Highest priority, tie-broken by most recent request.
        let mut best: Option<(u32, u64, usize)> = None;
        for (idx, view) in self.views.iter().enumerate() {
            if !view.state.purge(now) || !view.state.requested.get() {
                continue;
            }
            let key = (
                view.feature.borrow().priority(),
                view.state.request_seq.get(),
                idx,
            );
            if best.is_none_or(|b| key > b) {
                best = Some(key);
            }
        }
        best.map(|(_, _, idx)| idx)
    }
}

impl Default for IslandEngine {
    fn default() -> Self {
        Self::new()
    }
}
