use gl;
use nalgebra as na;

pub struct ColorBuffer {
    pub color: na::Vector4<f32>,
}

impl ColorBuffer {
    pub fn from_color(color: na::Vector3<f32>) -> ColorBuffer {
        ColorBuffer {
            color: color.fixed_resize::<4, 1>(1.0),
        }
    }

    pub fn update_color(&mut self, color: na::Vector3<f32>) {
        self.color = color.fixed_resize::<4, 1>(1.0);
    }

    pub fn set_used(&self) {
        unsafe {
            gl::ClearColor(self.color.x, self.color.y, self.color.z, 1.0);
        }
    }

    pub fn clear(&self) {
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }
    }
}
