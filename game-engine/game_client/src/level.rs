// game_client/src/level.rs

use bevy::prelude::*;

pub fn setup_level(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Pavimento voxel con grid pattern
    let floor_size = 20.0;
    let tile_size = 2.0;
    let tiles_per_side = (floor_size / tile_size) as i32;
    
    for x in -tiles_per_side/2..tiles_per_side/2 {
        for z in -tiles_per_side/2..tiles_per_side/2 {
            let is_dark = (x + z) % 2 == 0;
            let color = if is_dark {
                Color::srgb(0.25, 0.35, 0.25)
            } else {
                Color::srgb(0.35, 0.45, 0.35)
            };
            
            commands.spawn(PbrBundle {
                mesh: meshes.add(Cuboid::new(tile_size, 0.2, tile_size)),
                material: materials.add(color),
                transform: Transform::from_xyz(
                    x as f32 * tile_size + tile_size / 2.0,
                    -0.1,
                    z as f32 * tile_size + tile_size / 2.0
                ),
                ..default()
            });
        }
    }

    // Luce principale
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 3000.0,
            shadows_enabled: true,
            color: Color::srgb(1.0, 0.98, 0.95),
            shadow_depth_bias: 0.02,
            shadow_normal_bias: 0.6,
            ..default()
        },
        transform: Transform::from_xyz(0.0, 12.0, 0.0),
        ..default()
    });
    
    // Luce secondaria
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 800.0,
            shadows_enabled: false,
            color: Color::srgb(0.7, 0.8, 1.0),
            ..default()
        },
        transform: Transform::from_xyz(-8.0, 6.0, -8.0),
        ..default()
    });

    // Camera con FOV aumentato (per una visuale più ampia)
    commands.spawn(Camera3dBundle {
        projection: Projection::Perspective(PerspectiveProjection {
            fov: 90.0_f32.to_radians(), // Aumentato a 90 gradi
            ..default()
        }),
        transform: Transform::from_xyz(0.0, 2.0, 0.0), // Posizione iniziale
        ..default()
    });
    
    // ARMA RIMOSSA DALLA CAMERA - Ora sarà sul braccio del personaggio
}