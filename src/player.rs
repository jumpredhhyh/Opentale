use crate::debug_tools::debug_resource::SpellhavenDebug;
use crate::ui::ui::UiSpawnCallback;
use crate::world_generation::chunk_generation::VOXEL_SIZE;
use crate::world_generation::chunk_loading::chunk_loader::ChunkLoader;
use bevy::core_pipeline::experimental::taa::TemporalAntiAliasPlugin;
use bevy::pbr::ScreenSpaceAmbientOcclusion;
use bevy::prelude::*;
use bevy::render::camera::Exposure;
use bevy::{core_pipeline::experimental::taa::TemporalAntiAliasing, ecs::system::SystemId};
use bevy_atmosphere::prelude::AtmosphereCamera;
use bevy_panorbit_camera::PanOrbitCamera;
use bevy_rapier3d::prelude::{
    CharacterAutostep, CharacterLength, Collider, KinematicCharacterController,
    KinematicCharacterControllerOutput, RigidBody,
};
use std::f32::consts::PI;

pub const STEP_HEIGHT: f32 = 1. * VOXEL_SIZE;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(TemporalAntiAliasPlugin)
            .add_systems(Startup, register_spawn_player_system)
            .add_systems(Update, (movement, move_camera, move_body));
    }
}

#[derive(Component)]
pub struct Player {
    velocity: Vec3,
    jumped: bool,
    fly: bool,
}

#[derive(Component)]
struct PlayerBody;

#[derive(Component)]
struct PlayerCamera;

#[derive(Resource)]
pub struct PlayerSpawnCallback(pub SystemId);

fn register_spawn_player_system(world: &mut World) {
    let id = world.register_system(spawn_player);
    world.insert_resource(PlayerSpawnCallback(id));
}

fn spawn_player(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
    ui_spawn_callback: Res<UiSpawnCallback>,
) {
    // Player
    commands.spawn((
        RigidBody::KinematicPositionBased,
        Transform::from_xyz(0., 2200., 0.),
        Collider::cuboid(0.4, 0.9, 0.4),
        KinematicCharacterController {
            offset: CharacterLength::Absolute(0.01),
            autostep: Some(CharacterAutostep {
                min_width: CharacterLength::Absolute(0.01),
                max_height: CharacterLength::Absolute(VOXEL_SIZE + 0.1),
                include_dynamic_bodies: true,
            }),
            ..default()
        },
        Player {
            velocity: Vec3::ZERO,
            jumped: false,
            fly: true,
        },
        ChunkLoader::default(),
        Name::new("Player"),
    ));

    commands.spawn((
        Camera3d::default(),
        Msaa::Off,
        TemporalAntiAliasing::default(),
        Transform::from_xyz(-4.0, 6.5, 8.0).looking_at(Vec3::ZERO, Vec3::Y),
        Projection::Perspective(PerspectiveProjection {
            far: 2f32.powi(20),
            ..default()
        }),
        Exposure { ev100: 10f32 },
        PanOrbitCamera::default(),
        AtmosphereCamera::default(),
        PlayerCamera,
        Name::new("PlayerCamera"),
        ScreenSpaceAmbientOcclusion::default(),
    ));

    commands
        .spawn((PlayerBody, Name::new("PlayerBody"), Mesh3d::default()))
        .with_children(|commands| {
            commands.spawn((
                SceneRoot(asset_server.load("player.gltf#Scene0")),
                Transform::from_xyz(0., 0.15, 0.),
                Name::new("PlayerHead"),
            ));
            commands.spawn((
                Mesh3d(meshes.add(Mesh::from(Capsule3d {
                    radius: 0.4,
                    half_length: 0.3,
                    ..default()
                }))),
                MeshMaterial3d(materials.add(Color::srgb(0.8, 0.7, 0.6))),
                Transform::from_xyz(0., -0.35, 0.),
                Name::new("PlayerTorso"),
            ));
        });

    commands.run_system(ui_spawn_callback.0);
}

fn move_body(
    player: Query<&Transform, (With<Player>, Without<PlayerBody>)>,
    mut player_body: Query<&mut Transform, (With<PlayerBody>, Without<Player>)>,
) {
    let (Ok(player), Ok(mut player_body)) = (player.single(), player_body.single_mut()) else {
        return;
    };

    let difference = player.translation - player_body.translation;
    player_body.translation += difference * 0.25;
    player_body.rotation = player_body.rotation.lerp(player.rotation, 0.25);
}

fn move_camera(
    player: Query<&Transform, (With<Player>, Without<PlayerCamera>)>,
    mut camera: Query<&mut PanOrbitCamera, (With<PlayerCamera>, Without<Player>)>,
    options: Res<SpellhavenDebug>,
) {
    if options.unlock_camera {
        return;
    }

    let (Ok(player), Ok(mut camera)) = (player.single(), camera.single_mut()) else {
        return;
    };

    let camera_position = camera.target_focus;
    let difference = (player.translation + Vec3::Y) - camera_position;
    camera.target_focus += difference * 0.25;
}

