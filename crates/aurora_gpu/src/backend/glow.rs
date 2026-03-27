use crate::gpu_context::GpuContext;
use aurora_core::color::Color;
use glow::HasContext;
use glutin::config::ConfigTemplateBuilder;
use glutin::context::{ContextApi, ContextAttributesBuilder, PossiblyCurrentContext, Version};
use glutin::display::GetGlDisplay;
use glutin::prelude::*;
use glutin::surface::{Surface, SwapInterval, WindowSurface};
use glutin_winit::DisplayBuilder;
use raw_window_handle::HasWindowHandle;
use std::num::NonZeroU32;

const VERTEX_SHADER: &str = r#"#version 330 core
const vec2 pos[4] = vec2[](vec2(-1,-1), vec2(1,-1), vec2(-1,1), vec2(1,1));
const vec2 uv[4]  = vec2[](vec2(0,1), vec2(1,1), vec2(0,0), vec2(1,0));
out vec2 v_uv;
void main() {
    gl_Position = vec4(pos[gl_VertexID], 0.0, 1.0);
    v_uv = uv[gl_VertexID];
}
"#;

const FRAGMENT_SHADER: &str = r#"#version 330 core
in vec2 v_uv;
out vec4 frag_color;
uniform sampler2D tex;
void main() {
    frag_color = texture(tex, v_uv);
}
"#;

/// OpenGL GPU backend using `glow` + `glutin`.
///
/// Uploads the CPU pixel buffer as a texture and draws a fullscreen quad.
/// Presents via `SwapBuffers`, which bypasses WM_PAINT entirely on Windows.
pub struct GlowBackend {
    gl: glow::Context,
    gl_surface: Surface<WindowSurface>,
    gl_context: PossiblyCurrentContext,
    texture: glow::Texture,
    program: glow::Program,
    vao: glow::VertexArray,
    buffer: Vec<u32>,
    rgba_buffer: Vec<u8>,
    width: u32,
    height: u32,
}

impl GlowBackend {
    /// Creates a new OpenGL backend for the given window.
    pub fn new(
        window: &winit::window::Window,
        event_loop: &winit::event_loop::ActiveEventLoop,
    ) -> Result<Self, String> {
        let template = ConfigTemplateBuilder::new().with_alpha_size(8);

        let display_builder = DisplayBuilder::new();
        let (_, gl_config) = display_builder
            .build(event_loop, template, |configs| {
                configs
                    .reduce(|acc, config| {
                        if config.num_samples() > acc.num_samples() {
                            config
                        } else {
                            acc
                        }
                    })
                    .unwrap()
            })
            .map_err(|e| format!("Failed to build GL display: {e}"))?;

        let gl_display = gl_config.display();

        let raw_handle = window
            .window_handle()
            .map_err(|e| format!("Window handle error: {e}"))?;

        let context_attrs = ContextAttributesBuilder::new()
            .with_context_api(ContextApi::OpenGl(Some(Version::new(3, 3))))
            .build(Some(raw_handle.into()));

        let not_current_context = unsafe {
            gl_display
                .create_context(&gl_config, &context_attrs)
                .map_err(|e| format!("Failed to create GL context: {e}"))?
        };

        let physical = window.inner_size();
        let (w, h) = (
            NonZeroU32::new(physical.width.max(1)).unwrap(),
            NonZeroU32::new(physical.height.max(1)).unwrap(),
        );
        let surface_attrs =
            glutin::surface::SurfaceAttributesBuilder::<WindowSurface>::new().build(
                raw_handle.into(),
                w,
                h,
            );

        let gl_surface = unsafe {
            gl_display
                .create_window_surface(&gl_config, &surface_attrs)
                .map_err(|e| format!("Failed to create GL surface: {e}"))?
        };

        let gl_context = not_current_context
            .make_current(&gl_surface)
            .map_err(|e| format!("Failed to make GL context current: {e}"))?;

        let _ = gl_surface.set_swap_interval(
            &gl_context,
            SwapInterval::Wait(NonZeroU32::new(1).unwrap()),
        );

        let gl = unsafe {
            glow::Context::from_loader_function_cstr(|name| {
                gl_display.get_proc_address(name) as *const _
            })
        };

        let program = Self::create_program(&gl)?;

        let vao = unsafe {
            gl.create_vertex_array()
                .map_err(|e| format!("Failed to create VAO: {e}"))?
        };

        let texture = unsafe {
            let tex = gl
                .create_texture()
                .map_err(|e| format!("Failed to create texture: {e}"))?;
            gl.bind_texture(glow::TEXTURE_2D, Some(tex));
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MIN_FILTER,
                glow::NEAREST as i32,
            );
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MAG_FILTER,
                glow::NEAREST as i32,
            );
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_WRAP_S,
                glow::CLAMP_TO_EDGE as i32,
            );
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_WRAP_T,
                glow::CLAMP_TO_EDGE as i32,
            );
            tex
        };

        let width = physical.width;
        let height = physical.height;
        let pixel_count = (width * height) as usize;

        Ok(Self {
            gl,
            gl_surface,
            gl_context,
            texture,
            program,
            vao,
            buffer: vec![0u32; pixel_count],
            rgba_buffer: vec![0u8; pixel_count * 4],
            width,
            height,
        })
    }

    fn create_program(gl: &glow::Context) -> Result<glow::Program, String> {
        unsafe {
            let program = gl
                .create_program()
                .map_err(|e| format!("Failed to create program: {e}"))?;

            let vs = gl
                .create_shader(glow::VERTEX_SHADER)
                .map_err(|e| format!("Failed to create vertex shader: {e}"))?;
            gl.shader_source(vs, VERTEX_SHADER);
            gl.compile_shader(vs);
            if !gl.get_shader_compile_status(vs) {
                let log = gl.get_shader_info_log(vs);
                gl.delete_shader(vs);
                return Err(format!("Vertex shader error: {log}"));
            }

            let fs = gl
                .create_shader(glow::FRAGMENT_SHADER)
                .map_err(|e| format!("Failed to create fragment shader: {e}"))?;
            gl.shader_source(fs, FRAGMENT_SHADER);
            gl.compile_shader(fs);
            if !gl.get_shader_compile_status(fs) {
                let log = gl.get_shader_info_log(fs);
                gl.delete_shader(vs);
                gl.delete_shader(fs);
                return Err(format!("Fragment shader error: {log}"));
            }

            gl.attach_shader(program, vs);
            gl.attach_shader(program, fs);
            gl.link_program(program);
            if !gl.get_program_link_status(program) {
                let log = gl.get_program_info_log(program);
                gl.delete_shader(vs);
                gl.delete_shader(fs);
                gl.delete_program(program);
                return Err(format!("Program link error: {log}"));
            }

            gl.detach_shader(program, vs);
            gl.detach_shader(program, fs);
            gl.delete_shader(vs);
            gl.delete_shader(fs);

            Ok(program)
        }
    }

    fn convert_to_rgba(&mut self) {
        let rgba = &mut self.rgba_buffer;
        for (i, &pixel) in self.buffer.iter().enumerate() {
            let offset = i * 4;
            rgba[offset] = ((pixel >> 16) & 0xFF) as u8;
            rgba[offset + 1] = ((pixel >> 8) & 0xFF) as u8;
            rgba[offset + 2] = (pixel & 0xFF) as u8;
            rgba[offset + 3] = 255;
        }
    }
}

