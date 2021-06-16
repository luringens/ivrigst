use crate::{
    render_gl::{
        self, buffer,
        data::{self, f32_f32_f32},
        Program,
    },
    resources::Resources,
};
use anyhow::{Context, Result};
use nalgebra as na;
use render_gl_derive::VertexAttribPointers;

const SHADER_PATH: &str = "shaders/triangle";
const SHADER_NAME: &str = "triangle";

#[derive(Copy, Clone, Debug, VertexAttribPointers)]
#[repr(C, packed)]
pub struct Vertex {
    #[location = 0]
    pub pos: data::f32_f32_f32,
    #[location = 1]
    pub normal: data::f32_f32_f32,
}

pub struct Model {
    program: render_gl::Program,
    vao: buffer::VertexArray,
    _vbo: buffer::ArrayBuffer,
    ibo: buffer::ElementArrayBuffer,
    indices: i32,
    size: na::Vector3<f32>,
}

impl Model {
    pub fn new(res: &Resources) -> Result<Self> {
        // set up shader program
        let program = render_gl::Program::from_res(res, SHADER_PATH)?;

        let model = res
            .load_model("model.obj")
            .context("Failed to load model.")?;

        let mut min = na::Vector3::from_element(f32::MAX);
        let mut max = na::Vector3::from_element(f32::MIN);
        for pos in model.positions.chunks_exact(3) {
            min[0] = min[0].min(pos[0]);
            max[0] = max[0].max(pos[0]);
            min[1] = min[1].min(pos[1]);
            max[1] = max[1].max(pos[1]);
            min[2] = min[2].min(pos[2]);
            max[2] = max[2].max(pos[2]);
        }
        let center = min + (max - min) / 2.0;

        let vertices: Vec<Vertex> = model
            .positions
            .chunks_exact(3)
            .zip(model.normals.chunks_exact(3))
            .map(|(p, n)| {
                (
                    f32_f32_f32::from((p[0] - center[0], p[1] - center[1], p[2] - center[2])),
                    f32_f32_f32::from((n[0], n[1], n[2])),
                )
            })
            .map(|(pos, normal)| Vertex { pos, normal })
            .collect();
        let vbo = buffer::ArrayBuffer::new();
        vbo.bind();
        vbo.static_draw_data(&vertices);

        // set up vertex array object
        let vao = buffer::VertexArray::new();
        vao.bind();
        Vertex::vertex_attrib_pointers();

        // indices buffer
        let ibo = buffer::ElementArrayBuffer::new();
        ibo.bind();
        ibo.static_draw_data(&model.indices);
        ibo.unbind();
        vbo.unbind();
        vao.unbind();

        Ok(Self {
            program,
            _vbo: vbo,
            vao,
            ibo,
            indices: model.indices.len() as i32,
            size: max - min,
        })
    }

    pub fn shader(&mut self) -> &mut Program {
        &mut self.program
    }

    pub fn get_size(&self) -> &na::Vector3<f32> {
        &self.size
    }

    pub fn render(&self) {
        self.program.set_used();
        self.vao.bind();
        self.ibo.bind();

        unsafe {
            gl::Disable(gl::BLEND);
            gl::Enable(gl::DEPTH_TEST);
            gl::DepthFunc(gl::LESS);
            gl::Enable(gl::CULL_FACE);
            gl::DrawElements(
                gl::TRIANGLES,
                self.indices,
                gl::UNSIGNED_INT,
                std::ptr::null::<std::ffi::c_void>(),
            );
        }
        self.ibo.unbind();
        self.vao.unbind();
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
