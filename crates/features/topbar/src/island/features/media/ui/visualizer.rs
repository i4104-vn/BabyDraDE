//! Animated wave-bars visualizer for the media player island view.

use gtk4::prelude::*;
use std::cell::Cell;
use std::rc::Rc;

/// Builds the visualizer and returns its box plus the playing-state flag.
pub fn create_visualizer() -> (gtk4::Box, Rc<Cell<bool>>) {
    let visualizer_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(2)
        .css_classes(["notch-visualizer"])
        .valign(gtk4::Align::Center)
        .build();

    let mut bars = Vec::new();
    for _ in 0..4 {
        let bar = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        bar.add_css_class("visualizer-bar");
        bar.set_size_request(2, 2);
        bar.set_valign(gtk4::Align::End);
        visualizer_box.append(&bar);
        bars.push(bar);
    }

    let is_playing = Rc::new(Cell::new(false));
    let start_time = Rc::new(Cell::new(0i64));
    let was_playing = Rc::new(Cell::new(false));

    let Some(first) = bars.first().cloned() else {
        return (visualizer_box, is_playing);
    };
    first.add_tick_callback({
        let bars = Rc::new(bars);
        let is_playing = is_playing.clone();
        let start_time = start_time.clone();
        let was_playing = was_playing.clone();
        move |_, clock| {
            match true {
                _ if is_playing.get() => {
                    was_playing.set(true);
                    let now = clock.frame_time();
                    if start_time.get() == 0 {
                        start_time.set(now);
                    }
                    let elapsed = (now - start_time.get()) as f64 / 1_000_000.0;
                    for (i, bar) in bars.iter().enumerate() {
                        let a = (elapsed * (6.0 + i as f64 * 2.5)).sin();
                        let b = (elapsed * (4.0 - i as f64 * 1.2)).cos();
                        let height = ((a + b) / 2.0 * 4.0 + 6.0) as i32;
                        bar.set_size_request(2, height.clamp(2, 10));
                    }
                }
                _ if was_playing.replace(false) => {
                    start_time.set(0);
                    for bar in bars.iter() {
                        bar.set_size_request(2, 2);
                    }
                }
                _ => {}
            }
            glib::ControlFlow::Continue
        }
    });

    (visualizer_box, is_playing)
}
