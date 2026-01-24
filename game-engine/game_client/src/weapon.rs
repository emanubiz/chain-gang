// game_client/src/weapon.rs

use bevy::prelude::*;
use game_shared::{WeaponType, WeaponStats, NetworkMessage};
use bevy_renet::renet::RenetClient;

/// Componente per identificare l'arma del giocatore
#[derive(Component)]
pub struct PlayerWeapon {
    pub weapon_type: WeaponType,
    pub stats: WeaponStats,
    pub last_shot_time: f32,
}

impl PlayerWeapon {
    pub fn new(weapon_type: WeaponType) -> Self {
        let stats = WeaponStats::from_type(weapon_type);
        Self {
            weapon_type,
            stats,
            last_shot_time: 0.0,
        }
    }

    pub fn can_shoot(&self, current_time: f32) -> bool {
        current_time - self.last_shot_time >= self.stats.fire_rate
    }

    pub fn shoot(&mut self, current_time: f32) {
        self.last_shot_time = current_time;
    }
}

/// Spawna un'arma parametrica in base al tipo
pub fn spawn_weapon(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    weapon_type: WeaponType,
) -> Entity {
    let (size, color, sight_color, position, rotation) = match weapon_type {
        WeaponType::Pistol => (
            Vec3::new(0.10, 0.06, 0.30),
            Color::srgb(0.15, 0.15, 0.18),
            Color::srgb(0.8, 0.8, 0.1),
            Vec3::new(0.3, 0.8, 0.4), // Braccio destro altezza
            Quat::from_rotation_x(-0.2),
        ),
        WeaponType::Rifle => (
            Vec3::new(0.12, 0.06, 0.50),
            Color::srgb(0.12, 0.12, 0.15),
            Color::srgb(0.8, 0.1, 0.1),
            Vec3::new(0.35, 0.75, 0.45), // Braccio destro altezza
            Quat::from_rotation_x(-0.25),
        ),
        WeaponType::Shotgun => (
            Vec3::new(0.14, 0.08, 0.45),
            Color::srgb(0.2, 0.15, 0.1),
            Color::srgb(0.1, 0.8, 0.1),
            Vec3::new(0.35, 0.7, 0.4), // Braccio destro altezza
            Quat::from_rotation_x(-0.3),
        ),
    };

    let weapon_mat = materials.add(color);
    let sight_mat = materials.add(sight_color);

    let weapon = commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(size.x, size.y, size.z)),
        material: weapon_mat,
        transform: Transform::from_translation(position).with_rotation(rotation),
        ..default()
    }).id();

    // Mirino
    let sight = commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(0.04, 0.04, 0.04)),
        material: sight_mat,
        transform: Transform::from_xyz(0.0, 0.05, -size.z * 0.4),
        ..default()
    }).id();

    commands.entity(weapon).add_child(sight);
    commands.entity(weapon).insert(PlayerWeapon::new(weapon_type));

    weapon
}

