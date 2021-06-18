#![allow(clippy::missing_safety_doc)]

mod camera;
mod model;
pub mod render_gl;
pub mod resources;
mod sdl2_egui_translation;
mod ui;

use anyhow::{anyhow, Result};
use nalgebra as na;
use sdl2::event::Event;
use std::path::Path;

use crate::{model::Model, resources::Resources, sdl2_egui_translation::*, ui::UI};

#[cfg(debug_assertions)]
const ASSETS_PATH: &str = "..\\..\\assets";
#[cfg(not(debug_assertions))]
const ASSETS_PATH: &str = "assets";

fn main() {
    let res =
        Resources::from_relative_exe_path(Path::new(ASSETS_PATH)).expect("Failed to find assets");

    let sdl = sdl2::init().unwrap();
    let video_subsystem = sdl.video().unwrap();

    let gl_attr = video_subsystem.gl_attr();

    gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
    gl_attr.set_context_version(4, 1);

    let window = video_subsystem
        .window("MedVis", 1200, 800)
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

    let mut cursor: sdl2::mouse::Cursor;
    let mut ctx = egui::CtxRef::default();
    let mut first_frame = true;
    let mut mvp_needs_update = true;
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
                        button: sdl2_to_egui_pointerbutton(mouse_btn),
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
                        button: sdl2_to_egui_pointerbutton(mouse_btn),
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
                    raw_input.scroll_delta[1] += y as f32;
                    mvp_needs_update = true;
                }
                Event::KeyDown {
                    keycode: Some(keycode),
                    keymod,
                    ..
                } => {
                    if let Some(event) = sdl2_to_egui_key(keycode, keymod, true) {
                        raw_input.events.push(event);
                    }
                    if let Some(event) = sdl2_to_egui_text(keycode, keymod) {
                        raw_input.events.push(event);
                    }
                }
                Event::KeyUp {
                    keycode: Some(keycode),
                    keymod,
                    ..
                } => {
                    if let Some(event) = sdl2_to_egui_key(keycode, keymod, false) {
                        raw_input.events.push(event);
                    }
                }
                _ => {}
            }
        }

        // UI handling
        ctx.begin_frame(raw_input);
        build_ui(&ctx, &mut model);
        let (output, shapes) = ctx.end_frame();
        let clipped_meshes: Vec<egui::ClippedMesh> = ctx.tessellate(shapes);

        // Handle egui output - clipboard events, changing cursor, etc.
        match handle_output(output) {
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
        if mvp_needs_update {
            let mut attr = model.get_attributes().clone();

            let aspect = viewport.size().0 as f32 / viewport.size().1 as f32;
            let model_view_projection = camera.construct_mvp(aspect, model_isometry);
            let c = camera.position();
            attr.camera_position = na::Vector3::new(c[0], c[1], c[2]);
            attr.projection_matrix = model_view_projection;
            model.set_attributes(attr);
            mvp_needs_update = false;
        }
        model.render();

        // The egui texture isn't available until one frame has passed, so we've got to do it here.
        if first_frame {
            let texture = ctx.texture();
            ui.set_texture(texture.width as i32, texture.height as i32, &texture);
            first_frame = false;
        }

        // Render the UI
        for egui::ClippedMesh(clip_rect, mesh) in clipped_meshes.into_iter() {
            debug_assert!(mesh.is_valid());

            ui.render(&mesh.vertices, &mesh.indices, clip_rect, viewport.size());
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

fn build_ui(ctx: &egui::CtxRef, model: &mut Model) {
    egui::Window::new("Settings")
        .auto_sized()
        .collapsible(true)
        .show(ctx, |ui| {
            egui::Grid::new("settings_grid")
                .striped(true)
                .spacing([40.0, 4.0])
                .show(ui, |ui| {
                    let mut attr = model.get_attributes().clone();

                    // Colour widget.
                    ui.label("Model base colour");
                    let mut color = [attr.color[0], attr.color[1], attr.color[2]];
                    ui.color_edit_button_rgb(&mut color);
                    attr.color = na::Vector3::from(color);

                    ui.end_row();

                    // Toon shading enable/disable
                    ui.label("Toon shading factor");
                    ui.add(egui::Slider::new(&mut attr.toon_factor, 0.0..=1.0));
                    ui.end_row();

                    // Distance shading parameters widget.
                    use crate::model::DistanceShadingChannel as DSC;
                    ui.label("Distance shading channel");
                    egui::ComboBox::from_id_source("distance_shading_channel")
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
                    ui.end_row();

                    ui.label("Distance shading constriction");
                    ui.add(egui::Slider::new(
                        &mut attr.distance_shading_constrict,
                        0.0..=1.0,
                    ));
                    ui.end_row();

                    ui.label("Distance shading power");
                    ui.scope(|ui| {
                        ui.set_enabled(attr.distance_shading_channel != DSC::Hue);
                        ui.add(egui::Slider::new(
                            &mut attr.distance_shading_power,
                            0.0..=1.0,
                        ))
                        .on_disabled_hover_text("Not used when channel is set to Hue.");
                    });
                    ui.end_row();

                    model.set_attributes(attr);
                });

            ui.horizontal(|ui| {
                ui.label("Read more at:");
                ui.add(egui::Hyperlink::new("https://github.com/stisol/rmedvis"));
            });
        });
}

fn handle_output(output: egui::Output) -> Result<sdl2::mouse::Cursor> {
    let system_cursor = egui_to_sdl2_cursor(output.cursor_icon);
    let cursor = sdl2::mouse::Cursor::from_system(system_cursor).map_err(|e| anyhow!(e))?;

    if !output.copied_text.is_empty() {
        use clipboard::{ClipboardContext, ClipboardProvider};
        let mut ctx: ClipboardContext =
            ClipboardProvider::new().map_err(|_| anyhow!("Could not open clipboard."))?;
        ctx.set_contents(output.copied_text)
            .map_err(|_| anyhow!("Could not set clipboard text."))?;
    }

    if let Some(url) = output.open_url {
        if let Err(e) = webbrowser::open(&url.url) {
            eprintln!("Error opening link: {}", e);
        }
    }

    Ok(cursor)
}
