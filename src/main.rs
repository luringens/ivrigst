#![doc = include_str!("../README.md")]
#![allow(clippy::missing_safety_doc)]

mod camera;
mod geometry;
mod model;
pub mod render_gl;
pub mod resources;
mod texture_tester;
mod ui;

use nalgebra as na;
use sdl2::event::Event;
use std::path::Path;
use texture_tester::TextureTester;

use crate::{model::Model, resources::Resources, ui::UI};

#[cfg(debug_assertions)]
#[cfg(target_os = "linux")]
const ASSETS_PATH: &str = "../../assets";
#[cfg(debug_assertions)]
#[cfg(not(target_os = "linux"))]
const ASSETS_PATH: &str = "..\\..\\assets";
#[cfg(not(debug_assertions))]
const ASSETS_PATH: &str = "assets";
const DEFAULT_MODEL_PATH: &str = "model.obj";

fn main() {
    let res =
        Resources::from_relative_exe_path(Path::new(&ASSETS_PATH)).expect("Failed to find assets");

    let sdl = sdl2::init().unwrap();
    let video_subsystem = sdl.video().unwrap();

    let gl_attr = video_subsystem.gl_attr();

    gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
    gl_attr.set_context_version(4, 1);

    let window = video_subsystem
        .window("IVRIGST", 1200, 800)
        .opengl()
        .resizable()
        .build()
        .unwrap();

    let _gl_context = window.gl_create_context().unwrap();
    let _gl =
        gl::load_with(|s| video_subsystem.gl_get_proc_address(s) as *const std::os::raw::c_void);

    let mut model = Model::new(&res, DEFAULT_MODEL_PATH).ok();
    let mut ui = UI::new(&res).expect("Failed to set up UI.");
    if let Some(model) = model.as_mut() {
        ui.apply_preset(model);
    }

    // set up shared state for window
    let mut viewport =
        render_gl::Viewport::for_window(window.size().0 as i32, window.size().1 as i32);
    viewport.set_used();
    let mut color_buffer = render_gl::ColorBuffer::from_color(na::Vector3::new(0.3, 0.3, 0.5));

    // Camera and projection
    let model_isometry = na::Isometry3::new(na::Vector3::zeros(), na::zero());
    let mut camera = camera::Camera::new();
    camera.set_dist(
        model
            .as_ref()
            .map(|m| m.get_size().magnitude() * 1.2)
            .unwrap_or(100.0),
    );

    render_gl::check_gl_error();

    let time = std::time::Instant::now();
    let mut texture_tester = TextureTester::new(&res).expect("Failed to set up texture tester.");

    let mut cursor: sdl2::mouse::Cursor;
    let mut ctx = egui::CtxRef::default();
    let mut first_frame = true;
    let mut mvp_needs_update = true;
    let mut current_model_file = match model {
        Some(_) => DEFAULT_MODEL_PATH.to_owned(),
        None => String::new(),
    };
    let mut ui_actions = ui::UiActions {
        show_debug: false,
        file_to_load: current_model_file.clone(),
        clear_color: color_buffer.color.xyz(),
    };

    let mut event_pump = sdl.event_pump().unwrap();
    'main: loop {
        let mut raw_input: egui::RawInput = Default::default();
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'main,
                Event::Window {
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
                Event::MouseButtonDown {
                    mouse_btn, x, y, ..
                } => {
                    camera.mousedown();
                    raw_input.events.push(egui::Event::PointerButton {
                        pos: egui::pos2(x as f32, y as f32),
                        button: ui::sdl2_to_egui_pointerbutton(mouse_btn),
                        pressed: true,
                        modifiers: Default::default(),
                    })
                }
                Event::MouseButtonUp {
                    mouse_btn, x, y, ..
                } => {
                    camera.mouseup();
                    raw_input.events.push(egui::Event::PointerButton {
                        pos: egui::pos2(x as f32, y as f32),
                        button: ui::sdl2_to_egui_pointerbutton(mouse_btn),
                        pressed: false,
                        modifiers: Default::default(),
                    })
                }
                Event::MouseMotion {
                    xrel, yrel, x, y, ..
                } => {
                    raw_input
                        .events
                        .push(egui::Event::PointerMoved(egui::pos2(x as f32, y as f32)));

                    if !ctx.wants_pointer_input() {
                        let view_updated = camera.mousemove(xrel, yrel);
                        mvp_needs_update = mvp_needs_update || view_updated;
                    }
                }
                Event::MouseWheel { y, .. } => {
                    camera.mousewheel(y);
                    raw_input.events.push(egui::Event::Scroll(egui::Vec2 {
                        x: 0.0,
                        y: y as f32,
                    }));
                    mvp_needs_update = true;
                }
                Event::KeyDown {
                    keycode: Some(keycode),
                    keymod,
                    ..
                } => {
                    if let Some(event) = ui::sdl2_to_egui_key(keycode, keymod, true) {
                        raw_input.events.push(event);
                    }
                    if let Some(event) = ui::sdl2_to_egui_text(keycode, keymod) {
                        raw_input.events.push(event);
                    }
                }
                Event::KeyUp {
                    keycode: Some(keycode),
                    keymod,
                    ..
                } => {
                    if let Some(event) = ui::sdl2_to_egui_key(keycode, keymod, false) {
                        raw_input.events.push(event);
                    }
                }
                _ => {}
            }
        }

        // UI handling
        ctx.begin_frame(raw_input);
        ui.build_ui(&ctx, &mut model, &mut ui_actions);
        let (output, shapes) = ctx.end_frame();
        let clipped_meshes: Vec<egui::ClippedMesh> = ctx.tessellate(shapes);

        // Handle egui output - clipboard events, changing cursor, etc.
        match ui.handle_output(output) {
            Ok(c) => {
                // Unsets when dropped, so we need to store a reference to it outside the loop.
                cursor = c;
                cursor.set();
            }
            Err(e) => {
                eprintln!("egui output handling error:");
                eprintln!("{}", e);
            }
        }

        color_buffer.clear();

        // Update camera if necessary.
        if mvp_needs_update && model.is_some() {
            let model = model.as_mut().unwrap();
            let mut attr = model.get_attributes().clone();

            let aspect = viewport.size().0 as f32 / viewport.size().1 as f32;
            let model_view_projection = camera.construct_mvp(aspect, model_isometry);
            let c = camera.position();
            attr.camera_position = na::Vector3::new(c[0], c[1], c[2]);
            attr.projection_matrix = model_view_projection;
            model.set_attributes(attr);
            mvp_needs_update = false;
        }

        // Render the model
        if let Some(model) = model.as_mut() {
            let elapsed = time.elapsed();
            let mut attr = model.get_attributes().clone();
            attr.elapsed = elapsed.as_millis() as f32;
            model.set_attributes(attr);
            model.render(&viewport);
        }

        // The egui font texture is available after a frame has passed, so we need to grab it here.
        if first_frame {
            let texture = ctx.font_image();
            ui.renderer
                .set_texture(texture.width as i32, texture.height as i32, &texture);
            first_frame = false;
        }

        // Render the UI
        for egui::ClippedMesh(clip_rect, mesh) in clipped_meshes.into_iter() {
            debug_assert!(mesh.is_valid());

            ui.renderer
                .render(&mesh.vertices, &mesh.indices, clip_rect, viewport.size());
        }

        // Render debug textures if chosen
        if let Some(model) = model.as_mut() {
            if ui_actions.show_debug {
                texture_tester.render(
                    &viewport,
                    model.get_hatch_texture(),
                    model.get_shadow_texture(),
                );
            }
        }

        // Check if model should be reloaded
        if ui_actions.file_to_load != current_model_file {
            let mut path = ui_actions.file_to_load.clone();
            path.push_str(".obj");
            if let Ok(mut new_model) = Model::new(&res, &path) {
                camera.set_dist(new_model.get_size().magnitude() * 1.2);
                ui.apply_preset(&mut new_model);
                model = Some(new_model);
                mvp_needs_update = true;
                current_model_file = ui_actions.file_to_load.clone();
            }
        }

        // Check the if clear color needs updating
        if ui_actions.clear_color != color_buffer.color.xyz() {
            color_buffer.update_color(ui_actions.clear_color);
        }
        window.gl_swap_window();
        render_gl::check_gl_error();

        // Update shaders if needed
        for path in res.updated_paths() {
            eprintln!("Path updated: {}", path.to_string_lossy());
            if let Some(model) = model.as_mut() {
                model.check_shader_update(&path, &res);
            }
            texture_tester.check_shader_update(&path, &res);
        }
    }
}
