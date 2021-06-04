use bevy::math::vec3;
use bevy::prelude::*;
use bevy_egui::{egui, EguiContext};

use crate::material::MyMaterial;

pub fn ui(egui_context: ResMut<EguiContext>, mut materials: ResMut<Assets<MyMaterial>>) {
    let (handle, _) = materials.iter().next().expect("No material found");
    let material = materials.get_mut(handle).expect("No material extracted");

    egui::Window::new("Settings").show(egui_context.ctx(), |ui| {
        // Colour widget.
        ui.horizontal(|ui| {
            let mut color = material.get_color().clone().into();
            ui.label("Model base colour");
            ui.color_edit_button_rgb(&mut color);
            material.set_color(vec3(color[0], color[1], color[2]));
        });

        // Distance shading parameters widget.
        ui.vertical(|ui| {
            let mut distance_shading = material.get_distance_shading();
            let mut distance_shading_power = material.get_distance_shading_power();
            ui.horizontal(|ui| {
                ui.label("Distance shading min");
                ui.add(egui::Slider::new(&mut distance_shading.x, 0.0..=500.0));
            });
            ui.horizontal(|ui| {
                ui.label("Distance shading max");
                ui.add(egui::Slider::new(&mut distance_shading.y, 0.0..=500.0));
            });
            ui.horizontal(|ui| {
                ui.label("Distance shading power");
                ui.add(egui::Slider::new(&mut distance_shading_power, 0.0..=1.0));
            });
            material.set_distance_shading(distance_shading);
            material.set_distance_shading_power(distance_shading_power);
        });
    });
}

pub fn camera(
    camera_transforms: Query<&Transform, With<bevy::render::camera::Camera>>,
    mut materials: ResMut<Assets<MyMaterial>>,
) {
    let camera_position = camera_transforms
        .iter()
        .next()
        .expect("No camera found")
        .translation;

    let (handle, _) = materials.iter().next().expect("No material found");
    let material = materials.get_mut(handle).expect("No material extracted");

    material.set_camera_position(camera_position);
}
