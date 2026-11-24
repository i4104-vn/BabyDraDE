//! Editor window assembly: overlay layout, canvas, and toolbar placement.

use gtk4::prelude::*;
use gtk4_layer_shell::{Edge, KeyboardMode, Layer};
use std::cell::RefCell;
use std::rc::Rc;

use super::models::ToolbarContext;
use super::toolbar;
use crate::adapters::desktop::screenshot::editor::canvas::{
    draw_editor_canvas, setup_editor_gest,
};
use crate::adapters::desktop::screenshot::editor::models::EditorState;
use crate::adapters::desktop::screenshot::editor::setup_editor_keys;
use ui_kit::layer_shell::{init_layer_window, LayerWindowConfig};

/// Constructs the screenshot editor window, maps its overlay design,
/// initializes the canvas, and builds the editing toolbars.
///
/// # Arguments
///
/// * `app` - Application that owns the editor window.
/// * `temp_path` - File path of the captured screenshot.
///
/// # Returns
///
/// The configured editor window, or a bare window when the screenshot cannot be loaded.
pub(crate) fn build_editor_ui(app: &gtk4::Application, temp_path: &str) -> gtk4::ApplicationWindow {
    let pixbuf = match gdk_pixbuf::Pixbuf::from_file(temp_path) {
        Ok(pb) => pb,
        Err(_) => return gtk4::ApplicationWindow::new(app),
    };

    let state = Rc::new(RefCell::new(EditorState::new(pixbuf)));
    let window = gtk4::ApplicationWindow::new(app);

    // Stretch across the entire screen, ignoring panel exclusive zones
    init_layer_window(
        &window,
        &LayerWindowConfig {
            layer: Layer::Overlay,
            keyboard_mode: KeyboardMode::Exclusive,
            exclusive_zone: -1,
            anchors: &[
                (Edge::Top, true),
                (Edge::Bottom, true),
                (Edge::Left, true),
                (Edge::Right, true),
            ],
            margin_bottom: 0,
            namespace: Some("babydra-screenshot"),
        },
    );
    window.add_css_class("screenshot-window");

    let overlay = gtk4::Overlay::new();
    window.set_child(Some(&overlay));

    let drawing_area = setup_canvas(&state, &overlay);

    let toolbar_wrapper = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
    toolbar_wrapper.set_halign(gtk4::Align::Center);
    toolbar_wrapper.set_valign(gtk4::Align::End);
    toolbar_wrapper.set_margin_bottom(30);
    toolbar_wrapper.set_visible(false); // Hidden initially

    let ctx = ToolbarContext {
        state: state.clone(),
        window: window.clone(),
        canvas: drawing_area.clone(),
        wrapper: toolbar_wrapper.clone(),
    };
    let parts = toolbar::build(&ctx);
    toolbar_wrapper.append(&parts.toolbar);
    overlay.add_overlay(&toolbar_wrapper);

    setup_editor_gest(
        &drawing_area,
        state.clone(),
        &toolbar_wrapper,
        &parts.pen,
        &parts.color_popover,
    );
    setup_editor_keys(&window, state.clone(), &drawing_area);

    window
}

/// Creates the drawing area and connects its cached canvas render callback.
///
/// # Arguments
///
/// * `state` - Shared editor state read by the draw function.
/// * `overlay` - Container that receives the canvas as its child.
///
/// # Returns
///
/// The configured drawing area.
fn setup_canvas(
    state: &Rc<RefCell<EditorState>>,
    overlay: &gtk4::Overlay,
) -> gtk4::DrawingArea {
    let drawing_area = gtk4::DrawingArea::new();
    drawing_area.set_hexpand(true);
    drawing_area.set_vexpand(true);
    overlay.set_child(Some(&drawing_area));

    let state_draw = state.clone();
    let canvas_scale = drawing_area.clone();
    let layer_cache: super::super::canvas::SharedCache = Rc::new(RefCell::new(None));
    drawing_area.set_draw_func(move |_, cr, width, height| {
        let scale_factor = canvas_scale.scale_factor();
        {
            let mut s_mut = state_draw.borrow_mut();
            s_mut.canvas_w = width as f64;
            s_mut.canvas_h = height as f64;
        }
        let s = state_draw.borrow();
        draw_editor_canvas(
            cr,
            &s,
            width as f64,
            height as f64,
            &layer_cache,
            scale_factor,
        );
    });

    drawing_area
}
