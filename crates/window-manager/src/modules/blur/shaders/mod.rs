//! GLSL shader sources, compilation and linked program handles.

mod compile;
mod sources;

pub(crate) use super::models::{CompositeProgram, DownsampleProgram, UpsampleProgram};
pub(crate) use compile::{
    compile_composite, compile_downsample, compile_upsample, setup_tex_params,
};
pub(crate) use sources::{
    COMPOSITE_SHADER_SRC, DOWNSAMPLE_SHADER_SRC, QUAD_VERTICES, UPSAMPLE_SHADER_SRC,
    VERTEX_SHADER_SRC,
};
