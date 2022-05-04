//! Module containing the [UIRenderer] struct and its related [Vertex] struct.

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
const _SHADER_NAME: &str = "egui";
const TEXTURE_UNIT: gl::types::GLenum = gl::TEXTURE0;

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

/// The struct that does the rendering work of passing [egui]'s vertices to the GPU.
pub struct UIRenderer {
    ibo: buffer::ElementArrayBuffer,
    program: render_gl::Program,
    texture: buffer::Texture,
    vao: buffer::VertexArray,
    vbo: buffer::ArrayBuffer,
}

impl UIRenderer {
    /// Initializes [UIRenderer], compiling shaders and initializing buffers.
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

        let texture = buffer::Texture::new(TEXTURE_UNIT);
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

    /// Handle egui Texture updates.
    /// NOTE: I have not yet added support for multiple egui textures. As such, this is partially
    /// left unimplemented.
    pub fn egui_texture_delta(&self, textures_delta: egui::TexturesDelta) {
        // Free texture_ids no longer in use:
        for _texture_id in textures_delta.free {
            unimplemented!("Freeing egui textures is not currently implemented.");
        }

        if textures_delta.set.len() > 1 {
            unimplemented!("Using multiple egui textures is not currently implemented.");
        }

        for (_texture_id, image_delta) in textures_delta.set {
            self.apply_egui_texture_delta(image_delta);
        }
    }

    /// Uploads [egui::Texture] to the GPU.
    fn apply_egui_texture_delta(&self, texture_delta: egui::epaint::ImageDelta) {
        let (x, y, pixels) = match texture_delta.image {
            egui::ImageData::Color(image) => {
                let pixels: Vec<u8> = image
                    .pixels
                    .into_iter()
                    .flat_map(|c| c.to_array())
                    .collect();
                (image.size[0] as i32, image.size[1] as i32, pixels)
            }
            egui::ImageData::Font(image) => {
                let pixels: Vec<u8> = image.srgba_pixels(1.0).flat_map(|c| c.to_array()).collect();
                (image.size[0] as i32, image.size[1] as i32, pixels)
            }
        };

        if let Some([x_offset, y_offset]) = texture_delta.pos {
            self.texture.update_subtexture(
                (x, y),
                Some(pixels.as_slice()),
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                (x_offset as i32, y_offset as i32),
            );
        } else {
            self.texture.load_texture(
                (x, y),
                Some(pixels.as_slice()),
                gl::SRGB8_ALPHA8 as gl::types::GLint,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                true,
            );
        }
    }

    /// Renders `egui`'s vertices.
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
}
