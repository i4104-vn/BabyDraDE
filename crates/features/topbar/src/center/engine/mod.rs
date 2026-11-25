//! Island engine: widget tree, lifecycle and arbitration loop.
//!
//! Split into focused modules:
//! - [`feature`]: the single `IslandFeature` trait + capsule dimensions
//! - [`handle`]: per-view state and `IslandViewHandle`
//! - [`context`]: `IslandCtx`, the only context features receive
//! - [`record`]: internal registration record
//! - [`arbitration`]: winner selection rules

mod arbitration;
mod builder;
mod context;
mod feature;
mod handle;
mod record;

pub use context::IslandCtx;
pub use feature::{IslandFeature, CAPSULE_HEIGHT, CAPSULE_WIDTH, PLAYER_CAPSULE_WIDTH};
pub use handle::IslandViewHandle;

use std::cell::{Cell, RefCell};
use std::rc::{Rc, Weak};
use std::time::Instant;

use gtk4::prelude::*;

use handle::ViewState;
use record::ViewRecord;

/// Owns the island widget tree and drives arbitration.
pub struct IslandEngine {
    pub(crate) root: gtk4::Box,
    pub(crate) capsule: gtk4::Box,
    content: gtk4::Box,
    left_bracket: gtk4::Label,
    right_bracket: gtk4::Label,
    self_weak: Weak<RefCell<IslandEngine>>,
    pub(crate) views: Vec<ViewRecord>,
    pub(crate) displayed: Option<usize>,
    pub(crate) hovered: Cell<bool>,
    pub(crate) animating: Cell<bool>,
}

impl IslandEngine {
    pub fn new() -> Self {
        let shell = builder::build();
        Self {
            root: shell.root,
            capsule: shell.capsule,
            content: shell.content,
            left_bracket: shell.left_bracket,
            right_bracket: shell.right_bracket,
            self_weak: Weak::new(),
            views: Vec::new(),
            displayed: None,
            hovered: shell.hovered,
            animating: Cell::new(false),
        }
    }

    /// Called once by the capsule so contexts can request immediate ticks.
    pub(crate) fn set_self_weak(&mut self, weak: Weak<RefCell<IslandEngine>>) {
        self.self_weak = weak;
    }

    pub fn root(&self) -> gtk4::Box {
        self.root.clone()
    }

    pub fn capsule(&self) -> gtk4::Box {
        self.capsule.clone()
    }

    /// Registers a feature, builds its view through the shared context and
    /// returns its handle.
    pub fn register_feature<F: IslandFeature + 'static>(
        &mut self,
        mut feature: F,
    ) -> IslandViewHandle {
        let state = Rc::new(ViewState::default());
        let handle = IslandViewHandle {
            state: state.clone(),
        };
        let ctx = IslandCtx::new(
            self.capsule.clone(),
            handle.clone(),
            self.self_weak.clone(),
            feature.id(),
            false,
            false,
        );
        let widget = feature.build(&ctx);

        let container = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .visible(false)
            .build();
        container.append(&widget);
        self.content.append(&container);
        self.views.push(ViewRecord {
            feature: Rc::new(RefCell::new(feature)),
            container,
            state,
        });
        handle
    }

    /// Dispatches a capsule click to the displayed feature.
    pub fn handle_click(&mut self) {
        if let Some(index) = self.displayed {
            if let Some(view) = self.views.get_mut(index) {
                view.feature.borrow_mut().on_click();
            }
        }
    }

    /// One arbitration tick (the capsule calls this every 150 ms).
    pub fn tick(&mut self, engine_rc: &Rc<RefCell<IslandEngine>>) {
        let now = Instant::now();
        let hovered = self.hovered.get();
        let alive_count = arbitration::refresh_requests(&self.views, now, hovered);

        for (index, view) in self.views.iter().enumerate() {
            let ctx = IslandCtx::new(
                self.capsule.clone(),
                IslandViewHandle {
                    state: view.state.clone(),
                },
                self.self_weak.clone(),
                view.feature.borrow().id(),
                self.displayed == Some(index),
                hovered,
            );
            if let Ok(mut feature) = view.feature.try_borrow_mut() {
                feature.tick(&ctx);
            }
        }

        let show_brackets = alive_count > 1;
        if self.left_bracket.is_visible() != show_brackets {
            self.left_bracket.set_visible(show_brackets);
            self.right_bracket.set_visible(show_brackets);
        }

        if !self.animating.get() {
            if let Some(index) = self.displayed {
                let (width, height) = self.views[index].feature.borrow().size();
                self.capsule.set_size_request(width, height);
            }
        }

        let winner = arbitration::select_winner(&self.views, now);
        if winner == self.displayed {
            return;
        }
        self.animating.set(false);
        super::ui::transition::apply_transition(self, winner, engine_rc);
    }
}

impl Default for IslandEngine {
    fn default() -> Self {
        Self::new()
    }
}
