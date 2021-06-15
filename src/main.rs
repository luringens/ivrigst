mod camera;
mod model;
pub mod render_gl;
pub mod resources;
mod ui;

use crate::resources::Resources;
use model::Model;
use nalgebra as na;
use std::path::Path;
use ui::UI;

fn main() {
    let res =
        Resources::from_relative_exe_path(Path::new("assets")).expect("Failed to find assets");

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

    let mut model = Model::new(&res).expect("Failed to set up model.");
    let ui = UI::new(&res).expect("Failed to set up UI.");

    // set up shared state for window
    let mut viewport =
        render_gl::Viewport::for_window(window.size().0 as i32, window.size().1 as i32);
    viewport.set_used();
    let color_buffer = render_gl::ColorBuffer::from_color(na::Vector3::new(0.3, 0.3, 0.5));
    color_buffer.set_used();

    // Camera and projection
    let model_isometry = na::Isometry3::new(na::Vector3::zeros(), na::zero());
    let mut camera = camera::Camera::new();
    camera.set_dist(model.get_size().magnitude() * 1.2);

    unsafe {
        gl::Enable(gl::DEPTH_TEST);
        gl::DepthFunc(gl::LESS);
    }

    render_gl::check_gl_error();

    let mut ctx = egui::CtxRef::default();
    {
        let mut raw_input: egui::RawInput = Default::default();
        raw_input.screen_rect = Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(window.size().0 as f32, window.size().1 as f32),
        ));
    }

    let mut ui_texture_needs_update = true;
    let mut mvp_needs_update = true;
    let mut event_pump = sdl.event_pump().unwrap();
    'main: loop {
        let mut raw_input: egui::RawInput = Default::default();
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
                    raw_input.screen_rect = Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(w as f32, h as f32),
                    ));
                }
                sdl2::event::Event::MouseButtonDown {
                    mouse_btn, x, y, ..
                } => {
                    camera.mousedown();
                    raw_input.events.push(egui::Event::PointerButton {
                        pos: egui::pos2(x as f32, y as f32),
                        button: sdl2_to_egui_pointerbutton(mouse_btn),
                        pressed: true,
                        modifiers: Default::default(),
                    })
                }
                sdl2::event::Event::MouseButtonUp {
                    mouse_btn, x, y, ..
                } => {
                    camera.mouseup();
                    raw_input.events.push(egui::Event::PointerButton {
                        pos: egui::pos2(x as f32, y as f32),
                        button: sdl2_to_egui_pointerbutton(mouse_btn),
                        pressed: false,
                        modifiers: Default::default(),
                    })
                }
                sdl2::event::Event::MouseMotion {
                    xrel, yrel, x, y, ..
                } => {
                    camera.mousemove(xrel, yrel);
                    raw_input
                        .events
                        .push(egui::Event::PointerMoved(egui::pos2(x as f32, y as f32)));
                    mvp_needs_update = true;
                }
                sdl2::event::Event::MouseWheel { y, .. } => {
                    camera.mousewheel(y);
                    raw_input.scroll_delta[1] += y as f32;
                    mvp_needs_update = true;
                }
                _ => {}
            }
        }

        // UI
        ctx.begin_frame(raw_input);
        egui::CentralPanel::default().show(&ctx, |ui| {
            ui.label("Hello world!");
            if ui.button("Click me").clicked() {
                eprintln!("Clicky!")
            }
        });
        let (output, shapes) = ctx.end_frame();
        handle_output(output);
        let clipped_meshes: Vec<egui::ClippedMesh> = ctx.tessellate(shapes); // create triangles to paint

        color_buffer.clear();

        // Paint
        unsafe {
            gl::Disable(gl::CULL_FACE);
        }
        for clipped_mesh in clipped_meshes.into_iter() {
            let _rect = dbg!(clipped_mesh.0);
            let indices = clipped_mesh.1.indices;
            let vertices = clipped_mesh.1.vertices;

            if ui_texture_needs_update {
                let texture = match clipped_mesh.1.texture_id {
                    egui::TextureId::Egui => ctx.texture(),
                    egui::TextureId::User(_) => unimplemented!(),
                };
                let pixels: Vec<u8> = texture
                    .srgba_pixels()
                    .flat_map(|c32| std::array::IntoIter::new([c32.r(), c32.g(), c32.b(), c32.a()]))
                    .collect();

                ui.set_texture(texture.width as i32, texture.height as i32, &pixels);
                ui_texture_needs_update = false;
            }
            ui.render(&vertices, &indices, window.size());
        }
        // unsafe {
        //     gl::Enable(gl::CULL_FACE);
        // }

        if mvp_needs_update {
            let aspect = window.size().1 as f32 / window.size().0 as f32;
            let model_view_projection = camera.construct_mvp(aspect, model_isometry);
            let shader = model.shader();
            shader.set_used();
            unsafe {
                shader.set_uniform_matrix4("ProjectionMatrix", model_view_projection);
            }
            mvp_needs_update = false;
        }

        model.render();
        window.gl_swap_window();
        render_gl::check_gl_error();
    }
}

fn handle_output(_output: egui::Output) -> () {
    // Todo
}

// fn convert_screen_space_to_gl_coordinates(pos: &egui::Pos2, size: (u32, u32)) -> (f32, f32) {
//     (
//         pos.x / size.0 as f32 * 2.0 - 1.0,
//         pos.y / size.1 as f32 * 2.0 - 1.0,
//     )
// }

// fn convert_gl_coordinates_to_screen_space(x: f32, y: f32, size: (u32, u32)) -> (f32, f32) {
//     (
//         (x + 1.0) / 2.0 * size.0 as f32,
//         (y + 1.0) / 2.0 * size.1 as f32,
//     )
// }

fn sdl2_to_egui_pointerbutton(button: sdl2::mouse::MouseButton) -> egui::PointerButton {
    match button {
        sdl2::mouse::MouseButton::Left => egui::PointerButton::Primary,
        sdl2::mouse::MouseButton::Right => egui::PointerButton::Secondary,
        sdl2::mouse::MouseButton::Middle => egui::PointerButton::Middle,
        _ => egui::PointerButton::Middle,
    }
}
