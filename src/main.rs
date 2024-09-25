use std::f32::consts::PI;
use std::time::Duration;
use camera_controller::{CameraController, CameraControllerPlugin};

use bevy::{
    animation::{animate_targets, RepeatAnimation},
    pbr::CascadeShadowConfigBuilder,
    prelude::*,
};

const MODEL_PATH: &str = "ANGEL-FRANK1_converted.glb";

#[derive(Resource)]
struct ModelAsset {
    scene: Handle<Scene>,
    animation: Handle<AnimationClip>,
}

fn main() {
    App::new()
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 2000.0,
        })
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, load_model)
        .add_systems(Startup, setup.after(load_model))
        .add_plugins(CameraControllerPlugin)
        .add_systems(Update, setup_scene_once_loaded.before(animate_targets))
        .add_systems(Update, animate_light_direction)
        .run();
}

#[derive(Resource)]
pub struct Animations {
    animations: Vec<AnimationNodeIndex>,
    #[allow(dead_code)]
    graph: Handle<AnimationGraph>,
}

mod camera_controller;

fn load_model(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let scene = asset_server.load(format!("{}#Scene0", MODEL_PATH));
    let animation = asset_server.load(format!("{}#Animation0", MODEL_PATH));
    
    commands.insert_resource(ModelAsset { scene, animation });
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    model_asset: Res<ModelAsset>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
) {
    // Build the animation graph
    let mut graph = AnimationGraph::new();
    let animations = graph
        .add_clips(
            [model_asset.animation.clone()]
                .into_iter(),
            1.0,
            graph.root,
        )
        .collect();

    // Insert a resource with the current scene information
    let graph = graphs.add(graph);
    commands.insert_resource(Animations {
        animations,
        graph: graph.clone(),
    });

    // Camera setup
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 5.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    })
    .insert(CameraController::default());

    //Mesh
    commands.spawn(PbrBundle {
        mesh: meshes.add(Plane3d::default().mesh().size(500000.0, 500000.0)),
        material: materials.add(Color::srgb(0.3, 0.5, 0.3)),
        ..default()
    });

    // Light setup
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0, // Adjust the light intensity
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0), // Adjust the light position
        ..default()
    });

    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
        cascade_shadow_config: CascadeShadowConfigBuilder {
            num_cascades: 1,
            maximum_distance: 1.6,
            ..default()
        }
        .into(),
        ..default()
    });

    // Load the glTF scene and start the animation
    // let scene_handle = asset_server.load("assets/ANGEL-FRANK1_converted.glb#Scene0");
    commands.spawn(SceneBundle {
        scene: model_asset.scene.clone(),
        ..default()
    });
}

// Once the scene is loaded, start the animation
fn setup_scene_once_loaded(
    mut commands: Commands,
    animations: Res<Animations>,
    mut players: Query<(Entity, &mut AnimationPlayer), Added<AnimationPlayer>>,
) {
    for (entity, mut player) in &mut players {
        let mut transitions = AnimationTransitions::new();

        // Start the animation via the `AnimationTransitions` component
        transitions
            .play(&mut player, animations.animations[0], Duration::ZERO)
            .repeat();

        commands
            .entity(entity)
            .insert(animations.graph.clone())
            .insert(transitions);
    }
}

fn animate_light_direction(
    time: Res<Time>,
    mut query: Query<&mut Transform, With<DirectionalLight>>,
) {
    for mut transform in &mut query {
        transform.rotation = Quat::from_euler(
            EulerRot::ZYX,
            0.0,
            time.elapsed_seconds() * std::f32::consts::PI / 50.0,
            -std::f32::consts::FRAC_PI_4,
        );
    }
}
