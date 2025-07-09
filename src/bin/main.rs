use bevy::pbr::wireframe::{WireframeConfig, WireframePlugin};
use bevy::pbr::ExtendedMaterial;
use bevy::prelude::*;
use bevy::window::PresentMode;
use bevy_atmosphere::prelude::AtmospherePlugin;
use bevy_inspector_egui::bevy_egui::EguiPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_panorbit_camera::PanOrbitCameraPlugin;
use bevy_rapier3d::prelude::{NoUserData, RapierPhysicsPlugin};
use opentale::animations::AnimationPlugin;
use opentale::debug_tools::debug_resource::SpellhavenDebugPlugin;
use opentale::player::PlayerPlugin;
use opentale::ui::ui::GameUiPlugin;
use opentale::world_generation::array_texture::ArrayTextureMaterial;
use opentale::world_generation::WorldGenerationPlugin;
use std::f32::consts::PI;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Opentale".into(),
                        present_mode: PresentMode::Immediate,
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
            PanOrbitCameraPlugin,
            WorldGenerationPlugin,
            AtmospherePlugin,
            RapierPhysicsPlugin::<NoUserData>::default(),
            //RapierDebugRenderPlugin::default(),
            PlayerPlugin,
            WireframePlugin { ..default() },
            AnimationPlugin,
            //BirdCameraPlugin,
            EguiPlugin {
                enable_multipass_for_primary_context: false,
            },
            WorldInspectorPlugin::new(),
            GameUiPlugin,
            SpellhavenDebugPlugin,
            MaterialPlugin::<ExtendedMaterial<StandardMaterial, ArrayTextureMaterial>>::default(),
        ))
        .add_systems(Startup, setup)
        .insert_resource(WireframeConfig {
            global: false,
            default_color: Color::srgb(1., 0., 0.),
        })
        .run();
}

fn setup(mut commands: Commands, _asset_server: Res<AssetServer>) {
    commands.spawn((
        DirectionalLight {
            shadows_enabled: true,
            illuminance: 1000.,
            ..default()
        },
        Transform {
            translation: Vec3::new(0.0, 2.0, 0.0),
            rotation: Quat::from_rotation_x(-PI / 3.),
            ..default()
        },
        Name::new("Light"),
    ));

    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 75f32,
        ..default()
    });

    // commands.spawn(SceneBundle {
    //     scene: asset_server.load("player.gltf#Scene0"),
    //     transform: Transform::from_xyz(0., 150., 0.),
    //     ..default()
    // });
}
