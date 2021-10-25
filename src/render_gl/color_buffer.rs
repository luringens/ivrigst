//! Contains [ColorBuffer].

use nalgebra as na;

/// Represents a colorbuffer and provides convenient methods to clear this buffer.
pub struct ColorBuffer {
    pub color: na::Vector4<f32>,
}

impl ColorBuffer {
    /// Initialize [ColorBuffer] with a given clear color.
    pub fn from_color(color: na::Vector3<f32>) -> ColorBuffer {
        let mut buffer = ColorBuffer {
            color: Default::default(),
        };

        buffer.update_color(color);
        buffer.clear();
        buffer
    }

    /// Set a new clear color.
    pub fn update_color(&mut self, color: na::Vector3<f32>) {
        assert!(0.0 <= color.x && color.x <= 1.0);
        assert!(0.0 <= color.y && color.y <= 1.0);
        assert!(0.0 <= color.z && color.z <= 1.0);
        self.color = color.fixed_resize::<4, 1>(1.0);

        // Correctness: No particular requirements. Correct range for values is 0.0 <= x <= 1.0 which
        // is checked above. However, values outside this range are clamped to this range anyways.
        unsafe {
            gl::ClearColor(self.color.x, self.color.y, self.color.z, 1.0);
        }
    }

    /// Clears the color buffer.
    pub fn clear(&self) {
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }
    }
}
