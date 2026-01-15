use bevy::prelude::*;
use game_shared::hello_shared;

fn main() {
    println!("🎮 CLIENT: Avvio grafica...");
    hello_shared();

    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "CHAIN GANG - Client".into(),
                resolution: (800., 600.).into(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(Color::srgb(0.1, 0.1, 0.15))) // Colore di sfondo scuro
        .add_systems(Startup, setup_level) // <--- Aggiungiamo questo sistema
        .run();
}

fn setup_level(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // 1. Pavimento (Un cubo verde piatto)
    commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(20.0, 1.0, 20.0)), // Largo 20, Alto 1
        material: materials.add(Color::srgb(0.3, 0.5, 0.3)),
        transform: Transform::from_xyz(0.0, -0.5, 0.0),
        ..default()
    });

    // 2. Il Giocatore (Un cubo rosso - il nostro Voxel)
    commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(1.0, 1.0, 1.0)), // Cubo 1x1x1
        material: materials.add(Color::srgb(0.8, 0.2, 0.2)),
        transform: Transform::from_xyz(0.0, 0.5, 0.0),
        ..default()
    });

    // 3. Luce (Un "Sole" per fare le ombre)
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });

    // 4. Telecamera (Posizionata in alto che guarda giù)
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(-5.0, 6.0, 8.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
}