/// Sistema per gestire lo sparo
pub fn handle_shooting(
    mouse_button: Res<ButtonInput<bevy::input::mouse::MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut client: ResMut<RenetClient>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    time: Res<Time>,
    mut weapon_query: Query<&mut PlayerWeapon>,
    camera_query: Query<&Transform, With<Camera3d>>,
) {
    if !client.is_connected() {
        return;
    }

    // SPARO: Click sinistro del mouse O tasto F
    let shoot = mouse_button.just_pressed(bevy::input::mouse::MouseButton::Left) 
        || keyboard.just_pressed(KeyCode::KeyF);

    if !shoot {
        return;
    }

    let current_time = time.elapsed_seconds();

    // Trova l'arma del giocatore
    if let Ok(mut weapon) = weapon_query.get_single_mut() {
        if !weapon.can_shoot(current_time) {
            println!("⏳ Aspetta il cooldown!");
            return;
        }

        // Calcola la direzione dello sparo dalla camera
        if let Ok(camera_transform) = camera_query.get_single() {
            let shoot_direction = camera_transform.forward();
            let muzzle_pos = camera_transform.translation + shoot_direction * 0.5;

            // EFFETTO MUZZLE FLASH (lampo alla bocca dell'arma)
            commands.spawn((
                PbrBundle {
                    mesh: meshes.add(Sphere::new(0.15)),
                    material: materials.add(StandardMaterial {
                        base_color: Color::srgb(1.0, 0.8, 0.0),
                        emissive: LinearRgba::rgb(10.0, 8.0, 0.0),
                        ..default()
                    }),
                    transform: Transform::from_translation(muzzle_pos),
                    ..default()
                },
                MuzzleFlash { lifetime: 0.05, elapsed: 0.0 },
            ));

            // PROIETTILE VISIVO (tracciante) - PIÙ GRANDE E LENTO
            commands.spawn((
                PbrBundle {
                    mesh: meshes.add(Sphere::new(0.1)), // Sfera invece di capsula
                    material: materials.add(StandardMaterial {
                        base_color: Color::srgb(1.0, 0.9, 0.2),
                        emissive: LinearRgba::rgb(5.0, 4.5, 1.0),
                        ..default()
                    }),
                    transform: Transform::from_translation(muzzle_pos),
                    ..default()
                },
                Projectile {
                    velocity: shoot_direction * 50.0, // PIÙ LENTO per vederlo
                    lifetime: 5.0,
                    elapsed: 0.0,
                },
            ));

            // Invia il messaggio di sparo al server SU CANALE 1
            let msg = NetworkMessage::PlayerShoot {
                origin: camera_transform.translation,
                direction: shoot_direction.into(),
                weapon_type: weapon.weapon_type,
            };

            if let Ok(data) = bincode::serialize(&msg) {
                client.send_message(1, data); // CANALE 1 per shooting
                println!("💥 BANG! {:?} - Inviato al server", weapon.weapon_type);
            }

            weapon.shoot(current_time);
        }
    }
}

/// Componente per il lampo dello sparo
#[derive(Component)]
pub struct MuzzleFlash {
    pub lifetime: f32,
    pub elapsed: f32,
}

/// Componente per il proiettile visivo
#[derive(Component)]
pub struct Projectile {
    pub velocity: Vec3,
    pub lifetime: f32,
    pub elapsed: f32,
}

/// Sistema per animare i proiettili
pub fn update_projectiles(
    mut commands: Commands,
    mut projectile_query: Query<(Entity, &mut Transform, &mut Projectile)>,
    time: Res<Time>,
) {
    let dt = time.delta_seconds();
    
    for (entity, mut transform, mut projectile) in projectile_query.iter_mut() {
        projectile.elapsed += dt;
        
        if projectile.elapsed >= projectile.lifetime {
            commands.entity(entity).despawn();
            continue;
        }
        
        // Muovi il proiettile
        transform.translation += projectile.velocity * dt;
    }
}

/// Sistema per rimuovere i muzzle flash
pub fn cleanup_muzzle_flash(
    mut commands: Commands,
    mut query: Query<(Entity, &mut MuzzleFlash)>,
    time: Res<Time>,
) {
    for (entity, mut flash) in query.iter_mut() {
        flash.elapsed += time.delta_seconds();
        if flash.elapsed >= flash.lifetime {
            commands.entity(entity).despawn();
        }
    }
}

/// Componente per marker temporanei
#[derive(Component)]
pub struct HitMarker {
    pub lifetime: f32,
    pub elapsed: f32,
}

/// Sistema per rimuovere i marker dopo il tempo
pub fn cleanup_hit_markers(
    mut commands: Commands,
    mut query: Query<(Entity, &mut HitMarker)>,
    time: Res<Time>,
) {
    for (entity, mut marker) in query.iter_mut() {
        marker.elapsed += time.delta_seconds();
        if marker.elapsed >= marker.lifetime {
            commands.entity(entity).despawn();
        }
    }
}