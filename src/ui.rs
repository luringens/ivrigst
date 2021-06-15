use crate::{
    render_gl::{
        self, buffer,
        data::{f32_f32, f32_f32_f32_f32},
    },
    resources::Resources,
};
use anyhow::Result;
use render_gl_derive::VertexAttribPointers;

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

pub struct UI {
    program: render_gl::Program,
    vao: buffer::VertexArray,
    vbo: buffer::ArrayBuffer,
    ibo: buffer::ElementArrayBuffer,
    texture: buffer::Texture,
}

impl UI {
    pub fn new(res: &Resources) -> Result<Self> {
        // set up shader program
        let program = render_gl::Program::from_res(res, "shaders/textured")?;

        let vbo = buffer::ArrayBuffer::new();
        vbo.bind();

        // set up vertex array object
        let vao = buffer::VertexArray::new();
        vao.bind();
        Vertex::vertex_attrib_pointers();

        // indices buffer
        let ibo = buffer::ElementArrayBuffer::new();
        vao.unbind();
        vbo.unbind();

        let texture = buffer::Texture::new();

        Ok(Self {
            program,
            vbo,
            vao,
            ibo,
            texture,
        })
    }

    pub fn set_texture(&self, width: i32, height: i32, pixels: &[u8]) {
        self.texture.load_texture(width, height, pixels);
    }

    pub fn render(
        &self,
        vertices: &[egui::epaint::Vertex],
        indices: &[u32],
        window_size: (u32, u32),
    ) {
        let vertices: Vec<Vertex> = vertices.into_iter().map(From::from).collect();
        self.program.set_used();

        self.vbo.bind();
        self.vbo.dynamic_draw_data(&vertices);

        // set up vertex array object
        self.vao.bind();
        Vertex::vertex_attrib_pointers();

        // indices buffer
        self.ibo.bind();
        self.ibo.dynamic_draw_data(&indices);

        unsafe {
            gl::DrawElements(
                gl::TRIANGLES,
                indices.len() as i32,
                gl::UNSIGNED_INT,
                0 as *const std::ffi::c_void,
            );
        }
        self.ibo.unbind();
        self.vao.unbind();
        self.vbo.unbind();
    }
}
