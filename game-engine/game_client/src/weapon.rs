// game_client/src/weapon.rs

use bevy::prelude::*;
use game_shared::{WeaponType, WeaponStats, NetworkMessage};
use bevy_renet::renet::RenetClient;

use super::player::LocalPlayer;

/// Arma del giocatore locale (usata da handle_shooting)
#[derive(Component)]
pub struct PlayerWeapon {
    pub weapon_type: WeaponType,
    pub stats: WeaponStats,
    pub last_shot_time: f32,
}

impl PlayerWeapon {
    pub fn new(weapon_type: WeaponType) -> Self {
        let stats = WeaponStats::from_type(weapon_type);
        Self { weapon_type, stats, last_shot_time: 0.0 }
    }

    pub fn can_shoot(&self, t: f32) -> bool {
        t - self.last_shot_time >= self.stats.fire_rate
    }

    pub fn shoot(&mut self, t: f32) {
        self.last_shot_time = t;
    }
}

// ─── Helper colori ───────────────────────────────────────────────────────────

fn dark_metal() -> Color   { Color::srgb(0.14, 0.14, 0.17) }
fn mid_metal() -> Color    { Color::srgb(0.35, 0.35, 0.40) }
fn black() -> Color        { Color::srgb(0.08, 0.08, 0.08) }
fn dark_olive() -> Color   { Color::srgb(0.20, 0.22, 0.14) }
fn wood_dark() -> Color    { Color::srgb(0.35, 0.22, 0.10) }
fn wood_light() -> Color   { Color::srgb(0.55, 0.38, 0.18) }
fn sight_yellow() -> Color { Color::srgb(0.90, 0.80, 0.10) }

// ─── Arma 3D (su corpo giocatore remoto) ─────────────────────────────────────
// NON aggiunge PlayerWeapon — solo visuale.

pub fn spawn_weapon(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    weapon_type: WeaponType,
    player_entity: Entity,
) {
    match weapon_type {
        WeaponType::Pistol => spawn_pistol_3d(commands, meshes, materials, player_entity),
        WeaponType::Rifle  => spawn_rifle_3d(commands, meshes, materials, player_entity),
        WeaponType::Shotgun => spawn_shotgun_3d(commands, meshes, materials, player_entity),
    }
}

fn spawn_pistol_3d(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    player_entity: Entity,
) {
    // Base: mano destra ≈ (0.30, 0.85, -0.15) relativo al parent player (piedi=Y=0)
    let base_pos = Vec3::new(0.30, 0.85, -0.18);
    let rot = Quat::from_rotation_x(-0.15);

    let dm  = materials.add(dark_metal());
    let mm  = materials.add(mid_metal());
    let bk  = materials.add(black());
    let sy  = materials.add(sight_yellow());

    // Slide (corpo superiore)
    let slide = commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(0.07, 0.06, 0.22)),
        material: dm.clone(),
        transform: Transform::from_translation(base_pos).with_rotation(rot),
        ..default()
    }).id();

    // Frame (corpo inferiore — leggermente più basso e corto)
    let frame = commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(0.07, 0.05, 0.18)),
        material: dm.clone(),
        transform: Transform::from_xyz(0.0, -0.028, 0.02),
        ..default()
    }).id();

    // Impugnatura
    let grip = commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(0.065, 0.17, 0.08)),
        material: bk,
        transform: Transform::from_xyz(0.0, -0.115, 0.07),
        ..default()
    }).id();

    // Canna (si estende in avanti)
    let barrel = commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(0.035, 0.032, 0.10)),
        material: mm.clone(),
        transform: Transform::from_xyz(0.0, 0.015, -0.155),
        ..default()
    }).id();

    // Mirino frontale
    let front_sight = commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(0.018, 0.030, 0.018)),
        material: sy,
        transform: Transform::from_xyz(0.0, 0.052, -0.20),
        ..default()
    }).id();

    // Mirino posteriore
    let rear_sight = commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(0.055, 0.022, 0.018)),
        material: mm,
        transform: Transform::from_xyz(0.0, 0.050, 0.09),
        ..default()
    }).id();

    commands.entity(slide).push_children(&[frame, grip, barrel, front_sight, rear_sight]);
    commands.entity(player_entity).add_child(slide);
}

