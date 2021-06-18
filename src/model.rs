use crate::{
    render_gl::{
        self, buffer,
        data::{self, f32_f32_f32},
    },
    resources::Resources,
};
use anyhow::{Context, Result};
use nalgebra as na;
use render_gl_derive::VertexAttribPointers;

const SHADER_PATH: &str = "shaders/model";
const SHADER_NAME: &str = "model";

#[derive(Copy, Clone, Debug, VertexAttribPointers)]
#[repr(C, packed)]
pub struct Vertex {
    #[location = 0]
    pub pos: data::f32_f32_f32,
    #[location = 1]
    pub normal: data::f32_f32_f32,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(C)]
pub enum DistanceShadingChannel {
    None = 0,
    Hue = 1,
    Saturation = 2,
    Value = 3,
}

#[derive(Debug, Clone)]
pub struct Attributes {
    pub projection_matrix: na::Matrix4<f32>,
    pub camera_position: [f32; 3],
    pub color: [f32; 3],
    pub model_size: f32,
    pub distance_shading_power: f32,
    pub distance_shading_constrict: f32,
    pub toon_factor: f32,
    pub distance_shading_channel: DistanceShadingChannel,
}

impl Default for Attributes {
    fn default() -> Self {
        Self {
            projection_matrix: Default::default(),
            camera_position: Default::default(),
            color: [1.0, 0.56, 0.72],
            model_size: Default::default(),
            distance_shading_power: 0.8,
            distance_shading_constrict: 0.2,
            toon_factor: 0.8,
            distance_shading_channel: DistanceShadingChannel::None,
        }
    }
}

pub struct Model {
    program: render_gl::Program,
    vao: buffer::VertexArray,
    _vbo: buffer::ArrayBuffer,
    ibo: buffer::ElementArrayBuffer,
    indices: i32,
    size: na::Vector3<f32>,
    attributes: Attributes,
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

        unsafe {
            program.set_used();
            program.set_uniform_3f("camera_position", (10.0, 0.0, 0.0));
            program.set_uniform_3f("color", (0.9, 0.5, 0.9));
            program.set_uniform_f("model_size", 100.0);
            program.set_uniform_f("distance_shading_power", 0.5);
            program.set_uniform_f("distance_shading_constrict", 1.0);
            program.set_uniform_f("toon_factor", 0.5);
            program.unset_used();
            render_gl::check_gl_error();
        }

        Ok(Self {
            program,
            _vbo: vbo,
            vao,
            ibo,
            indices: model.indices.len() as i32,
            size: max - min,
            attributes: Default::default(),
        })
    }

    pub fn get_attributes(&self) -> &Attributes {
        &self.attributes
    }

    pub fn set_attributes(&mut self, attributes: Attributes) {
        self.program.set_used();
        unsafe {
            if attributes.projection_matrix != self.attributes.projection_matrix {
                self.program
                    .set_uniform_matrix4("projection_matrix", attributes.projection_matrix);
            }
            if attributes.camera_position != self.attributes.camera_position {
                self.program
                    .set_uniform_3f_arr("camera_position", attributes.camera_position);
            }
            if attributes.color != self.attributes.color {
                self.program.set_uniform_3f_arr("color", attributes.color);
            }
            if attributes.model_size != self.attributes.model_size {
                self.program
                    .set_uniform_f("model_size", attributes.model_size);
            }
            if attributes.distance_shading_power != self.attributes.distance_shading_power {
                self.program
                    .set_uniform_f("distance_shading_power", attributes.distance_shading_power);
            }
            if attributes.distance_shading_constrict != self.attributes.distance_shading_constrict {
                self.program.set_uniform_f(
                    "distance_shading_constrict",
                    attributes.distance_shading_constrict,
                );
            }
            if attributes.toon_factor != self.attributes.toon_factor {
                self.program
                    .set_uniform_f("toon_factor", attributes.toon_factor);
            }
            if attributes.distance_shading_channel != self.attributes.distance_shading_channel {
                self.program.set_uniform_ui(
                    "distance_shading_channel",
                    attributes.distance_shading_channel as u32,
                )
            }
        }
        self.program.unset_used();
        self.attributes = attributes;
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

    pub fn check_shader_update(&mut self, path: &std::path::Path, res: &Resources) -> bool {
        let path = path.file_stem().map(|p| p.to_string_lossy().to_string());
        if path == Some(SHADER_NAME.to_string()) {
            match render_gl::Program::from_res(res, SHADER_PATH) {
                Ok(program) => {
                    self.program.unset_used();

                    program.set_used();
                    let attr = &self.attributes;
                    unsafe {
                        program.set_uniform_matrix4("projection_matrix", attr.projection_matrix);
                        program.set_uniform_3f_arr("camera_position", attr.camera_position);
                        program.set_uniform_3f_arr("color", attr.color);
                        program.set_uniform_f("model_size", attr.model_size);
                        program
                            .set_uniform_f("distance_shading_power", attr.distance_shading_power);
                        program.set_uniform_f(
                            "distance_shading_constrict",
                            attr.distance_shading_constrict,
                        );
                        program.set_uniform_f("toon_factor", attr.toon_factor);
                        program.set_uniform_ui(
                            "distance_shading_channel",
                            attr.distance_shading_channel as u32,
                        )
                    }
                    program.unset_used();
                    self.program = program;
                    return true;
                }
                Err(e) => eprintln!("Shader reload error: {}", e),
            }
        }
        false
    }
}
