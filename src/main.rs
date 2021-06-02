use bevy::{
    math::{vec2, vec3},
    prelude::*,
    reflect::TypeUuid,
    render::{
        pipeline::{PipelineDescriptor, RenderPipeline},
        render_graph::{base, AssetRenderResourcesNode, RenderGraph},
        renderer::RenderResources,
        shader::ShaderStages,
    },
};
use bevy_egui::{egui, EguiContext, EguiPlugin};

const MODEL_PATH: &'static str = "model.obj";

fn main() {
    color_backtrace::install();
    App::build()
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(WindowDescriptor {
            title: "Medvis".to_string(),
            width: 1920.,
            height: 1080.,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(bevy_obj::ObjPlugin)
        .add_plugin(EguiPlugin)
        .add_plugin(bevy_orbit_controls::OrbitCameraPlugin)
        .add_asset::<MyMaterial>()
        .add_startup_system(setup.system())
        .add_system(ui.system())
        .run();
}

#[derive(RenderResources, TypeUuid)]
#[uuid = "0805ae06-bfbc-4e78-86bb-c1a4f143c6ad"]
struct MyMaterial {
    color: Vec3,
    distance_shading: Vec2,
}

impl Default for MyMaterial {
    fn default() -> Self {
        Self {
            color: vec3(0.2, 0.0, 0.5),
            distance_shading: vec2(120.0, 170.0),
        }
    }
}

fn setup(
    mut commands: Commands,
    mut pipelines: ResMut<Assets<PipelineDescriptor>>,
    mut materials: ResMut<Assets<MyMaterial>>,
    mut render_graph: ResMut<RenderGraph>,
    // meshes: Res<Assets<Mesh>>,
    asset_server: Res<AssetServer>,
) {
    asset_server
        .watch_for_changes()
        .expect("Failed to watch for changes");

    // Create a new shader pipeline
    let pipeline_handle = pipelines.add(PipelineDescriptor::default_config(ShaderStages {
        vertex: asset_server.load::<Shader, _>("shader.vert"),
        fragment: Some(asset_server.load::<Shader, _>("shader.frag")),
    }));

    // Add an AssetRenderResourcesNode to our Render Graph. This will bind MyMaterial resources to
    // our shader
    render_graph.add_system_node(
        "my_material",
        AssetRenderResourcesNode::<MyMaterial>::new(true),
    );

    // Add a Render Graph edge connecting our new "my_material" node to the main pass node. This
    // ensures "my_material" runs before the main pass
    render_graph
        .add_node_edge("my_material", base::node::MAIN_PASS)
        .unwrap();

    // Create a new material
    let material = materials.add(MyMaterial::default());

    // Camera
    commands
        .spawn_bundle(PerspectiveCameraBundle {
            transform: Transform::from_translation(Vec3::ONE).looking_at(Vec3::ZERO, Vec3::Y),
            ..Default::default()
        })
        .insert(bevy_orbit_controls::OrbitCamera {
            distance: 200.0,
            ..Default::default()
        });

    let transform_rot = Transform::from_rotation(Quat::from_rotation_x(std::f32::consts::PI));
    let transform_mov = Transform::from_translation(Vec3::new(0.0, -390.0, 50.0));
    let transform = transform_rot * transform_mov;
    let mesh_handle = asset_server.load(MODEL_PATH);
    commands
        .spawn_bundle(MeshBundle {
            mesh: mesh_handle,
            transform,
            render_pipelines: RenderPipelines::from_pipelines(vec![RenderPipeline::new(
                pipeline_handle,
            )]),
            ..Default::default()
        })
        .insert(material);

    // let mesh = meshes.get(mesh_handle.clone_weak());
    // println!("{:?}", mesh.is_some());
    // if let Some(mesh) = mesh {
    //     let vertices = mesh.get_vertex_buffer_data();
    //     let (mut xmin, mut ymin, mut zmin) = (u8::MAX, u8::MAX, u8::MAX);
    //     let (mut xmax, mut ymax, mut zmax) = (u8::MIN, u8::MIN, u8::MIN);

    //     let mut iterator = vertices.chunks_exact(3);
    //     while let Some([x, y, z]) = iterator.next() {
    //         xmin = xmin.min(*x);
    //         ymin = ymin.min(*y);
    //         zmin = zmin.min(*z);
    //         xmax = xmax.max(*x);
    //         ymax = ymax.max(*y);
    //         zmax = zmax.max(*z);
    //     }

    //     let corner1 = vec3(xmin as f32, ymin as f32, zmin as f32);
    //     let corner2 = vec3(xmax as f32, ymax as f32, zmax as f32);
    //     let diagonal = corner1 - corner2;
    //     let max_distance = diagonal.length();
    //     println!("{}", max_distance);
    // }
}

fn ui(
    egui_context: ResMut<EguiContext>,
    mut materials: ResMut<Assets<MyMaterial>>,
    // meshes: Res<Assets<Mesh>>,
    // mut render_graph: ResMut<RenderGraph>,
    // asset_server: Res<AssetServer>,
) {
    let (handle, _) = materials.iter().next().expect("No material found");
    let material = materials.get_mut(handle).expect("No material extracted");
    let mut color = material.color.clone().into();
    egui::Window::new("Settings").show(egui_context.ctx(), |ui| {
        ui.horizontal(|ui| {
            ui.label("Model base colour");
            ui.color_edit_button_rgb(&mut color);
            material.color.x = color[0];
            material.color.y = color[1];
            material.color.z = color[2];
        });

        ui.horizontal(|ui| {
            ui.label("Distance shading min");
            ui.add(egui::Slider::new(
                &mut material.distance_shading.x,
                0.0..=200.0,
            ));
        });
        ui.horizontal(|ui| {
            ui.label("Distance shading max");
            ui.add(egui::Slider::new(
                &mut material.distance_shading.y,
                0.0..=200.0,
            ));
        });
    });
}