fn spawn_rifle_3d(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    player_entity: Entity,
) {
    let base_pos = Vec3::new(0.30, 0.82, -0.22);
    let rot = Quat::from_rotation_x(-0.15);

    let do_ = materials.add(dark_olive());
    let dm  = materials.add(dark_metal());
    let bk  = materials.add(black());
    let wd  = materials.add(wood_dark());

    // Corpo ricevitore
    let receiver = commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(0.09, 0.07, 0.34)),
        material: do_.clone(),
        transform: Transform::from_translation(base_pos).with_rotation(rot),
        ..default()
    }).id();

    // Canna lunga
    let barrel = commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(0.035, 0.032, 0.28)),
        material: dm.clone(),
        transform: Transform::from_xyz(0.0, 0.01, -0.31),
        ..default()
    }).id();

    // Calcio
    let stock = commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(0.09, 0.07, 0.16)),
        material: wd.clone(),
        transform: Transform::from_xyz(0.0, -0.008, 0.25),
        ..default()
    }).id();

    // Impugnatura pistol grip
    let grip = commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(0.07, 0.17, 0.085)),
        material: bk.clone(),
        transform: Transform::from_xyz(0.0, -0.12, 0.09),
        ..default()
    }).id();

    // Caricatore (curved-ish con un rettangolo)
    let mag = commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(0.075, 0.20, 0.065)),
        material: bk.clone(),
        transform: Transform::from_xyz(0.0, -0.135, -0.04),
        ..default()
    }).id();

    // Rail superiore
    let rail = commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(0.055, 0.025, 0.26)),
        material: dm.clone(),
        transform: Transform::from_xyz(0.0, 0.048, 0.01),
        ..default()
    }).id();

    // Flash hider (bocca della canna)
    let muzzle = commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(0.042, 0.042, 0.04)),
        material: dm,
        transform: Transform::from_xyz(0.0, 0.01, -0.47),
        ..default()
    }).id();

    commands.entity(receiver).push_children(&[barrel, stock, grip, mag, rail, muzzle]);
    commands.entity(player_entity).add_child(receiver);
}

fn spawn_shotgun_3d(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    player_entity: Entity,
) {
    let base_pos = Vec3::new(0.30, 0.80, -0.20);
    let rot = Quat::from_rotation_x(-0.15);

    let dm  = materials.add(dark_metal());
    let mm  = materials.add(mid_metal());
    let bk  = materials.add(black());
    let wd  = materials.add(wood_dark());
    let wl  = materials.add(wood_light());

    // Corpo principale
    let body = commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(0.11, 0.085, 0.38)),
        material: dm.clone(),
        transform: Transform::from_translation(base_pos).with_rotation(rot),
        ..default()
    }).id();

    // Canna doppia (più grossa e corta)
    let barrel = commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(0.075, 0.065, 0.22)),
        material: mm.clone(),
        transform: Transform::from_xyz(0.0, 0.025, -0.30),
        ..default()
    }).id();

    // Pump forend
    let pump = commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(0.090, 0.055, 0.15)),
        material: wl,
        transform: Transform::from_xyz(0.0, -0.03, -0.18),
        ..default()
    }).id();

    // Calcio (legno)
    let stock = commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(0.11, 0.08, 0.16)),
        material: wd.clone(),
        transform: Transform::from_xyz(0.0, -0.005, 0.27),
        ..default()
    }).id();

    // Impugnatura pistol
    let grip = commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(0.09, 0.18, 0.095)),
        material: bk,
        transform: Transform::from_xyz(0.0, -0.13, 0.09),
        ..default()
    }).id();

    // Bocca della canna
    let muzzle = commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(0.080, 0.070, 0.03)),
        material: mm,
        transform: Transform::from_xyz(0.0, 0.025, -0.415),
        ..default()
    }).id();

    commands.entity(body).push_children(&[barrel, pump, stock, grip, muzzle]);
    commands.entity(player_entity).add_child(body);
}

// ─── Arma FPS (vista in prima persona, figlia della camera) ──────────────────

/// Marker per l'arma in vista FPS
#[derive(Component)]
pub struct FpsWeapon;

/// Spawna l'arma FPS: figlia della camera, con PlayerWeapon per handle_shooting
pub fn spawn_fps_weapon(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    weapon_type: WeaponType,
    camera_entity: Entity,
) {
    // Posizione camera-local: destra, in basso, avanti
    let (root_pos, root_rot) = match weapon_type {
        WeaponType::Pistol  => (Vec3::new(0.20, -0.22, -0.36), Quat::from_rotation_x(0.06)),
        WeaponType::Rifle   => (Vec3::new(0.22, -0.22, -0.46), Quat::from_rotation_x(0.05)),
        WeaponType::Shotgun => (Vec3::new(0.24, -0.24, -0.44), Quat::from_rotation_x(0.06)),
    };

    let root = commands.spawn((
        SpatialBundle {
            transform: Transform::from_translation(root_pos).with_rotation(root_rot),
            ..default()
        },
        PlayerWeapon::new(weapon_type),
        FpsWeapon,
    )).id();

    match weapon_type {
        WeaponType::Pistol  => build_fps_pistol(commands, meshes, materials, root),
        WeaponType::Rifle   => build_fps_rifle(commands, meshes, materials, root),
        WeaponType::Shotgun => build_fps_shotgun(commands, meshes, materials, root),
    }

    commands.entity(camera_entity).add_child(root);
}

