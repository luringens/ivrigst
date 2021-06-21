use crate::{
    render_gl::{
        self,
        buffer::{self, FrameBuffer, Texture},
        data::{self, f32_f32_f32},
        Viewport,
    },
    resources::Resources,
};
use anyhow::{Context, Result};
use nalgebra as na;
use render_gl_derive::VertexAttribPointers;

const SHADER_PATH: &str = "shaders/model";
const SHADER_NAME: &str = "model";
const SHADOW_WIDTH: gl::types::GLsizei = 2048;
const SHADOW_HEIGHT: gl::types::GLsizei = 2048;

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
    pub camera_position: na::Vector3<f32>,
    pub light_position: na::Vector3<f32>,
    pub color: na::Vector3<f32>,
    pub model_size: f32,
    pub distance_shading_power: f32,
    pub distance_shading_constrict: f32,
    pub toon_factor: f32,
    pub distance_shading_channel: DistanceShadingChannel,
    pub shadows: bool,
    pub shadows_follow: bool,
    pub shadows_orbit_radius: f32,
    pub elapsed: f32,
}

impl Default for Attributes {
    fn default() -> Self {
        Self {
            projection_matrix: Default::default(),
            camera_position: Default::default(),
            light_position: na::Vector3::new(0.45, 0.25, -0.6),
            color: na::Vector3::new(1.0, 0.56, 0.72),
            model_size: Default::default(),
            distance_shading_power: 0.8,
            distance_shading_constrict: 0.2,
            toon_factor: 0.8,
            distance_shading_channel: DistanceShadingChannel::None,
            shadows: true,
            shadows_follow: false,
            shadows_orbit_radius: 25.0,
            elapsed: 0.0,
        }
    }
}

pub struct Model {
    program: render_gl::Program,
    shadow_program: render_gl::Program,
    vao: buffer::VertexArray,
    _vbo: buffer::ArrayBuffer,
    ibo: buffer::ElementArrayBuffer,
    indices: i32,
    size: na::Vector3<f32>,
    attributes: Attributes,
    depth_map: Texture,
    depth_map_fbo: FrameBuffer,
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
        let model_size = (max - min).magnitude();

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

        // Shadowstuff
        let shadow_program = render_gl::Program::from_res(res, "shaders/shadow")?;
        shadow_program.set_used();

        let depth_map = Texture::new();
        depth_map.load_texture(
            (SHADOW_WIDTH, SHADOW_HEIGHT),
            None,
            gl::DEPTH_COMPONENT as gl::types::GLint,
            gl::DEPTH_COMPONENT,
            gl::FLOAT,
            false,
        );
        depth_map.set_border_color(&[1.0, 1.0, 1.0, 1.0]);

        let depth_map_fbo = FrameBuffer::new();
        depth_map_fbo.bind();
        depth_map_fbo.set_type(gl::NONE, gl::NONE);
        depth_map_fbo.bind_texture(gl::DEPTH_ATTACHMENT, &depth_map);
        depth_map_fbo.unbind();

        let attributes = Attributes {
            model_size,
            ..Default::default()
        };

