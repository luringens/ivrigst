//! Contains the [Camera] struct and related constants.

use core::f32;
use na::Point3;
use nalgebra as na;

/// Avoids a camera distance of zero, messing up math elsewhere.
const MIN_ZOOM: f32 = 1.0;
/// Reasonable far plane distance.
const MAX_ZOOM: f32 = 400.0;

pub struct Camera {
    fov: f32,
    mousedown: bool,
    roll: f32,
    pitch: f32,
    dist: f32,
}

#[allow(clippy::new_without_default)]
impl Camera {
    /// Initialize a new [Camera] with default values.
    pub fn new() -> Self {
        Self {
            fov: f32::consts::PI / 4.0,
            mousedown: false,
            roll: 0.0,
            pitch: f32::consts::PI / 4.0,
            dist: 5.0,
        }
    }

    /// Returns the position of the camera.
    pub fn position(&self) -> Point3<f32> {
        let rot = na::Rotation3::from_euler_angles(self.roll, self.pitch, 0.0);
        let rotated = rot * na::Vector3::z();
        Point3::from(rotated) * self.dist
    }

    /// Constructs a model-view-projection matrix using the camera.
    pub fn construct_mvp(&self, aspect: f32, model: na::Isometry3<f32>) -> na::Matrix4<f32> {
        let eye = self.position();
        let target = na::Point3::new(0.0, 0.0, 0.0);
        let view = na::Isometry3::look_at_rh(&eye, &target, &na::Vector3::y());
        let projection = na::Perspective3::new(aspect, self.fov, 0.1, 1000.0);
        projection.into_inner() * (view * model).to_homogeneous()
    }

    /// Informs the camera that the mouse button is held down, to enable camera movement.
    pub fn mousedown(&mut self) {
        self.mousedown = true;
    }

    /// Informs the camera that the mouse button is no longer being held down, to disable camera
    /// movement.
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

    /// Informs the camera that the mousewheel has been scrolled, to enable camera zoom.
    pub fn mousewheel(&mut self, y: i32) {
        self.dist = (self.dist - 3.0 * y as f32).clamp(MIN_ZOOM, MAX_ZOOM);
    }

    /// Manually sets the camera distance from origin.
    pub fn set_dist(&mut self, dist: f32) {
        self.dist = dist.clamp(MIN_ZOOM, MAX_ZOOM);
    }
}
