pub mod clock;
pub mod control_center;
pub mod status;
pub mod system_monitor;
pub mod tray;

use gtk4::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

pub fn create_right_section(window: &gtk4::ApplicationWindow) -> gtk4::Box {
    let container = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(12)
        .hexpand(true)
        .halign(gtk4::Align::End)
        .valign(gtk4::Align::Center)
        .build();

    let app = window
        .application()
        .expect("Window must have an application");
    let cc_slot = Rc::new(RefCell::new(None));
    let cal_slot = Rc::new(RefCell::new(None));

    let tray = tray::create_tray_widget(window);
    let sys_mon = system_monitor::create_sys_monitor_w();
    let status = status::create_status_icons(&app, cc_slot, cal_slot);

    container.append(&tray);
    container.append(&sys_mon);
    container.append(&status);

    container
}
