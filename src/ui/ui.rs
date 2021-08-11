use crate::{
    model::DistanceShadingChannel, resources::Resources,
    ui::sdl2_egui_translation::egui_to_sdl2_cursor,
};
use anyhow::{anyhow, Result};
use nalgebra as na;

use super::UIRenderer;

pub struct UI {
    pub renderer: UIRenderer,
    preset: Preset,
}

#[derive(Default)]
pub struct UiActions {
    pub show_debug: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Preset {
    ToonWithShadow,
    PseudoChromaDepth,
    PlainAerial,
}

impl UI {
    pub fn new(res: &Resources) -> Result<Self> {
        let renderer = UIRenderer::new(res)?;
        let preset = Preset::ToonWithShadow;
        Ok(Self { renderer, preset })
    }

    pub fn build_ui(
        &mut self,
        ctx: &egui::CtxRef,
        model: &mut crate::Model,
        ui_actions: &mut UiActions,
    ) {
        egui::Window::new("Settings")
            .auto_sized()
            .collapsible(true)
            .show(ctx, |ui| {
                egui::ComboBox::from_label("Preset")
                    .selected_text("Toon shading with shadows")
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut self.preset,
                            Preset::ToonWithShadow,
                            "Toon shading with shadows",
                        );
                        ui.selectable_value(
                            &mut self.preset,
                            Preset::PseudoChromaDepth,
                            "Pseudochroma depth",
                        );
                        ui.selectable_value(
                            &mut self.preset,
                            Preset::PlainAerial,
                            "Plain shading with aerial distance",
                        );
                    });
                if ui.button("Apply preset").clicked() {
                    self.apply_preset(model);
                }

                ui.collapsing("Advanced", |ui| {
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

                            ui.label("Hatching depth");
                            ui.add(egui::Slider::new(&mut attr.hatching_depth, 1.0..=4.0));
                            ui.end_row();

                            ui.label("Hatching steps");
                            ui.add(egui::Slider::new(&mut attr.hatching_steps, 1..=500));
                            ui.end_row();

                            ui.label("Hatching frequency");
                            ui.add(egui::Slider::new(&mut attr.hatching_frequency, 1..=50));
                            ui.end_row();

                            ui.label("Hatching intensity");
                            ui.add(egui::Slider::new(&mut attr.hatching_intensity, 0.0..=1.0));
                            ui.end_row();

                            ui.label("Display shader buffers");
                            ui.checkbox(&mut ui_actions.show_debug, "");
                            ui.end_row();

                            model.set_attributes(attr);
                        });
                });

                ui.horizontal(|ui| {
                    ui.label("Read more at:");
                    ui.add(egui::Hyperlink::new("https://github.com/stisol/rmedvis"));
                });
            });
    }

    fn apply_preset(&self, model: &mut crate::Model) {
        let mut preset = model.get_attributes().clone();
        match self.preset {
            Preset::ToonWithShadow => {
                preset.toon_factor = 0.8;
                preset.distance_shading_channel = DistanceShadingChannel::None;
                preset.shadow_intensity = 0.6;
                preset.shadows_follow = false;
                preset.shadows_orbit_radius = 25.0;
                preset.vertex_color_mix = 1.0;
            }
            Preset::PseudoChromaDepth => {
                preset.distance_shading_power = 0.8;
                preset.distance_shading_constrict = 0.8;
                preset.toon_factor = 0.7;
                preset.distance_shading_channel = DistanceShadingChannel::Hue;
                preset.shadows_follow = true;
                preset.shadows_orbit_radius = 25.0;
                preset.shadow_intensity = 0.6;
                preset.vertex_color_mix = 1.0;
            }
            Preset::PlainAerial => {
                preset.distance_shading_power = 0.8;
                preset.distance_shading_constrict = 0.8;
                preset.toon_factor = 0.3;
                preset.distance_shading_channel = DistanceShadingChannel::Saturation;
                preset.shadow_intensity = 0.0;
                preset.vertex_color_mix = 1.0;
            }
        };
        model.set_attributes(preset);
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
