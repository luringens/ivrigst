use core::f32;

use na::Point3;
use nalgebra as na;

pub struct Camera {
    fov: f32,
    mousedown: bool,
    roll: f32,
    pitch: f32,
    dist: f32,
}

#[allow(clippy::new_without_default)]
impl Camera {
    pub fn new() -> Self {
        Self {
            fov: f32::consts::PI / 4.0,
            mousedown: false,
            roll: 0.0,
            pitch: f32::consts::PI / 4.0,
            dist: 5.0,
        }
    }

    pub fn position(&self) -> Point3<f32> {
        let rot = na::Rotation3::from_euler_angles(self.roll, self.pitch, 0.0);
        let rotated = rot * na::Vector3::z();
        Point3::from(rotated) * self.dist
    }

    pub fn construct_mvp(&self, aspect: f32, model: na::Isometry3<f32>) -> na::Matrix4<f32> {
        let eye = self.position();
        let target = na::Point3::new(0.0, 0.0, 0.0);
        let view = na::Isometry3::look_at_rh(&eye, &target, &na::Vector3::y());
        let projection = na::Perspective3::new(aspect, self.fov, 0.1, 1000.0);
        projection.into_inner() * (view * model).to_homogeneous()
    }

    pub fn mousedown(&mut self) {
        self.mousedown = true;
    }

    pub fn mouseup(&mut self) {
        self.mousedown = false;
    }

    /// Mouse movement handler. Returns true if camera view has changed.
    pub fn mousemove(&mut self, xrel: i32, yrel: i32) -> bool {
        if !self.mousedown {
            return false;
        }

        self.pitch = (self.pitch + f32::consts::TAU / 500.0 * -xrel as f32) % f32::consts::TAU;
        self.roll = (self.roll + f32::consts::TAU / 500.0 * -yrel as f32).clamp(
            -f32::consts::PI / 2.0 + 0.001,
            f32::consts::PI / 2.0 - 0.001,
        );

        true
    }

    pub fn mousewheel(&mut self, y: i32) {
        self.dist = (self.dist - 2.0 * y as f32).clamp(0.0, f32::MAX);
    }

    pub fn set_dist(&mut self, dist: f32) {
        self.dist = dist;
    }
}
