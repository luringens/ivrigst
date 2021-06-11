pub mod render_gl;
pub mod resources;
mod triangle;

use crate::resources::Resources;
use nalgebra as na;
use std::path::Path;
use triangle::Triangle;

fn main() {
    let res = Resources::from_relative_exe_path(Path::new("assets")).unwrap();

    let sdl = sdl2::init().unwrap();
    let video_subsystem = sdl.video().unwrap();

    let gl_attr = video_subsystem.gl_attr();

    gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
    gl_attr.set_context_version(4, 1);

    let window = video_subsystem
        .window("MedVis", 900, 900)
        .opengl()
        .resizable()
        .build()
        .unwrap();

    let _gl_context = window.gl_create_context().unwrap();
    let _gl =
        gl::load_with(|s| video_subsystem.gl_get_proc_address(s) as *const std::os::raw::c_void);

    let triangle = Triangle::new(&res).expect("Failed to set up triangle.");

    // set up shared state for window
    let mut viewport =
        render_gl::Viewport::for_window(window.size().0 as i32, window.size().1 as i32);
    viewport.set_used();
    let color_buffer = render_gl::ColorBuffer::from_color(na::Vector3::new(0.3, 0.3, 0.5));
    color_buffer.set_used();

    // main loop

    let mut event_pump = sdl.event_pump().unwrap();
    'main: loop {
        for event in event_pump.poll_iter() {
            match event {
                sdl2::event::Event::Quit { .. } => break 'main,
                sdl2::event::Event::Window {
                    win_event: sdl2::event::WindowEvent::Resized(w, h),
                    ..
                } => {
                    viewport.update_size(w, h);
                    viewport.set_used();
                }
                _ => {}
            }
        }

        color_buffer.clear();
        triangle.render();
        window.gl_swap_window();
    }
}
