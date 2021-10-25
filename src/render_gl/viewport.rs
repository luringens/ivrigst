//! Module containing the [Viewport] struct.

/// Manages the size of the OpenGL viewport.
#[derive(Debug)]
pub struct Viewport {
    pub x: i32,
    pub y: i32,
    pub w: u32,
    pub h: u32,
}

impl Viewport {
    /// Initialize [Viewport] with given size.
    pub fn for_window(w: i32, h: i32) -> Viewport {
        assert!(w >= 0);
        assert!(h >= 0);
        Viewport {
            x: 0,
            y: 0,
            w: w as u32,
            h: h as u32,
        }
    }

    /// Update desired size for the viewport.
    pub fn update_size(&mut self, w: i32, h: i32) {
        assert!(w >= 0);
        assert!(h >= 0);
        self.w = w as u32;
        self.h = h as u32;
    }

    /// Get current viewport size.
    pub fn size(&self) -> (u32, u32) {
        (self.w, self.h)
    }

    /// Ask GL to use the currently set viewport size.
    pub fn set_used(&self) {
        // Safety: requires that `w` and `h` are not negative.
        unsafe {
            gl::Viewport(self.x, self.y, self.w as i32, self.h as i32);
        }
    }
}
