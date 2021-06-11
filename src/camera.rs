use core::f32;

use nalgebra as na;

pub struct Camera {
    eye: na::Point3<f32>,
    target: na::Point3<f32>,
    fov: f32,
}

impl Camera {
    pub fn new(eye: na::Point3<f32>, target: na::Point3<f32>, fov: f32) -> Self {
        Self { eye, target, fov }
    }

    pub fn construct_mvp(&self, aspect: f32, model: na::Isometry3<f32>) -> na::Matrix4<f32> {
        let view = na::Isometry3::look_at_rh(&self.eye, &self.target, &na::Vector3::y());
        let projection = na::Perspective3::new(aspect, self.fov, 0.1, 1000.0);
        projection.into_inner() * (view * model).to_homogeneous()
    }
}
