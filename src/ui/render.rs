use crate::{
    render_gl::{
        self, buffer,
        data::{f32_f32, f32_f32_f32_f32},
    },
    resources::Resources,
};
use anyhow::Result;
use render_gl_derive::VertexAttribPointers;

const SHADER_PATH: &str = "shaders/egui";
const SHADER_NAME: &str = "egui";

#[derive(Copy, Clone, Debug, VertexAttribPointers)]
#[repr(C, packed)]
struct Vertex {
    #[location = 0]
    pos: f32_f32,
    #[location = 1]
    uv: f32_f32,
    #[location = 2]
    color: f32_f32_f32_f32,
}

impl From<&egui::epaint::Vertex> for Vertex {
    fn from(v: &egui::epaint::Vertex) -> Self {
        Self {
            pos: (v.pos.x, v.pos.y).into(),
            uv: (v.uv.x, v.uv.y).into(),
            color: (
                v.color.r() as f32,
                v.color.g() as f32,
                v.color.b() as f32,
                v.color.a() as f32,
            )
                .into(),
        }
    }
}

pub struct UIRenderer {
    ibo: buffer::ElementArrayBuffer,
    program: render_gl::Program,
    texture: buffer::Texture,
    vao: buffer::VertexArray,
    vbo: buffer::ArrayBuffer,
}

impl UIRenderer {
    pub fn new(res: &Resources) -> Result<Self> {
        // set up shader program
        let program = render_gl::Program::from_res(res, SHADER_PATH)?;

        let vbo = buffer::ArrayBuffer::new();
        vbo.bind();

        // set up vertex array object
        let vao = buffer::VertexArray::new();
        vao.bind();
        Vertex::vertex_attrib_pointers();

        // indices buffer
        let ibo = buffer::ElementArrayBuffer::new();
        ibo.bind();

        let texture = buffer::Texture::new();
        texture.bind();
        texture.unbind();
        ibo.unbind();
        vao.unbind();
        vbo.unbind();

        Ok(Self {
            ibo,
            program,
            texture,
            vao,
            vbo,
        })
    }

    pub fn set_texture(&self, width: i32, height: i32, texture: &egui::Texture) {
        let pixels: Vec<u8> = texture
            .pixels
            .iter()
            .map(|&a| egui::epaint::Color32::from_white_alpha(a).to_tuple())
            .flat_map(|(r, g, b, a)| std::array::IntoIter::new([r, g, b, a]))
            .collect();
        self.texture.load_texture(
            (width, height),
            Some(pixels.as_slice()),
            gl::SRGB8_ALPHA8 as gl::types::GLint,
            gl::RGBA,
            gl::UNSIGNED_BYTE,
            true,
        );
    }

    pub fn render(
        &self,
        vertices: &[egui::epaint::Vertex],
        indices: &[u32],
        _clip_rect: egui::Rect,
        window_size: (u32, u32),
    ) {
        let vertices: Vec<Vertex> = vertices.iter().map(From::from).collect();
        self.program.set_used();

        self.vbo.bind();
        self.vbo.dynamic_draw_data(vertices.as_slice());

        self.vao.bind();
        self.ibo.bind();
        self.ibo.dynamic_draw_data(indices);

        self.texture.bind();

        unsafe {
            // All UI elements have the same depth, so don't do depth testing
            gl::Disable(gl::DEPTH_TEST);

            self.program
                .set_uniform_2f("screen_size", (window_size.0 as f32, window_size.1 as f32));

            // egui has no consistent mesh normal direction
            gl::Disable(gl::CULL_FACE);

            // Pre-multiplied alpha.
            gl::Enable(gl::BLEND);
            gl::BlendFuncSeparate(
                gl::ONE,
                gl::ONE_MINUS_SRC_ALPHA,
                gl::ONE_MINUS_DST_ALPHA,
                gl::ONE,
            );
            gl::DrawElements(
                gl::TRIANGLES,
                indices.len() as i32,
                gl::UNSIGNED_INT,
                std::ptr::null::<std::ffi::c_void>(),
            );
            // Reset blending to default
        }
        self.texture.unbind();
        self.ibo.unbind();
        self.vao.unbind();
        self.vbo.unbind();
    }

    pub fn check_shader_update(&mut self, path: &std::path::Path, res: &Resources) {
        let path = path.file_stem().map(|p| p.to_string_lossy().to_string());
        if path == Some(SHADER_NAME.to_string()) {
            match render_gl::Program::from_res(res, SHADER_PATH) {
                Ok(program) => {
                    self.program.unset_used();
                    self.program = program
                }
                Err(e) => eprintln!("Shader reload error: {}", e),
            }
        }
    }
}
