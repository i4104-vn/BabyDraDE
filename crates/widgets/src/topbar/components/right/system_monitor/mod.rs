//! System resource monitor widget for CPU load, RAM usage, and GPU metrics.

mod render;

use gtk4::glib;
use gtk4::prelude::*;
use infrastructure::system::{get_monitor_snapshot, SystemService};
use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

pub fn create_sys_monitor_w() -> gtk4::Box {
    let capsule = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);

    let (
        sys_label,
        popover,
        cpu_chart,
        ram_chart,
        gpu_chart,
        cpu_label,
        ram_label,
        ram_detail,
        gpu_label,
    ) = render::build_sys_monitor(&capsule);

    let cpu_history = Rc::new(RefCell::new(VecDeque::from(vec![0.0; 30])));
    let ram_history = Rc::new(RefCell::new(VecDeque::from(vec![0.0; 30])));
    let gpu_history = Rc::new(RefCell::new(VecDeque::from(vec![0.0; 30])));

    render::setup_chart_draw(&cpu_chart, cpu_history.clone(), "#3b82f6");
    render::setup_chart_draw(&ram_chart, ram_history.clone(), "#a855f7");
    render::setup_chart_draw(&gpu_chart, gpu_history.clone(), "#10b981");

    let cpu_history_loop = cpu_history.clone();
    let ram_history_loop = ram_history.clone();
    let gpu_history_loop = gpu_history.clone();
    let cpu_chart_loop = cpu_chart.clone();
    let ram_chart_loop = ram_chart.clone();
    let gpu_chart_loop = gpu_chart.clone();
    let sys_label_clone = sys_label.clone();
    let cpu_label_clone = cpu_label.clone();
    let ram_label_clone = ram_label.clone();
    let ram_detail_clone = ram_detail.clone();
    let gpu_label_clone = gpu_label.clone();

    let sys_service = Rc::new(RefCell::new(SystemService::new()));
    let sys_svc_timer = sys_service.clone();

    // Polling timer every 2 seconds
    glib::timeout_add_seconds_local(2, move || {
        let snapshot = get_monitor_snapshot(&mut sys_svc_timer.borrow_mut());

        sys_label_clone.set_text(&format!(
            "CPU: {:.0}%  RAM: {:.0}%",
            snapshot.cpu_percent, snapshot.ram_percent
        ));

        cpu_label_clone.set_text(&format!("CPU Usage: {:.1}%", snapshot.cpu_percent));
        ram_label_clone.set_text(&format!("RAM Usage: {:.1}%", snapshot.ram_percent));
        ram_detail_clone.set_text(&format!(
            "{:.2} GB / {:.2} GB",
            snapshot.ram_used_gb, snapshot.ram_total_gb
        ));
        gpu_label_clone.set_text(&format!("GPU Usage: {:.1}%", snapshot.gpu_percent));

        for (hist_rc, chart, val) in [
            (&cpu_history_loop, &cpu_chart_loop, snapshot.cpu_percent),
            (&ram_history_loop, &ram_chart_loop, snapshot.ram_percent),
            (&gpu_history_loop, &gpu_chart_loop, snapshot.gpu_percent),
        ] {
            let mut hist = hist_rc.borrow_mut();
            hist.pop_front();
            hist.push_back(val);
            chart.queue_draw();
        }

        glib::ControlFlow::Continue
    });

    let cpu_chart_hover = cpu_chart.clone();
    let ram_chart_hover = ram_chart.clone();
    let gpu_chart_hover = gpu_chart.clone();
    let popover_enter = popover.clone();

    let motion_controller = gtk4::EventControllerMotion::new();
    motion_controller.connect_enter(move |_, _, _| {
        cpu_chart_hover.queue_draw();
        ram_chart_hover.queue_draw();
        gpu_chart_hover.queue_draw();
        popover_enter.popup();
    });

    let popover_leave = popover.clone();
    motion_controller.connect_leave(move |_| {
        popover_leave.popdown();
    });

    capsule.add_controller(motion_controller);
    capsule
}