fn build_fps_pistol(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    root: Entity,
) {
    let dm = materials.add(dark_metal());
    let mm = materials.add(mid_metal());
    let bk = materials.add(black());
    let sy = materials.add(sight_yellow());

    // Slide
    let slide = commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(0.075, 0.065, 0.24)),
        material: dm.clone(),
        transform: Transform::from_xyz(0.0, 0.0, 0.0),
        ..default()
    }).id();
    // Frame
    let frame = commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(0.075, 0.055, 0.19)),
        material: dm.clone(),
        transform: Transform::from_xyz(0.0, -0.030, 0.022),
        ..default()
    }).id();
    // Impugnatura
    let grip = commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(0.068, 0.18, 0.085)),
        material: bk.clone(),
        transform: Transform::from_xyz(0.0, -0.125, 0.075),
        ..default()
    }).id();
    // Canna
    let barrel = commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(0.032, 0.030, 0.12)),
        material: mm.clone(),
        transform: Transform::from_xyz(0.0, 0.016, -0.18),
        ..default()
    }).id();
    // Mirino frontale
    let front_sight = commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(0.016, 0.032, 0.016)),
        material: sy,
        transform: Transform::from_xyz(0.0, 0.055, -0.22),
        ..default()
    }).id();
    // Mirino posteriore
    let rear_sight = commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(0.055, 0.020, 0.016)),
        material: mm,
        transform: Transform::from_xyz(0.0, 0.052, 0.10),
        ..default()
    }).id();
    // Guard (tacche di sicurezza)
    let guard = commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(0.065, 0.030, 0.10)),
        material: bk,
        transform: Transform::from_xyz(0.0, -0.058, 0.005),
        ..default()
    }).id();

    commands.entity(root).push_children(&[slide, frame, grip, barrel, front_sight, rear_sight, guard]);
}

fn build_fps_rifle(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    root: Entity,
) {
    let do_ = materials.add(dark_olive());
    let dm  = materials.add(dark_metal());
    let bk  = materials.add(black());
    let wd  = materials.add(wood_dark());

    // Ricevitore
    let recv = commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(0.095, 0.075, 0.36)),
        material: do_,
        transform: Transform::from_xyz(0.0, 0.0, 0.0),
        ..default()
    }).id();
    // Canna
    let barrel = commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(0.032, 0.032, 0.30)),
        material: dm.clone(),
        transform: Transform::from_xyz(0.0, 0.01, -0.33),
        ..default()
    }).id();
    // Calcio
    let stock = commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(0.090, 0.065, 0.15)),
        material: wd,
        transform: Transform::from_xyz(0.0, -0.008, 0.255),
        ..default()
    }).id();
    // Grip
    let grip = commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(0.072, 0.18, 0.085)),
        material: bk.clone(),
        transform: Transform::from_xyz(0.0, -0.13, 0.090),
        ..default()
    }).id();
    // Caricatore
    let mag = commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(0.072, 0.22, 0.065)),
        material: bk.clone(),
        transform: Transform::from_xyz(0.0, -0.148, -0.04),
        ..default()
    }).id();
    // Rail
    let rail = commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(0.052, 0.022, 0.28)),
        material: dm.clone(),
        transform: Transform::from_xyz(0.0, 0.050, 0.01),
        ..default()
    }).id();
    // Flash hider
    let muzzle = commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(0.040, 0.040, 0.040)),
        material: dm,
        transform: Transform::from_xyz(0.0, 0.01, -0.49),
        ..default()
    }).id();

    commands.entity(root).push_children(&[recv, barrel, stock, grip, mag, rail, muzzle]);
}

