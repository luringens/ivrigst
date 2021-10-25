//! Module containing the [TextureTester] struct and its related [Vertex] struct.

use crate::{
    render_gl::{
        self,
        buffer::{self, Texture},
        data::{self, f32_f32},
        Viewport,
    },
    resources::Resources,
};
use anyhow::Result;
use render_gl_derive::VertexAttribPointers;

const SHADER_PATH: &str = "shaders/texture_tester";
const SHADER_NAME: &str = "texture_tester";

#[derive(Copy, Clone, Debug, VertexAttribPointers)]
#[repr(C, packed)]
struct Vertex {
    #[location = 0]
    pub pos: data::f32_f32,
    #[location = 1]
    pub uv: data::f32_f32,
}

/// Renders textures to corners of the viewport. This is used to debug
/// intermediate render passes for shadows and hatching.
pub struct TextureTester {
    program: render_gl::Program,
    vao1: buffer::VertexArray,
    vao2: buffer::VertexArray,
    _vbo1: buffer::ArrayBuffer,
    _vbo2: buffer::ArrayBuffer,
    ibo: buffer::ElementArrayBuffer,
    indices: i32,
}

impl TextureTester {
    /// Set up TextureTester, compiling shaders and initializing buffers.
    pub fn new(res: &Resources) -> Result<Self> {
        // Compile shader program
        let program = render_gl::Program::from_res(res, SHADER_PATH)?;

        let vertices1: Vec<Vertex> = vec![
            Vertex {
                pos: f32_f32::from((0.5, -1.0)),
                uv: f32_f32::from((0.0, 0.0)),
            },
            Vertex {
                pos: f32_f32::from((1.0, -1.0)),
                uv: f32_f32::from((1.0, 0.0)),
            },
            Vertex {
                pos: f32_f32::from((0.5, -0.51)),
                uv: f32_f32::from((0.0, 1.0)),
            },
            Vertex {
                pos: f32_f32::from((1.0, -0.51)),
                uv: f32_f32::from((1.0, 1.0)),
            },
        ];

        let vertices2: Vec<Vertex> = vec![
            Vertex {
                pos: f32_f32::from((0.5, -0.49)),
                uv: f32_f32::from((0.0, 0.0)),
            },
            Vertex {
                pos: f32_f32::from((1.0, -0.49)),
                uv: f32_f32::from((1.0, 0.0)),
            },
            Vertex {
                pos: f32_f32::from((0.5, 0.0)),
                uv: f32_f32::from((0.0, 1.0)),
            },
            Vertex {
                pos: f32_f32::from((1.0, 0.0)),
                uv: f32_f32::from((1.0, 1.0)),
            },
        ];

        // Build array buffers
        let vbo1 = buffer::ArrayBuffer::new();
        vbo1.bind();
        vbo1.static_draw_data(&vertices1);
        let vao1 = buffer::VertexArray::new();
        vao1.bind();
        Vertex::vertex_attrib_pointers();
        vbo1.unbind();

        let vbo2 = buffer::ArrayBuffer::new();
        vbo2.bind();
        vbo2.static_draw_data(&vertices2);
        let vao2 = buffer::VertexArray::new();
        vao2.bind();
        Vertex::vertex_attrib_pointers();
        vbo2.unbind();

        // Build indice buffer
        let indices: Vec<u32> = vec![0, 1, 2, 1, 2, 3];
        let ibo = buffer::ElementArrayBuffer::new();
        ibo.bind();
        ibo.static_draw_data(&indices);
        ibo.unbind();

        let value = Self {
            program,
            _vbo1: vbo1,
            _vbo2: vbo2,
            vao1,
            vao2,
            ibo,
            indices: indices.len() as i32,
        };
        Ok(value)
    }

    /// Render the given textures to the viewport.
    pub fn render(&self, viewport: &Viewport, texture1: &Texture, texture2: &Texture) {
        self.program.set_used();
        viewport.set_used();

        self.vao1.bind();
        self.ibo.bind();
        unsafe {
            gl::Disable(gl::CULL_FACE);
            gl::Disable(gl::BLEND);
            gl::Disable(gl::DEPTH_TEST);
            texture1.bind_to(gl::TEXTURE0);
            gl::DrawElements(
                gl::TRIANGLES,
                self.indices,
                gl::UNSIGNED_INT,
                std::ptr::null::<std::ffi::c_void>(),
            );
            texture1.unbind();
            self.vao1.unbind();

            self.vao2.bind();
            self.ibo.bind();
            texture2.bind_to(gl::TEXTURE0);
            gl::DrawElements(
                gl::TRIANGLES,
                self.indices,
                gl::UNSIGNED_INT,
                std::ptr::null::<std::ffi::c_void>(),
            );
            texture2.unbind();
            self.vao2.unbind();
        }
        self.ibo.unbind();
    }

    /// Check if the shader has been updated.
    pub fn check_shader_update(&mut self, path: &std::path::Path, res: &Resources) -> bool {
        let path = path.file_stem().map(|p| p.to_string_lossy().to_string());
        if path == Some(SHADER_NAME.to_string()) {
            match render_gl::Program::from_res(res, SHADER_PATH) {
                Ok(program) => {
                    self.program.unset_used();
                    self.program = program;
                    return true;
                }
                Err(e) => eprintln!("Shader reload error: {}", e),
            }
        }
        false
    }
}
