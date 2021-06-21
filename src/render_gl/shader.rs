use crate::resources::Resources;
use anyhow::{anyhow, Context, Result};
use nalgebra as na;
use std::ffi::{CStr, CString};

pub struct Program {
    id: gl::types::GLuint,
}

impl Program {
    pub fn from_res(res: &Resources, name: &str) -> Result<Program> {
        const POSSIBLE_EXT: [&str; 2] = [".vert", ".frag"];

        let shaders = POSSIBLE_EXT
            .iter()
            .map(|file_extension| Shader::from_res(res, &format!("{}{}", name, file_extension)))
            .collect::<Result<Vec<Shader>>>()?;

        Program::from_shaders(&shaders[..])
    }

    pub fn from_shaders(shaders: &[Shader]) -> Result<Program> {
        let program_id = unsafe { gl::CreateProgram() };

        for shader in shaders {
            unsafe {
                gl::AttachShader(program_id, shader.id());
            }
        }

        unsafe {
            gl::LinkProgram(program_id);
        }

        let mut success: gl::types::GLint = 1;
        unsafe {
            gl::GetProgramiv(program_id, gl::LINK_STATUS, &mut success);
        }

        if success == 0 {
            let mut len: gl::types::GLint = 0;
            unsafe {
                gl::GetProgramiv(program_id, gl::INFO_LOG_LENGTH, &mut len);
            }

            let error = create_whitespace_cstring_with_len(len as usize);

            unsafe {
                gl::GetProgramInfoLog(
                    program_id,
                    len,
                    std::ptr::null_mut(),
                    error.as_ptr() as *mut gl::types::GLchar,
                );
            }

            return Err(anyhow!(error.to_string_lossy().into_owned()));
        }

        for shader in shaders {
            unsafe {
                gl::DetachShader(program_id, shader.id());
            }
        }

        Ok(Program { id: program_id })
    }

    pub unsafe fn set_uniform_matrix4glm(&self, uniform_id: &str, data: &nalgebra_glm::Mat4) {
        let uniform_location = self.get_uniform_location(uniform_id);
        let data = data.as_slice();
        gl::UniformMatrix4fv(uniform_location, 1, gl::FALSE, data.as_ptr());
    }

    pub unsafe fn set_uniform_matrix4(&self, uniform_id: &str, data: na::Matrix4<f32>) {
        let uniform_location = self.get_uniform_location(uniform_id);
        let data = data.as_slice();
        gl::UniformMatrix4fv(uniform_location, 1, gl::FALSE, data.as_ptr());
    }

    pub unsafe fn set_uniform_3f_na(&self, uniform_id: &str, v: na::Vector3<f32>) {
        self.set_uniform_3f(uniform_id, (v[0], v[1], v[2]))
    }

    pub unsafe fn set_uniform_3f_arr(&self, uniform_id: &str, [f0, f1, f2]: [f32; 3]) {
        self.set_uniform_3f(uniform_id, (f0, f1, f2))
    }

    pub unsafe fn set_uniform_3f(&self, uniform_id: &str, (f0, f1, f2): (f32, f32, f32)) {
        let uniform_location = self.get_uniform_location(uniform_id);
        gl::Uniform3f(uniform_location, f0, f1, f2);
    }

    pub unsafe fn set_uniform_2f(&self, uniform_id: &str, data: (f32, f32)) {
        let uniform_location = self.get_uniform_location(uniform_id);
        gl::Uniform2f(uniform_location, data.0, data.1);
    }

    pub unsafe fn set_uniform_f(&self, uniform_id: &str, data: f32) {
        let uniform_location = self.get_uniform_location(uniform_id);
        gl::Uniform1f(uniform_location, data);
    }

    pub unsafe fn set_uniform_ui(&self, uniform_id: &str, data: u32) {
        let uniform_location = self.get_uniform_location(uniform_id);
        gl::Uniform1ui(uniform_location, data);
    }

    pub unsafe fn get_uniform_location(&self, uniform_id: &str) -> gl::types::GLint {
        let uniform_id = CString::new(uniform_id).expect("Invalid uniform_id.");
        gl::GetUniformLocation(self.id, uniform_id.as_ptr())
    }

    pub fn unset_used(&self) {
        unsafe {
            gl::UseProgram(0);
        }
    }

    pub fn set_used(&self) {
        unsafe {
            gl::UseProgram(self.id);
        }
    }

    pub fn id(&self) -> gl::types::GLuint {
        self.id
    }
}

impl Drop for Program {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteProgram(self.id);
        }
    }
}

pub struct Shader {
    id: gl::types::GLuint,
}

impl Shader {
    pub fn from_res(res: &Resources, name: &str) -> Result<Shader> {
        const POSSIBLE_EXT: [(&str, gl::types::GLenum); 2] =
            [(".vert", gl::VERTEX_SHADER), (".frag", gl::FRAGMENT_SHADER)];

        let shader_kind = POSSIBLE_EXT
            .iter()
            .find(|&&(file_extension, _)| name.ends_with(file_extension))
            .map(|&(_, kind)| kind)
            .context(format!(
                "Can not determine shader type for resource {}",
                name
            ))?;

        let source = res
            .load_cstring(name)
            .context(format!("Error loading resource {}", name))?;

        Shader::from_source(&source, shader_kind)
    }

    pub fn from_source(source: &CStr, kind: gl::types::GLenum) -> Result<Shader> {
        let id = shader_from_source(source, kind)?;
        Ok(Shader { id })
    }

    pub fn from_vert_source(source: &CStr) -> Result<Shader> {
        Shader::from_source(source, gl::VERTEX_SHADER)
    }

    pub fn from_frag_source(source: &CStr) -> Result<Shader> {
        Shader::from_source(source, gl::FRAGMENT_SHADER)
    }

    pub fn id(&self) -> gl::types::GLuint {
        self.id
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteShader(self.id);
        }
    }
}

fn shader_from_source(source: &CStr, kind: gl::types::GLenum) -> Result<gl::types::GLuint> {
    let id = unsafe { gl::CreateShader(kind) };
    unsafe {
        gl::ShaderSource(id, 1, &source.as_ptr(), std::ptr::null());
        gl::CompileShader(id);
    }

    let mut success: gl::types::GLint = 1;
    unsafe {
        gl::GetShaderiv(id, gl::COMPILE_STATUS, &mut success);
    }

    if success == 0 {
        let mut len: gl::types::GLint = 0;
        unsafe {
            gl::GetShaderiv(id, gl::INFO_LOG_LENGTH, &mut len);
        }

        let error = create_whitespace_cstring_with_len(len as usize);

        unsafe {
            gl::GetShaderInfoLog(
                id,
                len,
                std::ptr::null_mut(),
                error.as_ptr() as *mut gl::types::GLchar,
            );
        }

        return Err(anyhow!(error.to_string_lossy().into_owned()));
    }

    Ok(id)
}

fn create_whitespace_cstring_with_len(len: usize) -> CString {
    // allocate buffer of correct size
    let mut buffer: Vec<u8> = Vec::with_capacity(len + 1);
    // fill it with len spaces
    buffer.extend([b' '].iter().cycle().take(len));
    // convert buffer to CString
    unsafe { CString::from_vec_unchecked(buffer) }
}
