pub mod border;
pub mod corners;
pub mod elements;

use presentation::BorderTheme;
use smithay::{
    backend::renderer::{
        element::{surface::WaylandSurfaceRenderElement, AsRenderElements},
        glow::GlowRenderer,
    },
    desktop::Window,
    utils::{Logical, Point, Scale},
};

pub use border::BorderShader;
pub use corners::CornerShader;
pub use elements::{CustomRenderElements, WindowRenderElement};

pub fn render_window_with_borders(
    renderer: &mut GlowRenderer,
    window: &Window,
    location: Point<i32, Logical>,
    scale: Scale<f64>,
    theme: &BorderTheme,
) -> Vec<CustomRenderElements> {
    let mut elements = Vec::new();

    if theme.thickness > 0.0 {
        let border_elem = BorderShader::element(renderer, window, location, theme);
        elements.push(CustomRenderElements::Shader(border_elem));
    }

    let surface_elements = window.render_elements::<WaylandSurfaceRenderElement<GlowRenderer>>(
        renderer,
        location.to_physical_precise_round(scale),
        scale,
        1.0,
    );

    let mut iter = surface_elements.into_iter();
    if let Some(top) = iter.next() {
        if theme.radius > 0.0 {
            elements.push(CustomRenderElements::Window(WindowRenderElement::new(
                top,
                theme.radius,
            )));
        } else {
            elements.push(CustomRenderElements::Surface(top));
        }
    }
    for surface in iter {
        elements.push(CustomRenderElements::Surface(surface));
    }

    elements
}