fn build_fps_shotgun(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    root: Entity,
) {
    let dm  = materials.add(dark_metal());
    let mm  = materials.add(mid_metal());
    let bk  = materials.add(black());
    let wd  = materials.add(wood_dark());
    let wl  = materials.add(wood_light());

    // Corpo
    let body = commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(0.11, 0.088, 0.40)),
        material: dm.clone(),
        transform: Transform::from_xyz(0.0, 0.0, 0.0),
        ..default()
    }).id();
    // Canna
    let barrel = commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(0.072, 0.065, 0.24)),
        material: mm.clone(),
        transform: Transform::from_xyz(0.0, 0.026, -0.32),
        ..default()
    }).id();
    // Pump
    let pump = commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(0.088, 0.055, 0.16)),
        material: wl,
        transform: Transform::from_xyz(0.0, -0.028, -0.19),
        ..default()
    }).id();
    // Calcio
    let stock = commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(0.11, 0.082, 0.16)),
        material: wd,
        transform: Transform::from_xyz(0.0, -0.004, 0.28),
        ..default()
    }).id();
    // Grip
    let grip = commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(0.092, 0.20, 0.096)),
        material: bk,
        transform: Transform::from_xyz(0.0, -0.145, 0.09),
        ..default()
    }).id();
    // Bocca
    let muzzle = commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(0.078, 0.070, 0.030)),
        material: mm,
        transform: Transform::from_xyz(0.0, 0.026, -0.44),
        ..default()
    }).id();

    commands.entity(root).push_children(&[body, barrel, pump, stock, grip, muzzle]);
}

// ─── Sistema: setup arma FPS una volta che il giocatore locale è pronto ──────

pub fn setup_fps_weapon(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    local_player: Option<Res<LocalPlayer>>,
    camera_query: Query<Entity, With<Camera3d>>,
    fps_weapon_query: Query<(), With<FpsWeapon>>,
) {
    // Esegui solo se: giocatore connesso, camera pronta, arma non ancora spawnata
    if local_player.is_none() { return; }
    if !fps_weapon_query.is_empty() { return; }

    if let Ok(camera_entity) = camera_query.get_single() {
        spawn_fps_weapon(
            &mut commands,
            &mut meshes,
            &mut materials,
            WeaponType::Rifle,
            camera_entity,
        );
        println!("🔫 CLIENT: Arma FPS (Rifle) attaccata alla camera");
    }
}

// ─── Sistema: sparo ──────────────────────────────────────────────────────────

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
    window_query: Query<&bevy::window::Window, With<bevy::window::PrimaryWindow>>,
) {
    if !client.is_connected() { return; }

    // Non sparare se mouse non è agganciato
    if let Ok(window) = window_query.get_single() {
        if !matches!(window.cursor.grab_mode, bevy::window::CursorGrabMode::Locked) {
            return;
        }
    }

    let shoot = mouse_button.just_pressed(bevy::input::mouse::MouseButton::Left)
        || keyboard.just_pressed(KeyCode::KeyF);
    if !shoot { return; }

    let current_time = time.elapsed_seconds();

    if let Ok(mut weapon) = weapon_query.get_single_mut() {
        if !weapon.can_shoot(current_time) {
            return;
        }

        if let Ok(camera_transform) = camera_query.get_single() {
            let shoot_direction = camera_transform.forward();
            // Muzzle flash al tip della canna (~0.45m avanti)
            let muzzle_pos = camera_transform.translation + shoot_direction * 0.45;

            // Muzzle flash
            commands.spawn((
                PbrBundle {
                    mesh: meshes.add(Sphere::new(0.10)),
                    material: materials.add(StandardMaterial {
                        base_color: Color::srgb(1.0, 0.85, 0.0),
                        emissive: LinearRgba::rgb(12.0, 9.0, 0.0),
                        ..default()
                    }),
                    transform: Transform::from_translation(muzzle_pos),
                    ..default()
                },
                MuzzleFlash { lifetime: 0.04, elapsed: 0.0 },
            ));

            // Tracciante
            commands.spawn((
                PbrBundle {
                    mesh: meshes.add(Sphere::new(0.07)),
                    material: materials.add(StandardMaterial {
                        base_color: Color::srgb(1.0, 0.9, 0.2),
                        emissive: LinearRgba::rgb(6.0, 5.0, 1.0),
                        ..default()
                    }),
                    transform: Transform::from_translation(muzzle_pos),
                    ..default()
                },
                Projectile {
                    velocity: shoot_direction * 60.0,
                    lifetime: 4.0,
                    elapsed: 0.0,
                },
            ));

            let msg = NetworkMessage::PlayerShoot {
                origin: camera_transform.translation,
                direction: shoot_direction.into(),
                weapon_type: weapon.weapon_type,
            };
            if let Ok(data) = bincode::serialize(&msg) {
                client.send_message(1, data);
            }

            weapon.shoot(current_time);
        }
    }
}

