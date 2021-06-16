pub mod buffer;
mod color_buffer;
pub mod data;
mod shader;
mod viewport;

pub use self::color_buffer::ColorBuffer;
pub use self::shader::{Program, Shader};
pub use self::viewport::Viewport;

/// Checks and prints OpenGL errors when compiled in debug mode.
pub fn check_gl_error() {
    if cfg!(debug_assertions) {
        let mut crash = false;
        loop {
            let err;
            unsafe {
                err = gl::GetError();
            }
            if err == gl::NO_ERROR {
                break;
            }

            eprintln!("OpenGL Error:");
            eprintln!("{}", get_gl_error_string(err));
            crash = true;
        }
        if crash {
            panic!("Ending appliation");
        }
    }
}

fn get_gl_error_string(error_id: u32) -> &'static str {
    match error_id {
        gl::NO_ERROR => "No error?",
        gl::INVALID_ENUM => "Invalid enum",
        gl::INVALID_VALUE => "Invalid value",
        gl::INVALID_OPERATION => "Invalid operation",
        gl::STACK_OVERFLOW => "Stack overflow",
        gl::STACK_UNDERFLOW => "Stack underflow",
        gl::OUT_OF_MEMORY => "Out of memory",
        _ => "UNKNWN",
    }
}