fn movement(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut players: Query<(
        &mut KinematicCharacterController,
        &mut Player,
        Option<&KinematicCharacterControllerOutput>,
        &mut Transform,
    )>,
    player_camera: Query<&PanOrbitCamera, With<PlayerCamera>>,
) {
    for (mut controller, mut player, controller_output, mut transform) in &mut players {
        if keyboard_input.just_pressed(KeyCode::KeyF) {
            player.fly = !player.fly;
        }

        let mut move_direction = Vec3::ZERO;
        let mut last_movement = player.velocity;

        if let Some(controller_output) = controller_output {
            if player.jumped && controller_output.grounded {
                player.jumped = false;
            }
        }

        last_movement.x *= 0.8;
        last_movement.y *= if player.fly { 0.8 } else { 0.98 };
        last_movement.z *= 0.8;

        // Directional movement
        if keyboard_input.pressed(KeyCode::KeyW) || keyboard_input.pressed(KeyCode::ArrowUp) {
            move_direction.z -= 1.;
        }
        if keyboard_input.pressed(KeyCode::KeyA) || keyboard_input.pressed(KeyCode::ArrowLeft) {
            move_direction.x -= 1.;
        }
        if keyboard_input.pressed(KeyCode::KeyS) || keyboard_input.pressed(KeyCode::ArrowDown) {
            move_direction.z += 1.;
        }
        if keyboard_input.pressed(KeyCode::KeyD) || keyboard_input.pressed(KeyCode::ArrowRight) {
            move_direction.x += 1.;
        }
        if player.fly {
            if keyboard_input.pressed(KeyCode::KeyE) {
                move_direction.y += 1.;
            }
            if keyboard_input.pressed(KeyCode::KeyQ) {
                move_direction.y -= 1.;
            }
        }

        let mut movement_speed = if keyboard_input.pressed(KeyCode::ShiftLeft) {
            2.
        } else {
            1.
        };

        if player.fly {
            movement_speed *= 50.;
        }

        if let Ok(player_camera) = player_camera.single() {
            // Rotate vector to camera
            let rotation = Quat::from_rotation_y(player_camera.yaw.unwrap_or(0.));
            move_direction = rotation.mul_vec3(move_direction.normalize_or_zero() * movement_speed);
        }

        if !player.fly && controller_output.is_some() && !controller_output.unwrap().grounded {
            move_direction.y -= 0.4;
        }

        move_direction *= time.delta_secs();

        // Jump if space pressed and the player is close enough to the ground
        if keyboard_input.pressed(KeyCode::Space)
            && controller_output.is_some()
            && controller_output.unwrap().grounded
            && !player.jumped
        {
            move_direction.y = 0.1;
            player.jumped = true;
        }

        let movement = move_direction + last_movement;
        controller.translation = Some(movement);
        player.velocity = movement;

        move_direction.y = 0.0;
        if move_direction.max_element() > 0.0 || move_direction.min_element() < 0.0 {
            transform.rotation = Quat::from_rotation_y(-move_direction.xz().to_angle() - PI / 2.0);
        }
    }
}

// fn kinematic_collision(
//     mut collision_event_reader: EventReader<Collision>,
//     mut bodies: Query<&RigidBody, Without<Player>>,
//     mut player_bodies: Query<(&mut Position, &ShapeHits), With<Player>>,
//     player_shape_hits_x: Query<&ShapeHits, With<PlayerSteppingCastX>>,
//     player_shape_hits_neg_x: Query<&ShapeHits, With<PlayerSteppingCastNegX>>,
//     player_shape_hits_z: Query<&ShapeHits, With<PlayerSteppingCastZ>>,
//     player_shape_hits_neg_z: Query<&ShapeHits, With<PlayerSteppingCastNegZ>>,
// ) {
//     // Iterate through collisions and move the kinematic body to resolve penetration
//     for Collision(contact) in collision_event_reader.iter() {
//         if let Ok((player_position, is_grounded)) = player_bodies.get_mut(contact.entity1) {
//             if let Ok(other_rb) = bodies.get_mut(contact.entity2) {
//                 handle_collision(player_position, player_shape_hits_x.single(), player_shape_hits_neg_x.single(), player_shape_hits_z.single(), player_shape_hits_neg_z.single(), other_rb, contact, false, !is_grounded.is_empty());
//             }
//         } else if let Ok((player_position, is_grounded)) = player_bodies.get_mut(contact.entity2) {
//             if let Ok(other_rb) = bodies.get_mut(contact.entity1) {
//                 handle_collision(player_position, player_shape_hits_x.single(), player_shape_hits_neg_x.single(), player_shape_hits_z.single(), player_shape_hits_neg_z.single(), other_rb, contact, true, !is_grounded.is_empty());
//             }
//         }
//     }
// }

// fn handle_collision(mut player_position: Mut<Position>, player_stepping_x: &ShapeHits, player_stepping_neg_x: &ShapeHits, player_stepping_z: &ShapeHits, player_stepping_neg_z: &ShapeHits, other_rb: &RigidBody, contact: &Contact, inverse: bool, is_grounded: bool) {
//     if contact.penetration <= Scalar::EPSILON || other_rb.is_kinematic() {
//         return;
//     }
//
//     let normal_to_use = if inverse { contact.normal * -1. } else { contact.normal };
//
//     if normal_to_use.y.abs() < 0.1 && is_grounded {
//         let corresponding_shape_hits: Option<&ShapeHits>;
//
//         if normal_to_use == Vec3::X {
//             corresponding_shape_hits = Some(player_stepping_x);
//         } else if normal_to_use == Vec3::NEG_X {
//             corresponding_shape_hits = Some(player_stepping_neg_x);
//         } else if normal_to_use == Vec3::Z {
//             corresponding_shape_hits = Some(player_stepping_z);
//         } else if normal_to_use == Vec3::NEG_Z {
//             corresponding_shape_hits = Some(player_stepping_neg_z);
//         } else {
//             corresponding_shape_hits = None;
//         }
//
//         if corresponding_shape_hits.is_some() {
//             if !corresponding_shape_hits.unwrap().is_empty() {
//                 let hit = corresponding_shape_hits.unwrap().as_slice().first().unwrap();
//                 if hit.time_of_impact > 0. {
//                     player_position.0.y += (STEP_HEIGHT + VOXEL_SIZE / 2.) - hit.time_of_impact;
//                     return;
//                 }
//             }
//         }
//     }
//
//     player_position.0 -= normal_to_use * contact.penetration;
// }