impl GpuContext for GlowBackend {
    fn resize(&mut self, width: u32, height: u32) {
        if width == 0 || height == 0 {
            return;
        }
        if self.width == width && self.height == height {
            return;
        }
        self.width = width;
        self.height = height;
        let pixel_count = (width as usize)
            .checked_mul(height as usize)
            .expect("pixel dimensions overflow");
        self.buffer.resize(pixel_count, 0);
        self.rgba_buffer.resize(pixel_count * 4, 0);

        if let (Some(w), Some(h)) = (NonZeroU32::new(width), NonZeroU32::new(height)) {
            self.gl_surface.resize(&self.gl_context, w, h);
        }

        unsafe {
            self.gl.viewport(0, 0, width as i32, height as i32);
        }
    }

    fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    fn clear(&mut self, color: Color) {
        self.buffer.fill(color.to_rgb_u32());
    }

    fn buffer_mut(&mut self) -> &mut [u32] {
        &mut self.buffer
    }

    fn present(&mut self) {
        if self.width == 0 || self.height == 0 {
            return;
        }

        self.convert_to_rgba();

        unsafe {
            self.gl
                .bind_texture(glow::TEXTURE_2D, Some(self.texture));
            self.gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                glow::RGBA as i32,
                self.width as i32,
                self.height as i32,
                0,
                glow::RGBA,
                glow::UNSIGNED_BYTE,
                glow::PixelUnpackData::Slice(Some(&self.rgba_buffer)),
            );

            self.gl.use_program(Some(self.program));
            self.gl.bind_vertex_array(Some(self.vao));
            self.gl.draw_arrays(glow::TRIANGLE_STRIP, 0, 4);
        }

        let _ = self.gl_surface.swap_buffers(&self.gl_context);
    }
}

impl Drop for GlowBackend {
    fn drop(&mut self) {
        unsafe {
            self.gl.delete_texture(self.texture);
            self.gl.delete_program(self.program);
            self.gl.delete_vertex_array(self.vao);
        }
    }
}
