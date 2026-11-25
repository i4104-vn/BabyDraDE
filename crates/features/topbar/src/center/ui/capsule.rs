//! Dynamic Island capsule: 150 ms arbitration engine + live features.

use gtk4::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use crate::center::engine::IslandEngine;

pub struct IslandCapsule {
    root: gtk4::Box,
    #[allow(dead_code)]
    engine: Rc<RefCell<IslandEngine>>,
}

impl IslandCapsule {
    pub fn new() -> Self {
        let engine = Rc::new(RefCell::new(IslandEngine::new()));
        let weak = Rc::downgrade(&engine);
        engine.borrow_mut().set_self_weak(weak);

        crate::center::ipc::init();

        let capsule = engine.borrow().capsule();
        crate::center::services::shortcuts::init();
        let click_engine = engine.clone();
        let click = gtk4::GestureClick::new();
        click.connect_pressed(move |_, _, _, _| {
            click_engine.borrow_mut().handle_click();
        });
        capsule.add_controller(click);

        {
            let mut engine = engine.borrow_mut();
            crate::center::registry::register_all(&mut engine);
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