// ─── Componenti ausiliari ─────────────────────────────────────────────────────

#[derive(Component)]
pub struct MuzzleFlash {
    pub lifetime: f32,
    pub elapsed: f32,
}

#[derive(Component)]
pub struct Projectile {
    pub velocity: Vec3,
    pub lifetime: f32,
    pub elapsed: f32,
}

#[derive(Component)]
pub struct HitMarker {
    pub lifetime: f32,
    pub elapsed: f32,
}

pub fn update_projectiles(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &mut Projectile)>,
    time: Res<Time>,
) {
    for (entity, mut transform, mut projectile) in query.iter_mut() {
        projectile.elapsed += time.delta_seconds();
        if projectile.elapsed >= projectile.lifetime {
            commands.entity(entity).despawn();
            continue;
        }
        transform.translation += projectile.velocity * time.delta_seconds();
    }
}

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

// ─── Effetto colpo: scintille + numero danno ─────────────────────────────────

#[derive(Component)]
pub struct HitSpark {
    pub velocity: Vec3,
    pub lifetime: f32,
    pub elapsed: f32,
}

#[derive(Component)]
pub struct DamageNumber {
    pub lifetime: f32,
    pub elapsed: f32,
}

/// Spawna l'effetto visivo quando uno sparo va a segno.
/// Chiamato dal client quando riceve ProjectileHit dal server.
pub fn spawn_hit_effect(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    position: Vec3,
    _damage: f32,
) {
    // Direzioni predeterminate per le scintille (6 raggi + 2 in alto)
    let spark_dirs: [Vec3; 8] = [
        Vec3::new( 1.0,  0.6,  0.0),
        Vec3::new(-1.0,  0.6,  0.0),
        Vec3::new( 0.0,  0.6,  1.0),
        Vec3::new( 0.0,  0.6, -1.0),
        Vec3::new( 0.7,  1.2,  0.7),
        Vec3::new(-0.7,  1.2, -0.7),
        Vec3::new( 0.7,  1.2, -0.7),
        Vec3::new(-0.7,  1.2,  0.7),
    ];

    // Colori alternati: rosso vivo e arancio
    let colors = [
        (Color::srgb(1.0, 0.15, 0.05), LinearRgba::rgb(14.0, 2.0, 0.5)),
        (Color::srgb(1.0, 0.55, 0.0),  LinearRgba::rgb(12.0, 6.0, 0.0)),
    ];

    let spark_mesh = meshes.add(Sphere::new(0.055));

    for (i, dir) in spark_dirs.iter().enumerate() {
        let (base_color, emissive) = colors[i % 2];
        let spark_mat = materials.add(StandardMaterial {
            base_color,
            emissive,
            ..default()
        });
        commands.spawn((
            PbrBundle {
                mesh: spark_mesh.clone(),
                material: spark_mat,
                transform: Transform::from_translation(position),
                ..default()
            },
            HitSpark {
                velocity: dir.normalize() * 3.5,
                lifetime: 0.38,
                elapsed: 0.0,
            },
        ));
    }

    // Il numero danno è gestito dal HitMarkerUI in hud.rs (aggiornato da network.rs)
}

/// Anima scintille: si muovono verso l'alto rallentando, poi despawn
pub fn update_hit_sparks(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &mut HitSpark)>,
    time: Res<Time>,
) {
    let dt = time.delta_seconds();
    for (entity, mut transform, mut spark) in query.iter_mut() {
        spark.elapsed += dt;
        if spark.elapsed >= spark.lifetime {
            commands.entity(entity).despawn();
            continue;
        }
        // Rallenta col tempo (attrito simulato)
        let t = spark.elapsed / spark.lifetime;
        let speed = 1.0 - t * t;
        transform.translation += spark.velocity * speed * dt;
    }
}

/// Anima numero danno: sale verso l'alto e poi despawn
pub fn update_damage_numbers(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &mut DamageNumber)>,
    time: Res<Time>,
) {
    let dt = time.delta_seconds();
    for (entity, mut transform, mut num) in query.iter_mut() {
        num.elapsed += dt;
        if num.elapsed >= num.lifetime {
            commands.entity(entity).despawn();
            continue;
        }
        // Sale lentamente, più veloce all'inizio
        let t = num.elapsed / num.lifetime;
        let rise_speed = (1.0 - t) * 1.2;
        transform.translation.y += rise_speed * dt;
    }
}
