use std::{borrow::BorrowMut, cell::RefCell, collections::HashMap};

use presentation::BorderTheme;
use smithay::{
    backend::renderer::{
        element::{Element, Kind},
        gles::{
            element::PixelShaderElement, GlesPixelProgram, GlesRenderer, Uniform, UniformName,
            UniformType,
        },
        glow::GlowRenderer,
    },
    desktop::Window,
    utils::{IsAlive, Logical, Point, Rectangle, Size},
};

const ROUNDED_BORDER_FRAG: &str = include_str!("shaders/rounded_borders.frag");
const BORDER_FRAG: &str = include_str!("shaders/borders.frag");

pub struct BorderShader {
    rounded: GlesPixelProgram,
    default: GlesPixelProgram,
}

struct BorderShaderElements(RefCell<HashMap<Window, PixelShaderElement>>);

impl BorderShader {
    pub fn init(renderer: &mut GlowRenderer) {
        let renderer: &mut GlesRenderer = renderer.borrow_mut();
        let rounded = renderer
            .compile_custom_pixel_shader(
                ROUNDED_BORDER_FRAG,
                &[
                    UniformName::new("startColor", UniformType::_3f),
                    UniformName::new("endColor", UniformType::_3f),
                    UniformName::new("thickness", UniformType::_1f),
                    UniformName::new("halfThickness", UniformType::_1f),
                    UniformName::new("radius", UniformType::_1f),
                    UniformName::new("gradientDirection", UniformType::_2f),
                ],
            )
            .expect("Failed to compile rounded border fragment shader");

        let default = renderer
            .compile_custom_pixel_shader(
                BORDER_FRAG,
                &[
                    UniformName::new("startColor", UniformType::_3f),
                    UniformName::new("endColor", UniformType::_3f),
                    UniformName::new("thickness", UniformType::_1f),
                    UniformName::new("halfThickness", UniformType::_1f),
                    UniformName::new("gradientDirection", UniformType::_2f),
                ],
            )
            .expect("Failed to compile border fragment shader");

        renderer
            .egl_context()
            .user_data()
            .insert_if_missing(|| BorderShader { rounded, default });
        renderer
            .egl_context()
            .user_data()
            .insert_if_missing(|| BorderShaderElements(RefCell::new(HashMap::new())));
    }

    pub fn get(renderer: &GlowRenderer) -> &BorderShader {
        renderer
            .egl_context()
            .user_data()
            .get::<BorderShader>()
            .expect("BorderShader not initialized")
    }

    pub fn element(
        renderer: &mut GlowRenderer,
        window: &Window,
        loc: Point<i32, Logical>,
        theme: &BorderTheme,
    ) -> PixelShaderElement {
        let thickness = theme.thickness;
        let thickness_loc = (thickness as i32, thickness as i32);
        let thickness_size = ((thickness * 2.0) as i32, (thickness * 2.0) as i32);
        let geo = Rectangle::new(
            loc - Point::from(thickness_loc),
            window.geometry().size + Size::from(thickness_size),
        );

        let elements = &mut renderer
            .egl_context()
            .user_data()
            .get::<BorderShaderElements>()
            .expect("BorderShaderElements not initialized")
            .0
            .borrow_mut();

        if let Some(elem) = elements.get_mut(window) {
            if elem.geometry(1.0.into()).to_logical(1) != geo {
                elem.resize(geo, None);
            }
            elem.clone()
        } else {
            let angle = theme.gradient_angle;
            let gradient_direction = [angle.cos(), angle.sin()];

            let elem = if theme.radius > 0.0 {
                PixelShaderElement::new(
                    Self::get(renderer).rounded.clone(),
                    geo,
                    None,
                    1.0,
                    vec![
                        Uniform::new("startColor", theme.start_color),
                        Uniform::new("endColor", theme.end_color),
                        Uniform::new("thickness", thickness),
                        Uniform::new("halfThickness", thickness * 0.5),
                        Uniform::new("radius", theme.radius + thickness + 2.0),
                        Uniform::new("gradientDirection", gradient_direction),
                    ],
                    Kind::Unspecified,
                )
            } else {
                PixelShaderElement::new(
                    Self::get(renderer).default.clone(),
                    geo,
                    None,
                    1.0,
                    vec![
                        Uniform::new("startColor", theme.start_color),
                        Uniform::new("endColor", theme.end_color),
                        Uniform::new("thickness", thickness),
                        Uniform::new("halfThickness", thickness * 0.5),
                        Uniform::new("gradientDirection", gradient_direction),
                    ],
                    Kind::Unspecified,
                )
            };

            elements.insert(window.clone(), elem.clone());
            elem
        }
    }

    pub fn cleanup(renderer: &mut GlowRenderer) {
        if let Some(elements) = renderer
            .egl_context()
            .user_data()
            .get::<BorderShaderElements>()
        {
            elements.0.borrow_mut().retain(|w, _| w.alive());
        }
    }
}
