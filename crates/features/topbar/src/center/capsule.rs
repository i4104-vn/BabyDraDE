//! Dynamic Island capsule: real 150 ms arbitration engine + live features.

use gtk4::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use super::engine::IslandEngine;
use super::features::{MediaFeature, RecordingFeature};

pub struct IslandCapsule {
    root: gtk4::Box,
    #[allow(dead_code)]
    engine: Rc<RefCell<IslandEngine>>,
}

impl IslandCapsule {
    pub fn new() -> Self {
        let engine = Rc::new(RefCell::new(IslandEngine::new()));

        let capsule = engine.borrow().capsule();
        let click_engine = engine.clone();
        let click = gtk4::GestureClick::new();
        click.connect_pressed(move |_, _, _, _| {
            click_engine.borrow_mut().handle_click();
        });
        capsule.add_controller(click);

        {
            let mut e = engine.borrow_mut();
            // Recording is intentionally dormant until a recorder service exists.
            e.register_feature(RecordingFeature::new());
            // Media player polls real MPRIS metadata via `playerctl`.
            e.register_feature(MediaFeature::new());
        }

        let tick_engine = engine.clone();
        glib::timeout_add_local(Duration::from_millis(150), move || {
            let rc = tick_engine.clone();
            rc.borrow_mut().tick(&rc);
            glib::ControlFlow::Continue
        });

        let root = engine.borrow().root();
        Self { root, engine }
    }

    pub fn widget(&self) -> &gtk4::Box {
        &self.root
    }
}

impl Default for IslandCapsule {
    fn default() -> Self {
        Self::new()
    }
}