        let value = Self {
            program,
            shadow_program,
            _vbo: vbo,
            vao,
            ibo,
            indices: model.indices.len() as i32,
            size: max - min,
            attributes,
            depth_map,
            depth_map_fbo,
        };
        value.reset_all_attributes();
        Ok(value)
    }

    pub fn get_attributes(&self) -> &Attributes {
        &self.attributes
    }

    pub fn set_attributes(&mut self, new: Attributes) {
        let old = &self.attributes;
        self.program.set_used();
        unsafe {
            if new.projection_matrix != old.projection_matrix {
                self.program
                    .set_uniform_matrix4("projection_matrix", &new.projection_matrix);
            }
            if new.camera_position != old.camera_position {
                self.program
                    .set_uniform_3f_na("camera_position", new.camera_position);
            }
            if new.color != old.color {
                self.program.set_uniform_3f_na("color", new.color);
            }
            if (new.model_size - old.model_size).abs() < f32::EPSILON {
                self.program.set_uniform_f("model_size", new.model_size);
            }
            if (new.distance_shading_power - old.distance_shading_power).abs() < f32::EPSILON {
                self.program
                    .set_uniform_f("distance_shading_power", new.distance_shading_power);
            }
            if (new.distance_shading_constrict - old.distance_shading_constrict).abs()
                < f32::EPSILON
            {
                self.program
                    .set_uniform_f("distance_shading_constrict", new.distance_shading_constrict);
            }
            if (new.toon_factor - old.toon_factor).abs() < f32::EPSILON {
                self.program.set_uniform_f("toon_factor", new.toon_factor);
            }
            if new.distance_shading_channel != old.distance_shading_channel {
                self.program.set_uniform_ui(
                    "distance_shading_channel",
                    new.distance_shading_channel as u32,
                )
            }
            if new.shadows != old.shadows {
                self.program.set_uniform_ui("shadows", new.shadows as u32)
            }
        }
        self.program.unset_used();
        self.attributes = new;
    }

    pub fn reset_all_attributes(&self) {
        self.program.set_used();
        let att = &self.attributes;
        unsafe {
            self.program
                .set_uniform_matrix4("projection_matrix", &att.projection_matrix);

            self.program
                .set_uniform_3f_na("camera_position", att.camera_position);
            self.program.set_uniform_3f_na("color", att.color);
            self.program.set_uniform_f("model_size", att.model_size);
            self.program
                .set_uniform_f("distance_shading_power", att.distance_shading_power);
            self.program
                .set_uniform_f("distance_shading_constrict", att.distance_shading_constrict);
            self.program.set_uniform_f("toon_factor", att.toon_factor);
            self.program.set_uniform_ui(
                "distance_shading_channel",
                att.distance_shading_channel as u32,
            );
            self.program.set_uniform_ui("shadows", att.shadows as u32)
        }
        self.program.unset_used();
    }

    pub fn get_size(&self) -> &na::Vector3<f32> {
        &self.size
    }

    pub fn render(&self, viewport: &Viewport) {
        unsafe {
            gl::Disable(gl::CULL_FACE);
            gl::Disable(gl::BLEND);
            gl::Enable(gl::DEPTH_TEST);
            gl::DepthFunc(gl::LESS);
            self.shadow_program.set_used();

            // Set up light perspective
            let near_plane = 1.0;
            let far_plane = 500.0;
            let bound = 250.0;
            let light_projection =
                na::Orthographic3::new(-bound, bound, -bound, bound, near_plane, far_plane);

            let light_pos = match self.attributes.shadows_follow {
                true => self.attributes.camera_position,
                false => self.attributes.light_position,
            };
            let light = light_pos.normalize() * self.attributes.camera_position.magnitude();

            let cycle_speed_ms = 2000.0;
            let degrees =
                (self.attributes.elapsed % cycle_speed_ms) / cycle_speed_ms * std::f32::consts::TAU;
            let axis = na::Unit::new_normalize(light);
            let rotation = na::Matrix4::from_axis_angle(&axis, degrees);

            let horizontal = na::Vector3::new(0.0, 1.0, 0.0).cross(&light);
            let up_vector =
                horizontal.cross(&light).normalize() * self.attributes.shadows_orbit_radius;

            let light = (rotation * (light + up_vector).to_homogeneous()).xyz();

            let center = na::Point3::new(0.0, 0.0, 0.0);
            let light_view = na::Matrix4::look_at_rh(
                &na::Point3::from(light),
                &center,
                &na::Vector3::new(0.0, 1.0, 0.0),
            );
            let light_vector = center - light;
            let light_space_matrix = light_projection.to_homogeneous() * light_view;
            self.shadow_program
                .set_uniform_matrix4("lightSpaceMatrix", &light_space_matrix);

            // Render shadow map.
            gl::Viewport(0, 0, SHADOW_WIDTH, SHADOW_HEIGHT);
            self.depth_map_fbo.bind();
            gl::Clear(gl::DEPTH_BUFFER_BIT);

            self.vao.bind();
            self.ibo.bind();
            gl::DrawElements(
                gl::TRIANGLES,
                self.indices,
                gl::UNSIGNED_INT,
                std::ptr::null::<std::ffi::c_void>(),
            );
            self.depth_map_fbo.unbind();

            // Main render of model using shadows.
            self.program.set_used();
            self.program
                .set_uniform_matrix4("light_space_matrix", &light_space_matrix);
            self.program.set_uniform_3f(
                "light_vector",
                (light_vector[0], light_vector[1], light_vector[2]),
            );
            gl::Enable(gl::CULL_FACE);
            gl::CullFace(gl::BACK);
            viewport.set_used();

            self.depth_map.bind();
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
                    self.program = program;
                    self.reset_all_attributes();
                    return true;
                }
                Err(e) => eprintln!("Shader reload error: {}", e),
            }
        }
        false
    }
}
