#![allow(clippy::new_without_default)]

use gl;

pub type ArrayBuffer = Buffer<{ gl::ARRAY_BUFFER }>;
pub type ElementArrayBuffer = Buffer<{ gl::ELEMENT_ARRAY_BUFFER }>;

pub struct Buffer<const T: gl::types::GLuint> {
    vbo: gl::types::GLuint,
}

impl<const T: gl::types::GLuint> Buffer<T> {
    pub fn new() -> Buffer<T> {
        let mut vbo: gl::types::GLuint = 0;
        unsafe {
            gl::GenBuffers(1, &mut vbo);
        }

        Buffer { vbo }
    }

    pub fn bind(&self) {
        unsafe {
            gl::BindBuffer(T, self.vbo);
        }
    }

    pub fn unbind(&self) {
        unsafe {
            gl::BindBuffer(T, 0);
        }
    }

    pub fn static_draw_data<S>(&self, data: &[S]) {
        self.draw_data(data, gl::STATIC_DRAW);
    }

    pub fn dynamic_draw_data<S>(&self, data: &[S]) {
        self.draw_data(data, gl::DYNAMIC_DRAW);
    }

    fn draw_data<S>(&self, data: &[S], usage: gl::types::GLenum) {
        unsafe {
            gl::BufferData(
                T,
                (data.len() * ::std::mem::size_of::<S>()) as gl::types::GLsizeiptr,
                data.as_ptr() as *const gl::types::GLvoid,
                usage,
            );
        }
    }
}

impl<const T: gl::types::GLuint> Drop for Buffer<T> {
    fn drop(&mut self) {
        self.unbind();
        unsafe {
            gl::DeleteBuffers(1, &self.vbo);
        }
    }
}

pub struct VertexArray {
    vao: gl::types::GLuint,
}

impl VertexArray {
    pub fn new() -> Self {
        let mut vao: gl::types::GLuint = 0;
        unsafe {
            gl::GenVertexArrays(1, &mut vao);
        }

        Self { vao }
    }

    pub fn bind(&self) {
        unsafe {
            gl::BindVertexArray(self.vao);
        }
    }

    pub fn unbind(&self) {
        unsafe {
            gl::BindVertexArray(0);
        }
    }
}

impl Drop for VertexArray {
    fn drop(&mut self) {
        self.unbind();
        unsafe {
            gl::DeleteVertexArrays(1, &self.vao);
        }
    }
}

pub struct Texture {
    texture_id: gl::types::GLuint,
}

impl Texture {
    pub fn new() -> Self {
        let mut texture_id: gl::types::GLuint = 0;
        unsafe {
            gl::GenTextures(1, &mut texture_id);
        }

        Self { texture_id }
    }

    pub fn load_texture(&self, width: i32, height: i32, pixels: &[u8]) {
        self.bind();
        unsafe {
            gl::TexImage2D(
                gl::TEXTURE_2D, // Target
                0,              // Level-of-detail number. 0 for no mip-map
                gl::RGB as i32,
                width,
                height,
                0, // Must be zero lol.
                gl::SRGB8_ALPHA8,
                gl::UNSIGNED_BYTE,
                pixels.as_ptr() as *const std::ffi::c_void,
            );

            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
        }
    }

    pub fn bind(&self) {
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, self.texture_id);
        }
    }

    pub fn unbind(&self) {
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, 0);
        }
    }
}

impl Drop for Texture {
    fn drop(&mut self) {
        unsafe { gl::DeleteTextures(1, &self.texture_id) }
    }
}
