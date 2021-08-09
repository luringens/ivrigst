use crate::{resources::Resources, ui::sdl2_egui_translation::egui_to_sdl2_cursor};
use anyhow::{anyhow, Result};
use nalgebra as na;

use super::UIRenderer;

pub struct UI {
    pub renderer: UIRenderer,
}

impl UI {
    pub fn new(res: &Resources) -> Result<Self> {
        let renderer = UIRenderer::new(res)?;
        Ok(Self { renderer })
    }

    pub fn build_ui(&self, ctx: &egui::CtxRef, model: &mut crate::Model) {
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

                        ui.label("Model colouring mix");
                        ui.add(egui::Slider::new(&mut attr.vertex_color_mix, 0.0..=1.0));
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

                        ui.label("Shadow intensity");
                        ui.add(egui::Slider::new(&mut attr.shadow_intensity, 0.0..=1.0));
                        ui.end_row();

                        ui.label("Light follows camera");
                        ui.checkbox(&mut attr.shadows_follow, "");
                        ui.end_row();

                        ui.label("Light X");
                        ui.scope(|ui| {
                            ui.set_enabled(!attr.shadows_follow);
                            ui.add(egui::Slider::new(&mut attr.light_position[0], -1.0..=1.0))
                                .on_disabled_hover_text("Disabled while following camera.");
                        });
                        ui.end_row();
                        ui.label("Light Y");
                        ui.scope(|ui| {
                            ui.set_enabled(!attr.shadows_follow);
                            ui.add(egui::Slider::new(&mut attr.light_position[1], -1.0..=1.0))
                                .on_disabled_hover_text("Disabled while following camera.");
                        });
                        ui.end_row();
                        ui.label("Light Z");
                        ui.scope(|ui| {
                            ui.set_enabled(!attr.shadows_follow);
                            ui.add(egui::Slider::new(&mut attr.light_position[2], -1.0..=1.0))
                                .on_disabled_hover_text("Disabled while following camera.");
                        });
                        ui.end_row();
                        ui.label("Light orbit distance");
                        ui.add(egui::Slider::new(
                            &mut attr.shadows_orbit_radius,
                            0.0..=100.0,
                        ));
                        ui.end_row();

                        model.set_attributes(attr);
                    });

                ui.horizontal(|ui| {
                    ui.label("Read more at:");
                    ui.add(egui::Hyperlink::new("https://github.com/stisol/rmedvis"));
                });
            });
    }

    pub fn handle_output(&self, output: egui::Output) -> Result<sdl2::mouse::Cursor> {
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
}
