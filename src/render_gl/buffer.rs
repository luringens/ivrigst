//! Contains structs wrapping OpenGL buffers.

#![allow(clippy::new_without_default)]

use core::panic;

/// Represents an OpenGL array buffer.
pub type ArrayBuffer = Buffer<{ gl::ARRAY_BUFFER }>;

/// Represents an OpenGL element array buffer.
pub type ElementArrayBuffer = Buffer<{ gl::ELEMENT_ARRAY_BUFFER }>;

/// Generic buffer intended to handle both array buffers and element array buffers.
pub struct Buffer<const T: gl::types::GLuint> {
    vbo: gl::types::GLuint,
}

impl<const T: gl::types::GLuint> Buffer<T> {
    /// Genereate this buffer.
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

    /// Uploads data to the buffer, informing the driver that this is static data.
    pub fn static_draw_data<S>(&self, data: &[S]) {
        self.draw_data(data, gl::STATIC_DRAW);
    }

    /// Uploads data to the buffer, informing the driver that this is dynamic data.
    pub fn dynamic_draw_data<S>(&self, data: &[S]) {
        self.draw_data(data, gl::DYNAMIC_DRAW);
    }

    /// Uploads data to the buffer with the given usage hint.
    fn draw_data<S>(&self, data: &[S], usage: gl::types::GLenum) {
        self.bind();
        // Safety: the size of the data **MUST** be correct.
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

/// OpenGL vertex array wrapper.
pub struct VertexArray {
    vao: gl::types::GLuint,
}

impl VertexArray {
    /// Generates a new vertex array.
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

/// OpenGL texture wrapper.
pub struct Texture {
    pub texture_id: gl::types::GLuint,
    texture_unit: gl::types::GLuint,
}

impl Texture {
    /// Generates a new texture unit.
    pub fn new(texture_unit: gl::types::GLenum) -> Self {
        let mut texture_id: gl::types::GLuint = 0;
        unsafe {
            gl::GenTextures(1, &mut texture_id);
        }

        Self {
            texture_id,
            texture_unit,
        }
    }

    /// Loads given texture data into this texture. If `pixels` is `None`, the texture memory is
    /// allocated but left unassigned.
    pub fn load_texture(
        &self,
        dimensions: (i32, i32),
        pixels: Option<&[u8]>,
        internal_format: gl::types::GLint,
        format: gl::types::GLenum,
        data_type: gl::types::GLenum,
        repeat: bool,
    ) {
        assert!(dimensions.0 >= 0);
        assert!(dimensions.1 >= 0);
        self.bind();
        let pixels_pointer = if let Some(pixels) = pixels {
            // Check correct size of data and dimensions.
            let pixel_size = match format {
                gl::RGBA => 4,
                _ => panic!("Unhandled pixel type."),
            };
            assert_eq!(
                pixels.len(),
                (dimensions.0 * dimensions.1) as usize * pixel_size,
                "Inconsistent texture size!"
            );

            pixels.as_ptr() as *const std::ffi::c_void
        } else {
            std::ptr::null() as *const std::ffi::c_void
        };

        // Safety: the size of the data **MUST** be correct, as checked above.
        unsafe {
            gl::TexImage2D(
                gl::TEXTURE_2D, // Target
                0,              // Level-of-detail number. 0 for no mip-map
                internal_format,
                dimensions.0,
                dimensions.1,
                0, // Docs declare that this argument must be zero.
                format,
                data_type,
                pixels_pointer,
            );
        }

        let param = match repeat {
            true => gl::REPEAT,
            false => gl::CLAMP_TO_BORDER,
        };

        // Safety: No particular requirements, beyond enums should be valid or else OpenGL will log
        // an error.
        unsafe {
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, param as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, param as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
        }
    }

    /// Sets the border color of the texture.
    pub fn set_border_color(&self, border_color: &[f32; 4]) {
        unsafe {
            gl::TexParameterfv(
                gl::TEXTURE_2D,
                gl::TEXTURE_BORDER_COLOR,
                border_color.as_ptr(),
            );
        }
    }

    pub fn bind(&self) {
        unsafe {
            gl::ActiveTexture(self.texture_unit);
            gl::BindTexture(gl::TEXTURE_2D, self.texture_id);
        }
    }

    /// Sets the texture compare mode for the texture.
    pub fn set_texture_compare_mode(&self, mode: gl::types::GLenum) {
        self.bind();
        unsafe {
            gl::TexParameteri(
                gl::TEXTURE_2D,
                gl::TEXTURE_COMPARE_MODE,
                mode as gl::types::GLint,
            );
        }
    }

    pub fn bind_to(&self, texture_unit: gl::types::GLenum) {
        unsafe {
            gl::ActiveTexture(texture_unit);
            gl::BindTexture(gl::TEXTURE_2D, self.texture_id);
        }
    }

    pub fn unbind(&self) {
        unsafe {
            gl::ActiveTexture(self.texture_unit);
            gl::BindTexture(gl::TEXTURE_2D, 0);
        }
    }
}

impl Drop for Texture {
    fn drop(&mut self) {
        unsafe { gl::DeleteTextures(1, &self.texture_id) }
    }
}

/// Represents an OpenGL framebuffer object.
pub struct FrameBuffer {
    fbo: gl::types::GLuint,
}

impl FrameBuffer {
    /// Generate a new framebuffer.
    pub fn new() -> Self {
        let mut fbo: gl::types::GLuint = 0;
        unsafe {
            gl::GenFramebuffers(1, &mut fbo);
        }

        Self { fbo }
    }

    /// Set the read- and write-types for this buffer.
    pub fn set_type(&self, draw_type: gl::types::GLenum, read_type: gl::types::GLenum) {
        self.bind();
        unsafe {
            gl::DrawBuffer(draw_type);
            gl::ReadBuffer(read_type);
        }
    }

    /// Bind a texture as the target for this buffer.
    pub fn bind_texture(&self, attachment: gl::types::GLenum, texture: &Texture) {
        self.bind();
        unsafe {
            gl::FramebufferTexture2D(
                gl::FRAMEBUFFER,
                attachment,
                gl::TEXTURE_2D,
                texture.texture_id,
                0,
            );
        }
    }

    pub fn bind(&self) {
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, self.fbo);
        }
    }

    pub fn unbind(&self) {
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
        }
    }
}

impl Drop for FrameBuffer {
    fn drop(&mut self) {
        self.unbind();
        unsafe {
            gl::DeleteFramebuffers(1, &self.fbo);
        }
    }
}
