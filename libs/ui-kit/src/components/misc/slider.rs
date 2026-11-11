//! PillSlider component — Smooth, pill-shaped GTK4 Scale slider with debouncing and mouse scroll.

use gtk4::glib;
use gtk4::prelude::*;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

/// Creates a standardized slider header row with title on left and percentage on right.
pub fn create_slider_header(title: &str, initial_val: f64) -> (gtk4::Box, gtk4::Label) {
    let header_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
    header_box.set_hexpand(true);

    let title_label = gtk4::Label::new(Some(title));
    title_label.add_css_class("control-slider-title");
    title_label.set_xalign(0.0);
    title_label.set_hexpand(true);

    let val_label = gtk4::Label::new(Some(&format!("{:.0}%", initial_val)));
    val_label.add_css_class("control-slider-val");
    val_label.set_xalign(1.0);

    header_box.append(&title_label);
    header_box.append(&val_label);

    (header_box, val_label)
}

pub type SliderCallbacks = Rc<RefCell<Vec<Rc<dyn Fn(f64)>>>>;

/// A smooth, rounded pill slider for volume and brightness control.
#[derive(Clone)]
pub struct PillSlider {
    pub container: gtk4::Box,
    pub scale: gtk4::Scale,
    silent: Rc<Cell<bool>>,
    change_callbacks: SliderCallbacks,
}

impl PillSlider {
    /// Creates a standard 0-100% pill slider.
    pub fn new(initial_val: f64, on_change: impl Fn(f64) + 'static) -> Self {
        Self::new_range(0.0, 100.0, 1.0, initial_val, on_change)
    }

    /// Creates a pill slider with a custom range and step.
    pub fn new_range(
        min: f64,
        max: f64,
        step: f64,
        initial_val: f64,
        on_change: impl Fn(f64) + 'static,
    ) -> Self {
        let container = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        container.add_css_class("pill-slider-container");
        container.set_hexpand(true);

        let scale = gtk4::Scale::with_range(gtk4::Orientation::Horizontal, min, max, step);
        scale.add_css_class("pill-slider");
        scale.set_hexpand(true);
        scale.set_draw_value(false);
        scale.set_value(initial_val);
        scale.set_cursor_from_name(Some("pointer"));

        container.append(&scale);

        let silent = Rc::new(Cell::new(false));
        let change_callbacks: SliderCallbacks = Rc::new(RefCell::new(vec![Rc::new(on_change)]));

        let silent_c = silent.clone();
        let callbacks_c = change_callbacks.clone();
        scale.connect_value_changed(move |s| {
            if silent_c.get() {
                return;
            }
            let val = s.value();
            for cb in callbacks_c.borrow().iter() {
                cb(val);
            }
        });

        Self {
            container,
            scale,
            silent,
            change_callbacks,
        }
    }

    /// Binds a label to show `{:.0}%` whenever the slider value changes.
    pub fn bind_label(&self, label: &gtk4::Label) {
        let lbl = label.clone();
        self.change_callbacks.borrow_mut().push(Rc::new(move |val| {
            lbl.set_text(&format!("{:.0}%", val));
        }));
    }

    /// Connects a debounced callback that fires `ms` milliseconds after the user stops moving the slider.
    pub fn connect_debounced(&self, ms: u32, callback: impl Fn(f64) + 'static) {
        let callback_rc = Rc::new(callback);
        let pending_source: Rc<RefCell<Option<glib::SourceId>>> = Rc::new(RefCell::new(None));

        let scale_c = self.scale.clone();
        let silent_c = self.silent.clone();
        let source_slot = pending_source.clone();

        self.scale.connect_value_changed(move |_| {
            if silent_c.get() {
                return;
            }
            if let Some(src) = source_slot.borrow_mut().take() {
                src.remove();
            }

            let sc = scale_c.clone();
            let cb = callback_rc.clone();
            let slot = source_slot.clone();

            let id =
                glib::timeout_add_local(std::time::Duration::from_millis(ms as u64), move || {
                    let val = sc.value();
                    cb(val);
                    *slot.borrow_mut() = None;
                    glib::ControlFlow::Break
                });

            *source_slot.borrow_mut() = Some(id);
        });
    }

    /// Adds a mouse scroll controller to any widget that steps the slider value by +/- 5%.
    pub fn add_scroll_to(&self, widget: &impl IsA<gtk4::Widget>) {
        let scroll_controller = gtk4::EventControllerScroll::new(
            gtk4::EventControllerScrollFlags::VERTICAL | gtk4::EventControllerScrollFlags::DISCRETE,
        );

        let scale = self.scale.clone();
        scroll_controller.connect_scroll(move |_, _dx, dy| {
            let current = scale.value();
            let step = 5.0;
            let new_val = if dy < 0.0 {
                (current + step).min(100.0)
            } else if dy > 0.0 {
                (current - step).max(0.0)
            } else {
                current
            };

            scale.set_value(new_val);
            glib::Propagation::Stop
        });

        widget.add_controller(scroll_controller);
    }

    /// Sets the slider value silently without triggering callbacks.
    pub fn set_value_silent(&self, val: f64) {
        self.silent.set(true);
        self.scale.set_value(val);
        self.silent.set(false);
    }

    /// Gets the current slider value.
    pub fn value(&self) -> f64 {
        self.scale.value()
    }
}
