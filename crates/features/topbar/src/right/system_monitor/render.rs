use gtk4::prelude::*;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;
use ui_kit::drawing::draw_metric_chart;

pub fn build_sys_monitor(
    capsule: &gtk4::Box,
) -> (
    gtk4::Label,
    gtk4::Popover,
    gtk4::DrawingArea,
    gtk4::DrawingArea,
    gtk4::DrawingArea,
    gtk4::Label,
    gtk4::Label,
    gtk4::Label,
    gtk4::Label,
) {
    capsule.add_css_class("sys-monitor-box");
    capsule.set_valign(gtk4::Align::Center);

    let sys_label = gtk4::Label::new(Some("CPU: 0%  RAM: 0%"));
    sys_label.add_css_class("status-text");
    capsule.append(&sys_label);

    let popover = gtk4::Popover::builder()
        .position(gtk4::PositionType::Bottom)
        .has_arrow(false)
        .autohide(false)
        .css_classes(["sys-monitor-popover", "status-popover"])
        .build();
    popover.set_parent(capsule);

    let popover_box = gtk4::Box::new(gtk4::Orientation::Vertical, 10);
    popover_box.set_size_request(230, -1);

    let popover_title = gtk4::Label::new(Some("System Resources"));
    popover_title.add_css_class("tile-title");
    popover_title.set_xalign(0.0);
    popover_title.set_margin_bottom(4);

    let cpu_label = gtk4::Label::new(Some("CPU Usage: 0.0%"));
    cpu_label.set_xalign(0.0);
    cpu_label.add_css_class("tile-subtitle");

    let cpu_chart = gtk4::DrawingArea::new();
    cpu_chart.set_size_request(230, 52);
    cpu_chart.add_css_class("sys-chart");

    let ram_header = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
    let ram_label = gtk4::Label::new(Some("RAM Usage: 0.0%"));
    ram_label.set_xalign(0.0);
    ram_label.set_hexpand(true);
    ram_label.set_halign(gtk4::Align::Start);
    ram_label.add_css_class("tile-subtitle");

    let ram_detail = gtk4::Label::new(Some("0.0 GB / 0.0 GB"));
    ram_detail.set_xalign(1.0);
    ram_detail.set_halign(gtk4::Align::End);
    ram_detail.add_css_class("control-square-label");
    ram_detail.set_opacity(0.7);

    ram_header.append(&ram_label);
    ram_header.append(&ram_detail);

    let ram_chart = gtk4::DrawingArea::new();
    ram_chart.set_size_request(230, 52);
    ram_chart.add_css_class("sys-chart");

    let gpu_label = gtk4::Label::new(Some("GPU Usage: 0.0%"));
    gpu_label.set_xalign(0.0);
    gpu_label.add_css_class("tile-subtitle");

    let gpu_chart = gtk4::DrawingArea::new();
    gpu_chart.set_size_request(230, 52);
    gpu_chart.add_css_class("sys-chart");

    popover_box.append(&popover_title);
    popover_box.append(&cpu_label);
    popover_box.append(&cpu_chart);
    popover_box.append(&ram_header);
    popover_box.append(&ram_chart);
    popover_box.append(&gpu_label);
    popover_box.append(&gpu_chart);

    popover.set_child(Some(&popover_box));

    (
        sys_label, popover, cpu_chart, ram_chart, gpu_chart, cpu_label, ram_label, ram_detail,
        gpu_label,
    )
}

pub fn setup_chart_draw(
    chart: &gtk4::DrawingArea,
    history: Rc<RefCell<VecDeque<f64>>>,
    color_hex: &'static str,
) {
    chart.set_draw_func(move |_, cr, width, height| {
        let history = history.borrow();
        let slice: Vec<f64> = history.iter().copied().collect();
        draw_metric_chart(cr, width as f64, height as f64, &slice, color_hex);
    });
}
