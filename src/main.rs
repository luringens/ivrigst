use bevy::{
    prelude::*,
    reflect::TypeUuid,
    render::{
        pipeline::{PipelineDescriptor, RenderPipeline},
        render_graph::{base, AssetRenderResourcesNode, RenderGraph},
        renderer::RenderResources,
        shader::ShaderStages,
    },
};

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
        .add_plugin(bevy_stl::StlPlugin)
        .add_plugin(bevy_orbit_controls::OrbitCameraPlugin)
        .add_asset::<MyMaterial>()
        .add_startup_system(setup.system())
        .run();
}

#[derive(RenderResources, Default, TypeUuid)]
#[uuid = "0805ae06-bfbc-4e78-86bb-c1a4f143c6ad"]
struct MyMaterial {
    pub color: Color,
}

fn setup(
    mut commands: Commands,
    mut pipelines: ResMut<Assets<PipelineDescriptor>>,
    mut materials: ResMut<Assets<MyMaterial>>,
    mut render_graph: ResMut<RenderGraph>,
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
    let material = materials.add(MyMaterial {
        color: Color::rgb(0.6, 0.7, 0.9),
    });

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
    commands
        .spawn_bundle(MeshBundle {
            mesh: asset_server.load(MODEL_PATH),
            transform,
            render_pipelines: RenderPipelines::from_pipelines(vec![RenderPipeline::new(
                pipeline_handle,
            )]),
            ..Default::default()
        })
        .insert(material);
}
