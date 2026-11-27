//! PillSlider component — Smooth, pill-shaped GTK4 Scale slider with debouncing and mouse scroll.

use gtk4::glib;
use gtk4::prelude::*;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

/// How many step increments one wheel notch moves the slider.
const WHEEL_STEP_FACTOR: f64 = 5.0;

/// Derives the wheel step from the slider's own step increment.
fn wheel_step(step_increment: f64) -> f64 {
    if step_increment.is_finite() && step_increment != 0.0 {
        step_increment.abs() * WHEEL_STEP_FACTOR
    } else {
        1.0
    }
}

/// Returns the value after one discrete wheel step, clamped to `min..=max`.
///
/// `delta` follows GTK's convention: negative when scrolling up, positive
/// when scrolling down. Scrolling up increases the value, down decreases it.
fn stepped_value(current: f64, delta: f64, min: f64, max: f64, step: f64) -> f64 {
    if !min.is_finite() || !max.is_finite() || min > max {
        return current;
    }

    let current = if current.is_finite() {
        current.clamp(min, max)
    } else {
        min
    };

    if delta == 0.0 || !delta.is_finite() || !step.is_finite() || step <= 0.0 {
        return current;
    }

    let next = if delta < 0.0 {
        current + step
    } else {
        current - step
    };

    if next.is_finite() {
        next.clamp(min, max)
    } else if delta < 0.0 {
        max
    } else {
        min
    }
}

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

        scale.set_increments(step, wheel_step(step));

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

        let slider = Self {
            container,
            scale,
            silent,
            change_callbacks,
        };

        slider.attach_scroll_controller(&slider.container, gtk4::PropagationPhase::Capture);

        slider
    }

    /// Binds a label to show `{:.0}%` whenever the slider value changes.
    pub fn bind_label(&self, label: &gtk4::Label) {
        let lbl = label.clone();
        self.change_callbacks.borrow_mut().push(Rc::new(move |val| {
            lbl.set_text(&format!("{:.0}%", val));
        }));
    }

    /// Connects a callback that fires `ms` milliseconds after the last change.
    ///
    /// The first change after an idle period is applied immediately, so a
    /// single wheel notch gives instant feedback; rapid drag/scroll events
    /// are still coalesced into one trailing update.
    pub fn connect_debounced(&self, ms: u32, callback: impl Fn(f64) + 'static) {
        let callback_rc = Rc::new(callback);
        let pending_source: Rc<RefCell<Option<glib::SourceId>>> = Rc::new(RefCell::new(None));
        let fired_immediately = Rc::new(Cell::new(false));

        let scale_c = self.scale.clone();
        let silent_c = self.silent.clone();
        let source_slot = pending_source.clone();

        self.scale.connect_value_changed(move |s| {
            if silent_c.get() {
                return;
            }
            let val = s.value();

            if !fired_immediately.get() {
                fired_immediately.set(true);
                callback_rc(val);
            }

            if let Some(src) = source_slot.borrow_mut().take() {
                src.remove();
            }

            let sc = scale_c.clone();
            let cb = callback_rc.clone();
            let slot = source_slot.clone();
            let fired_c = fired_immediately.clone();

            let id =
                glib::timeout_add_local(std::time::Duration::from_millis(ms as u64), move || {
                    cb(sc.value());
                    *slot.borrow_mut() = None;
                    fired_c.set(false);
                    glib::ControlFlow::Break
                });

            *source_slot.borrow_mut() = Some(id);
        });
    }

    /// Adds a mouse scroll controller to a wrapper widget (e.g. the control row).
    ///
    /// The controller runs in capture phase and stops the event, so scroll
    /// events reaching the slider are stepped exactly once — either here or
    /// by the slider's own controller, never by both.
    pub fn add_scroll_to(&self, widget: &impl IsA<gtk4::Widget>) {
        self.attach_scroll_controller(widget, gtk4::PropagationPhase::Capture);
    }

    fn attach_scroll_controller(
        &self,
        widget: &impl IsA<gtk4::Widget>,
        phase: gtk4::PropagationPhase,
    ) {
        let scroll_controller = gtk4::EventControllerScroll::new(
            gtk4::EventControllerScrollFlags::VERTICAL | gtk4::EventControllerScrollFlags::DISCRETE,
        );
        scroll_controller.set_propagation_phase(phase);

        let scale = self.scale.clone();
        scroll_controller.connect_scroll(move |_, _dx, dy| {
            if dy == 0.0 {
                return glib::Propagation::Proceed;
            }

            let adjustment = scale.adjustment();
            let step = wheel_step(adjustment.step_increment());
            let next = stepped_value(
                scale.value(),
                dy,
                adjustment.lower(),
                adjustment.upper(),
                step,
            );

            if (next - scale.value()).abs() > f64::EPSILON {
                scale.set_value(next);
            }

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
