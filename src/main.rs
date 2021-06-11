mod camera;
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

    let mut triangle = Triangle::new(&res).expect("Failed to set up triangle.");

    // set up shared state for window
    let mut viewport =
        render_gl::Viewport::for_window(window.size().0 as i32, window.size().1 as i32);
    viewport.set_used();
    let color_buffer = render_gl::ColorBuffer::from_color(na::Vector3::new(0.3, 0.3, 0.5));
    color_buffer.set_used();

    // Camera and projection
    let model_isometry = na::Isometry3::new(na::Vector3::zeros(), na::zero());
    let eye = na::Point3::new(3., 0., 3.);
    let target = na::Point3::new(0., 0., 0.);
    let fov = 3.14 / 4.0; // 45 degrees in radians
    let camera = camera::Camera::new(eye, target, fov);

    render_gl::check_gl_error();

    let mut mvp_needs_update = true;
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
                    mvp_needs_update = true;
                }
                _ => {}
            }
        }

        if mvp_needs_update {
            let aspect = window.size().1 as f32 / window.size().0 as f32;
            let model_view_projection = camera.construct_mvp(aspect, model_isometry);
            let shader = triangle.shader();
            shader.set_used();
            unsafe {
                shader.set_uniform_matrix4("ProjectionMatrix", model_view_projection);
            }
            mvp_needs_update = false;
        }

        color_buffer.clear();
        triangle.render();
        window.gl_swap_window();
        render_gl::check_gl_error();
    }
}
