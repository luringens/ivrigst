use bevy::{
    math::vec3,
    prelude::*,
    reflect::TypeUuid,
    render::{renderer::RenderResources, shader::ShaderDefs},
};

#[derive(RenderResources, ShaderDefs, TypeUuid)]
#[uuid = "0805ae06-bfbc-4e78-86bb-c1a4f143c6ad"]
pub struct MyMaterial {
    camera_position: Vec3,
    color: Vec3,
    model_size: f32,
    distance_shading_power: f32,
    distance_shading_constrict: f32,
    toon_factor: f32,

    #[render_resources(ignore)]
    #[shader_def]
    distance_shading_channel_hue: bool,

    #[render_resources(ignore)]
    #[shader_def]
    distance_shading_channel_saturation: bool,

    #[render_resources(ignore)]
    #[shader_def]
    distance_shading_channel_value: bool,

    /// Carefully laid out by struct to match shader expectation.
    floats: Vec<f32>,
    /// Carefully laid out by struct to match shader expectation.
    vectors: Vec<f32>,
}

#[derive(Debug, Eq, PartialEq)]
pub enum DistanceShadingChannel {
    Hue,
    Saturation,
    Value,
    None,
}

impl Default for MyMaterial {
    fn default() -> Self {
        let mut material = Self {
            distance_shading_power: f32::default(),
            model_size: f32::default(),
            camera_position: Vec3::default(),
            color: Vec3::default(),
            vectors: vec![f32::default(); 7],
            floats: vec![f32::default(); 4],
            distance_shading_channel_hue: false,
            distance_shading_channel_saturation: false,
            distance_shading_channel_value: true,
            distance_shading_constrict: 0.20,
            toon_factor: 0.8,
        };
        material.set_color(vec3(1.0, 0.56, 0.72));
        material.set_distance_shading_power(0.8);
        material
    }
}

impl MyMaterial {
    pub fn set_camera_position(&mut self, new: Vec3) {
        self.camera_position = new;
        self.vectors[0] = new.x;
        self.vectors[1] = new.y;
        self.vectors[2] = new.z;
    }

    pub fn set_color(&mut self, new: Vec3) {
        self.color = new;
        self.vectors[4] = new.x;
        self.vectors[5] = new.y;
        self.vectors[6] = new.z;
    }

    pub fn set_model_size(&mut self, new: f32) {
        self.floats[0] = new;
    }

    pub fn set_distance_shading_power(&mut self, new: f32) {
        self.distance_shading_power = new;
        self.floats[1] = new;
    }

    pub fn set_distance_shading_constrict(&mut self, new: f32) {
        self.distance_shading_constrict = new;
        self.floats[2] = new;
    }

    pub fn set_toon_factor(&mut self, new: f32) {
        self.toon_factor = new;
        self.floats[3] = new;
    }

    pub fn set_distance_shading_channel(&mut self, channel: DistanceShadingChannel) {
        self.distance_shading_channel_hue = false;
        self.distance_shading_channel_saturation = false;
        self.distance_shading_channel_value = false;
        match channel {
            DistanceShadingChannel::Hue => self.distance_shading_channel_hue = true,
            DistanceShadingChannel::Saturation => self.distance_shading_channel_saturation = true,
            DistanceShadingChannel::Value => self.distance_shading_channel_value = true,
            DistanceShadingChannel::None => {}
        };
    }

    pub fn get_color(&self) -> Vec3 {
        self.color
    }

    pub fn get_distance_shading_power(&self) -> f32 {
        self.distance_shading_power
    }

    pub fn get_distance_shading_constrict(&self) -> f32 {
        self.distance_shading_constrict
    }

    pub fn get_toon_factor(&self) -> f32 {
        self.toon_factor
    }

    pub fn get_distance_shading_channel(&self) -> DistanceShadingChannel {
        if self.distance_shading_channel_hue {
            return DistanceShadingChannel::Hue;
        }
        if self.distance_shading_channel_saturation {
            return DistanceShadingChannel::Saturation;
        }
        if self.distance_shading_channel_value {
            return DistanceShadingChannel::Value;
        }
        DistanceShadingChannel::None
    }
}
