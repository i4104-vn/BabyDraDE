//! Compiled blur program handles.

pub(crate) struct DownsampleProgram {
    pub(crate) program: glow::Program,
    pub(crate) u_texture: glow::UniformLocation,
    pub(crate) u_halfpixel: glow::UniformLocation,
    pub(crate) u_offset: glow::UniformLocation,
    pub(crate) u_scale_uv: glow::UniformLocation,
    pub(crate) a_position: u32,
}

pub(crate) struct UpsampleProgram {
    pub(crate) program: glow::Program,
    pub(crate) u_texture: glow::UniformLocation,
    pub(crate) u_halfpixel: glow::UniformLocation,
    pub(crate) u_offset: glow::UniformLocation,
    pub(crate) u_scale_uv: glow::UniformLocation,
    pub(crate) a_position: u32,
}

pub(crate) struct CompositeProgram {
    pub(crate) program: glow::Program,
    pub(crate) u_texture: glow::UniformLocation,
    pub(crate) u_rect_size: glow::UniformLocation,
    pub(crate) u_radius: glow::UniformLocation,
    pub(crate) u_opacity: glow::UniformLocation,
    pub(crate) u_tint: glow::UniformLocation,
    pub(crate) u_noise: glow::UniformLocation,
    pub(crate) u_brightness: glow::UniformLocation,
    pub(crate) u_contrast: glow::UniformLocation,
    pub(crate) u_scale_uv: glow::UniformLocation,
    pub(crate) a_position: u32,
}
