use glow::HasContext;

pub(crate) fn gl_object<T>(
    value: i32,
    create: impl FnOnce(std::num::NonZeroU32) -> T,
) -> Option<T> {
    std::num::NonZeroU32::new(value as u32).map(create)
}

pub(crate) struct GlStateSnapshot<'a> {
    gl: &'a glow::Context,
    pub(crate) framebuffer: Option<glow::NativeFramebuffer>,
    pub(crate) viewport: [i32; 4],
    scissor_enabled: bool,
    scissor_box: [i32; 4],
    blend_enabled: bool,
    blend_src_rgb: u32,
    blend_dst_rgb: u32,
    blend_src_alpha: u32,
    blend_dst_alpha: u32,
    blend_equation_rgb: u32,
    blend_equation_alpha: u32,
    active_texture: u32,
    active_texture_binding: Option<glow::NativeTexture>,
    texture0_binding: Option<glow::NativeTexture>,
    array_buffer: Option<glow::NativeBuffer>,
    program: Option<glow::NativeProgram>,
    vertex_attribs: Vec<(u32, bool)>,
}

impl GlStateSnapshot<'_> {
    pub(crate) unsafe fn capture(gl: &glow::Context, locations: [u32; 3]) -> GlStateSnapshot<'_> {
        let mut viewport = [0; 4];
        gl.get_parameter_i32_slice(glow::VIEWPORT, &mut viewport);
        let mut scissor_box = [0; 4];
        gl.get_parameter_i32_slice(glow::SCISSOR_BOX, &mut scissor_box);

        let active_texture = gl.get_parameter_i32(glow::ACTIVE_TEXTURE) as u32;
        let active_texture_binding = gl_object(
            gl.get_parameter_i32(glow::TEXTURE_BINDING_2D),
            glow::NativeTexture,
        );
        gl.active_texture(glow::TEXTURE0);
        let texture0_binding = gl_object(
            gl.get_parameter_i32(glow::TEXTURE_BINDING_2D),
            glow::NativeTexture,
        );
        gl.active_texture(active_texture);

        let mut vertex_attribs = Vec::with_capacity(locations.len());
        for location in locations {
            if vertex_attribs
                .iter()
                .any(|(existing, _)| *existing == location)
            {
                continue;
            }
            let mut enabled = [0.0];
            gl.get_vertex_attrib_parameter_f32_slice(
                location,
                glow::VERTEX_ATTRIB_ARRAY_ENABLED,
                &mut enabled,
            );
            vertex_attribs.push((location, enabled[0] != 0.0));
        }

        GlStateSnapshot {
            gl,
            framebuffer: gl_object(
                gl.get_parameter_i32(glow::FRAMEBUFFER_BINDING),
                glow::NativeFramebuffer,
            ),
            viewport,
            scissor_enabled: gl.is_enabled(glow::SCISSOR_TEST),
            scissor_box,
            blend_enabled: gl.is_enabled(glow::BLEND),
            blend_src_rgb: gl.get_parameter_i32(glow::BLEND_SRC_RGB) as u32,
            blend_dst_rgb: gl.get_parameter_i32(glow::BLEND_DST_RGB) as u32,
            blend_src_alpha: gl.get_parameter_i32(glow::BLEND_SRC_ALPHA) as u32,
            blend_dst_alpha: gl.get_parameter_i32(glow::BLEND_DST_ALPHA) as u32,
            blend_equation_rgb: gl.get_parameter_i32(glow::BLEND_EQUATION_RGB) as u32,
            blend_equation_alpha: gl.get_parameter_i32(glow::BLEND_EQUATION_ALPHA) as u32,
            active_texture,
            active_texture_binding,
            texture0_binding,
            array_buffer: gl_object(
                gl.get_parameter_i32(glow::ARRAY_BUFFER_BINDING),
                glow::NativeBuffer,
            ),
            program: gl_object(
                gl.get_parameter_i32(glow::CURRENT_PROGRAM),
                glow::NativeProgram,
            ),
            vertex_attribs,
        }
    }
}

impl Drop for GlStateSnapshot<'_> {
    fn drop(&mut self) {
        unsafe {
            let gl = self.gl;
            gl.bind_framebuffer(glow::FRAMEBUFFER, self.framebuffer);
            gl.viewport(
                self.viewport[0],
                self.viewport[1],
                self.viewport[2],
                self.viewport[3],
            );
            gl.scissor(
                self.scissor_box[0],
                self.scissor_box[1],
                self.scissor_box[2],
                self.scissor_box[3],
            );
            if self.scissor_enabled {
                gl.enable(glow::SCISSOR_TEST);
            } else {
                gl.disable(glow::SCISSOR_TEST);
            }

            gl.bind_buffer(glow::ARRAY_BUFFER, self.array_buffer);
            gl.use_program(self.program);
            gl.blend_func_separate(
                self.blend_src_rgb,
                self.blend_dst_rgb,
                self.blend_src_alpha,
                self.blend_dst_alpha,
            );
            gl.blend_equation_separate(self.blend_equation_rgb, self.blend_equation_alpha);
            if self.blend_enabled {
                gl.enable(glow::BLEND);
            } else {
                gl.disable(glow::BLEND);
            }

            gl.active_texture(glow::TEXTURE0);
            gl.bind_texture(glow::TEXTURE_2D, self.texture0_binding);
            if self.active_texture != glow::TEXTURE0 {
                gl.active_texture(self.active_texture);
                gl.bind_texture(glow::TEXTURE_2D, self.active_texture_binding);
            }

            for (location, enabled) in &self.vertex_attribs {
                if *enabled {
                    gl.enable_vertex_attrib_array(*location);
                } else {
                    gl.disable_vertex_attrib_array(*location);
                }
            }
        }
    }
}
