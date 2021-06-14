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
        unsafe {
            gl::BufferData(
                T,
                (data.len() * ::std::mem::size_of::<S>()) as gl::types::GLsizeiptr,
                data.as_ptr() as *const gl::types::GLvoid,
                gl::STATIC_DRAW,
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
    pub fn new() -> VertexArray {
        let mut vao: gl::types::GLuint = 0;
        unsafe {
            gl::GenVertexArrays(1, &mut vao);
        }

        VertexArray { vao }
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
