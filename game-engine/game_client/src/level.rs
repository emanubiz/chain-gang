// game_client/src/level.rs

use bevy::prelude::*;

pub fn setup_level(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // MAPPA PIÙ GRANDE: 50x50 invece di 20x20
    let floor_size = 50.0;
    let tile_size = 2.0;
    let tiles_per_side = (floor_size / tile_size) as i32;
    
    println!("🗺️  CLIENT: Creazione mappa {}x{}", floor_size, floor_size);
    
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

    // Aggiungi alcuni ostacoli/cover sparsi sulla mappa
    let cover_mat = materials.add(Color::srgb(0.4, 0.3, 0.2));
    let cover_positions = [
        (10.0, 0.0),
        (-10.0, 0.0),
        (0.0, 10.0),
        (0.0, -10.0),
        (15.0, 15.0),
        (-15.0, -15.0),
        (15.0, -15.0),
        (-15.0, 15.0),
    ];

    for (x, z) in cover_positions.iter() {
        commands.spawn(PbrBundle {
            mesh: meshes.add(Cuboid::new(2.0, 1.5, 2.0)),
            material: cover_mat.clone(),
            transform: Transform::from_xyz(*x, 0.75, *z),
            ..default()
        });
    }

    // Luce principale
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 8000.0, // Aumentata per mappa più grande
            shadows_enabled: true,
            color: Color::srgb(1.0, 0.98, 0.95),
            shadow_depth_bias: 0.02,
            shadow_normal_bias: 0.6,
            range: 60.0,
            ..default()
        },
        transform: Transform::from_xyz(0.0, 20.0, 0.0),
        ..default()
    });
    
    // Luci secondarie agli angoli
    let corner_lights = [
        (20.0, 10.0, 20.0),
        (-20.0, 10.0, 20.0),
        (20.0, 10.0, -20.0),
        (-20.0, 10.0, -20.0),
    ];

    for (x, y, z) in corner_lights.iter() {
        commands.spawn(PointLightBundle {
            point_light: PointLight {
                intensity: 2000.0,
                shadows_enabled: false,
                color: Color::srgb(0.7, 0.8, 1.0),
                range: 30.0,
                ..default()
            },
            transform: Transform::from_xyz(*x, *y, *z),
            ..default()
        });
    }

    // Camera con FOV aumentato
    commands.spawn(Camera3dBundle {
        projection: Projection::Perspective(PerspectiveProjection {
            fov: 90.0_f32.to_radians(),
            ..default()
        }),
        transform: Transform::from_xyz(0.0, 2.0, 0.0),
        ..default()
    });
    
    println!("✅ CLIENT: Livello pronto (Mappa 50x50 con cover)");
}