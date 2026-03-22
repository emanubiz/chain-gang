// game_client/src/level.rs

use bevy::prelude::*;

pub fn setup_level(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // ── LUCE AMBIENTALE ───────────────────────────────────────────────
    // Senza questa tutto è nero
    commands.insert_resource(AmbientLight {
        color: Color::srgb(0.6, 0.65, 0.75),
        brightness: 0.8,
    });

    // ── LUCE DIREZIONALE (sole) ───────────────────────────────────────
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 20000.0,
            color: Color::srgb(1.0, 0.97, 0.90),
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(15.0, 30.0, 15.0)
            .looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });

    // ── PAVIMENTO 50x50 a scacchiera ─────────────────────────────────
    let floor_size = 50.0_f32;
    let tile_size = 2.0_f32;
    let tiles_per_side = (floor_size / tile_size) as i32;

    println!("🗺️  CLIENT: Creazione mappa {}x{}", floor_size, floor_size);

    for x in -tiles_per_side / 2..tiles_per_side / 2 {
        for z in -tiles_per_side / 2..tiles_per_side / 2 {
            let is_dark = (x + z) % 2 == 0;
            let color = if is_dark {
                Color::srgb(0.32, 0.42, 0.32)
            } else {
                Color::srgb(0.48, 0.58, 0.48)
            };
            commands.spawn(PbrBundle {
                mesh: meshes.add(Cuboid::new(tile_size, 0.2, tile_size)),
                material: materials.add(color),
                transform: Transform::from_xyz(
                    x as f32 * tile_size + tile_size / 2.0,
                    -0.1,
                    z as f32 * tile_size + tile_size / 2.0,
                ),
                ..default()
            });
        }
    }

    // ── COVER / OSTACOLI ─────────────────────────────────────────────
    let cover_mat = materials.add(Color::srgb(0.50, 0.38, 0.24));
    let cover_positions = [
        (10.0_f32, 0.0_f32),
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

    // ── LUCI POINT (supplementari agli angoli) ────────────────────────
    let corner_lights = [
        (20.0_f32, 8.0_f32, 20.0_f32),
        (-20.0, 8.0, 20.0),
        (20.0, 8.0, -20.0),
        (-20.0, 8.0, -20.0),
    ];
    for (x, y, z) in corner_lights.iter() {
        commands.spawn(PointLightBundle {
            point_light: PointLight {
                intensity: 50_000.0,
                color: Color::srgb(0.8, 0.88, 1.0),
                range: 35.0,
                shadows_enabled: false,
                ..default()
            },
            transform: Transform::from_xyz(*x, *y, *z),
            ..default()
        });
    }

    // ── CAMERA ───────────────────────────────────────────────────────
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
