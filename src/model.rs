use crate::{
    render_gl::{
        self, buffer,
        data::{self, f32_f32_f32},
        Viewport,
    },
    resources::Resources,
};
use anyhow::{Context, Result};
use nalgebra as na;
use nalgebra_glm as glm;
use render_gl_derive::VertexAttribPointers;

const SHADER_PATH: &str = "shaders/model";
const SHADER_NAME: &str = "model";
const SHADOW_WIDTH: gl::types::GLsizei = 1024;
const SHADOW_HEIGHT: gl::types::GLsizei = 1024;

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
    depth_map: u32,
    depth_map_fbo: u32,
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
            program.set_uniform_ui("shadows", 1);
            program.unset_used();
            render_gl::check_gl_error();
        }

        // Shadowstuff
        let shadow_program;
        let mut depth_map = 0;
        let mut depth_map_fbo = 0;
        unsafe {
            shadow_program = render_gl::Program::from_res(res, "shaders/shadow")?;
            gl::GenFramebuffers(1, &mut depth_map_fbo);

            gl::GenTextures(1, &mut depth_map);
            gl::BindTexture(gl::TEXTURE_2D, depth_map);
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::DEPTH_COMPONENT as gl::types::GLint,
                SHADOW_WIDTH,
                SHADOW_HEIGHT,
                0,
                gl::DEPTH_COMPONENT,
                gl::FLOAT,
                0 as *const std::ffi::c_void,
            );
            #[rustfmt::skip]
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as gl::types::GLint);
            #[rustfmt::skip]
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as gl::types::GLint);
            #[rustfmt::skip]
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_BORDER as gl::types::GLint);
            #[rustfmt::skip]
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_BORDER as gl::types::GLint);
            let border_color = [1.0, 1.0, 1.0, 1.0];
            gl::TexParameterfv(
                gl::TEXTURE_2D,
                gl::TEXTURE_BORDER_COLOR,
                border_color.as_ptr(),
            );

            gl::BindFramebuffer(gl::FRAMEBUFFER, depth_map_fbo);
            #[rustfmt::skip]
            gl::FramebufferTexture2D(gl::FRAMEBUFFER, gl::DEPTH_ATTACHMENT, gl::TEXTURE_2D, depth_map, 0);
            gl::DrawBuffer(gl::NONE);
            gl::ReadBuffer(gl::NONE);

            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
        }

        Ok(Self {
            program,
            shadow_program,
            _vbo: vbo,
            vao,
            ibo,
            indices: model.indices.len() as i32,
            size: max - min,
            attributes: Default::default(),
            depth_map,
            depth_map_fbo,
        })
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
                    .set_uniform_matrix4("projection_matrix", new.projection_matrix);
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

    pub fn get_size(&self) -> &na::Vector3<f32> {
        &self.size
    }

    pub fn render(&self, viewport: &Viewport) {
        let light_space_matrix;
        let lv;
        unsafe {
            gl::Disable(gl::CULL_FACE);
            // gl::CullFace(gl::FRONT);

            gl::Disable(gl::BLEND);
            gl::Enable(gl::DEPTH_TEST);
            gl::DepthFunc(gl::LESS);
            // Configure Shader and Matrices
            self.shadow_program.set_used();
            let near_plane = 0.01;
            let far_plane = 1000.0;
            let light_projection = glm::ortho(-200.0, 200.0, -200.0, 200.0, near_plane, far_plane);
            let light = self.attributes.light_position;
            let center = glm::vec3(0.0, 0.0, 0.0);
            let light_view = glm::look_at(&light, &center, &glm::vec3(0.0, 1.0, 0.0));
            lv = center - light;
            light_space_matrix = light_projection * light_view;
            let light_space_matrix_uloc =
                self.shadow_program.get_uniform_location("lightSpaceMatrix");
            gl::UniformMatrix4fv(
                light_space_matrix_uloc,
                1,
                gl::FALSE,
                light_space_matrix.as_ptr(),
            );

            gl::Viewport(0, 0, SHADOW_WIDTH, SHADOW_HEIGHT);
            gl::BindFramebuffer(gl::FRAMEBUFFER, self.depth_map_fbo);
            gl::Clear(gl::DEPTH_BUFFER_BIT);

            // RENDER SCENE
            self.vao.bind();
            self.ibo.bind();
            gl::DrawElements(
                gl::TRIANGLES,
                self.indices,
                gl::UNSIGNED_INT,
                std::ptr::null::<std::ffi::c_void>(),
            );
            self.ibo.unbind();
            self.vao.unbind();

            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
        }

        self.program.set_used();
        self.vao.bind();
        self.ibo.bind();

        unsafe {
            self.program
                .set_uniform_matrix4glm("light_space_matrix", &light_space_matrix);
            self.program
                .set_uniform_3f("light_vector", (lv[0], lv[1], lv[2]));
            gl::Disable(gl::BLEND);
            gl::Enable(gl::CULL_FACE);
            gl::CullFace(gl::BACK);
            gl::Enable(gl::DEPTH_TEST);
            gl::DepthFunc(gl::LESS);
            viewport.set_used();

            gl::BindTexture(gl::TEXTURE_2D, self.depth_map);
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
                        program.set_uniform_3f_na("camera_position", attr.camera_position);
                        program.set_uniform_3f_na("color", attr.color);
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
                        );
                        self.program.set_uniform_ui("shadows", attr.shadows as u32);
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
