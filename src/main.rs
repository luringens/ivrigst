#![allow(clippy::missing_safety_doc)]

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
        .window("MedVis", 1000, 1000)
        .opengl()
        .resizable()
        .build()
        .unwrap();

    let _gl_context = window.gl_create_context().unwrap();
    let _gl =
        gl::load_with(|s| video_subsystem.gl_get_proc_address(s) as *const std::os::raw::c_void);

    let mut model = Model::new(&res).expect("Failed to set up model.");
    let mut ui = UI::new(&res).expect("Failed to set up UI.");

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

    render_gl::check_gl_error();

    let mut ctx = egui::CtxRef::default();
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
        if ui_texture_needs_update {
            raw_input.screen_rect = Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(viewport.size().0 as f32, viewport.size().1 as f32),
            ));
        }
        ctx.begin_frame(raw_input);
        egui::Window::new("Settings").show(&ctx, |ui| {
            let mut attr = model.get_attributes().clone();

            // Colour widget.
            ui.horizontal(|ui| {
                ui.label("Model base colour");
                ui.color_edit_button_rgb(&mut attr.color);
            });

            // Toon shading enable/disable
            ui.horizontal(|ui| {
                ui.label("Toon shading factor");
                ui.add(egui::Slider::new(&mut attr.toon_factor, 0.0..=1.0));
            });

            // Distance shading parameters widget.
            ui.vertical(|ui| {
                use crate::model::DistanceShadingChannel as DSC;
                egui::ComboBox::from_label("Distance shading channel")
                    .selected_text(format!("{:?}", attr.distance_shading_channel)) // Todo: fix
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut attr.distance_shading_channel,
                            DSC::Hue,
                            format!("{:?}", DSC::Hue),
                        );
                        ui.selectable_value(
                            &mut attr.distance_shading_channel,
                            DSC::Saturation,
                            format!("{:?}", DSC::Saturation),
                        );
                        ui.selectable_value(
                            &mut attr.distance_shading_channel,
                            DSC::Value,
                            format!("{:?}", DSC::Value),
                        );
                        ui.selectable_value(
                            &mut attr.distance_shading_channel,
                            DSC::None,
                            format!("{:?}", DSC::None),
                        );
                    });
                ui.horizontal(|ui| {
                    ui.label("Distance shading constriction");
                    ui.add(egui::Slider::new(
                        &mut attr.distance_shading_constrict,
                        0.0..=1.0,
                    ));
                });
                ui.horizontal(|ui| {
                    ui.label("Distance shading power");
                    ui.add(egui::Slider::new(
                        &mut attr.distance_shading_power,
                        0.0..=1.0,
                    ));
                });
            });

            model.set_attributes(attr);
        });
        let (output, shapes) = ctx.end_frame();
        handle_output(output);
        let clipped_meshes: Vec<egui::ClippedMesh> = ctx.tessellate(shapes); // create triangles to paint
        if ui_texture_needs_update {
            let texture = ctx.texture();
            ui.set_texture(texture.width as i32, texture.height as i32, &texture);
            ui_texture_needs_update = false;
        }
        color_buffer.clear();

        model.render();

        // Paint
        for egui::ClippedMesh(clip_rect, mesh) in clipped_meshes.into_iter() {
            debug_assert!(mesh.is_valid());

            ui.render(&mesh.vertices, &mesh.indices, clip_rect, viewport.size());
        }

        if mvp_needs_update {
            let aspect = viewport.size().0 as f32 / viewport.size().1 as f32;
            let model_view_projection = camera.construct_mvp(aspect, model_isometry);
            let c = camera.position();
            let camera_position = (c[0], c[1], c[2]);
            let shader = model.shader();
            shader.set_used();
            unsafe {
                shader.set_uniform_matrix4("ProjectionMatrix", model_view_projection);
                shader.set_uniform_3f("camera_position", camera_position);
            }
            mvp_needs_update = false;
        }

        window.gl_swap_window();
        render_gl::check_gl_error();

        // Update shaders if needed
        for path in res.updated_paths() {
            eprintln!("Path updated: {}", path.to_string_lossy());
            model.check_shader_update(&path, &res);
            ui.check_shader_update(&path, &res);
        }
    }
}

fn handle_output(_output: egui::Output) {
    // Todo
}

fn sdl2_to_egui_pointerbutton(button: sdl2::mouse::MouseButton) -> egui::PointerButton {
    match button {
        sdl2::mouse::MouseButton::Left => egui::PointerButton::Primary,
        sdl2::mouse::MouseButton::Right => egui::PointerButton::Secondary,
        sdl2::mouse::MouseButton::Middle => egui::PointerButton::Middle,
        _ => egui::PointerButton::Middle,
    }
}
