// game_client/src/main.rs - Step 1.3: Player Movement (Client-Side Prediction)

use bevy::prelude::*;
use game_shared::*;
use bevy_renet::renet::{ConnectionConfig, RenetClient};
use bevy_renet::renet::transport::{NetcodeClientTransport, ClientAuthentication};
use bevy_renet::RenetClientPlugin;
use std::collections::{HashMap, VecDeque};
use std::net::UdpSocket;
use std::time::SystemTime;

#[derive(Resource)]
struct Transport(NetcodeClientTransport);

/// ID del client corrente
#[derive(Resource)]
struct OurClientId(u64);

/// Mappa Entity ID -> Entity (per giocatori remoti e oggetti)
#[derive(Resource, Default)]
struct SynchronizedEntities {
    map: HashMap<u64, Entity>,
}

/// Entità del giocatore locale controllato dal client
#[derive(Resource)]
struct LocalPlayer(Entity);

/// Risorsa per tracciare la rotazione della camera (mouse look)
#[derive(Resource, Default)]
struct CameraRotation {
    yaw: f32,   // Rotazione orizzontale
    pitch: f32, // Rotazione verticale
}

/// Storico degli input inviati (per reconciliation)
#[derive(Resource, Default)]
struct InputHistory {
    inputs: VecDeque<(u32, PlayerInput)>,
    next_sequence: u32,
}

impl InputHistory {
    fn add(&mut self, input: PlayerInput) -> u32 {
        let seq = self.next_sequence;
        self.next_sequence += 1;
        
        let mut input_with_seq = input;
        input_with_seq.sequence_number = seq;
        
        self.inputs.push_back((seq, input_with_seq));
        
        // Mantieni solo gli ultimi 100 input
        while self.inputs.len() > 100 {
            self.inputs.pop_front();
        }
        
        seq
    }
    
    fn remove_until(&mut self, sequence_number: u32) {
        while let Some((seq, _)) = self.inputs.front() {
            if *seq <= sequence_number {
                self.inputs.pop_front();
            } else {
                break;
            }
        }
    }
}

fn main() {
    println!("🎮 CLIENT: Avvio grafica e connessione...");
    hello_shared();

    App::new()
        .add_plugins(
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "CHAIN GANG - Client".into(),
                    resolution: (1280., 720.).into(),
                    ..default()
                }),
                ..default()
            }),
        )
        .insert_resource(ClearColor(Color::srgb(0.15, 0.18, 0.25)))
        .add_plugins(RenetClientPlugin)
        .insert_resource(SynchronizedEntities::default())
        .insert_resource(InputHistory::default())
        .insert_resource(CameraRotation::default())
        .add_systems(Startup, (setup_level, setup_network).chain())
        .add_systems(Update, (
            update_transport,
            handle_mouse_look,
            handle_input,
            apply_local_prediction,
            update_camera_position,
            receive_network_messages,
            client_tick,
        ).chain())
        .run();
}

fn setup_network(mut commands: Commands) {
    let server_addr = format!("{}:{}", SERVER_ADDR, SERVER_PORT).parse().unwrap();
    let current_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();

    let client_id = current_time.as_millis() as u64;

    let client_socket = UdpSocket::bind("0.0.0.0:0")
        .expect("Impossibile bindare il socket UDP del client");

    println!("✅ CLIENT: Socket UDP bindato");
    println!("🔌 CLIENT: Connessione a {}...", server_addr);
    println!("🆔 CLIENT: Il nostro ID è {}", client_id);

    let client = RenetClient::new(ConnectionConfig::default());

    let authentication = ClientAuthentication::Unsecure {
        protocol_id: PROTOCOL_ID,
        client_id,
        server_addr,
        user_data: None,
    };

    let transport = NetcodeClientTransport::new(current_time, authentication, client_socket)
        .expect("Impossibile creare NetcodeClientTransport");

    commands.insert_resource(client);
    commands.insert_resource(Transport(transport));
    commands.insert_resource(OurClientId(client_id));
}

fn handle_mouse_look(
    mut camera_rotation: ResMut<CameraRotation>,
    mut mouse_motion: EventReader<bevy::input::mouse::MouseMotion>,
    mouse_button: Res<ButtonInput<bevy::input::mouse::MouseButton>>,
) {
    // Solo se il tasto destro del mouse è premuto (per non bloccare il cursore)
    if mouse_button.pressed(bevy::input::mouse::MouseButton::Right) {
        for motion in mouse_motion.read() {
            // Sensibilità mouse
            let sensitivity = 0.003;
            
            camera_rotation.yaw -= motion.delta.x * sensitivity;
            camera_rotation.pitch -= motion.delta.y * sensitivity;
            
            // Limita il pitch per non girare la testa troppo (no backflip)
            camera_rotation.pitch = camera_rotation.pitch.clamp(-1.5, 1.5);
        }
    }
}

fn handle_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut client: ResMut<RenetClient>,
    mut input_history: ResMut<InputHistory>,
    camera_rotation: Res<CameraRotation>,
    local_player: Option<Res<LocalPlayer>>,
) {
    // Aspetta che il giocatore locale sia spawnato
    if local_player.is_none() || !client.is_connected() {
        return;
    }
    
    // Leggi input da tastiera
    let mut move_direction = Vec2::ZERO;
    
    if keyboard.pressed(KeyCode::KeyW) {
        move_direction.y += 1.0;
    }
    if keyboard.pressed(KeyCode::KeyS) {
        move_direction.y -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyA) {
        move_direction.x -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyD) {
        move_direction.x += 1.0;
    }
    
    // Normalizza per movimento diagonale consistente
    if move_direction.length() > 0.0 {
        move_direction = move_direction.normalize();
    }
    
    let jump = keyboard.pressed(KeyCode::Space);
    
    // Crea input
    let input = PlayerInput {
        move_direction,
        jump,
        yaw: camera_rotation.yaw,
        pitch: camera_rotation.pitch,
        sequence_number: 0, // Verrà assegnato da input_history
    };
    
    // Aggiungi all'history e ottieni sequence number
    let seq = input_history.add(input);
    
    // Invia al server
    let mut input_with_seq = input;
    input_with_seq.sequence_number = seq;
    
    let msg = NetworkMessage::PlayerInput(input_with_seq);
    if let Ok(data) = bincode::serialize(&msg) {
        client.send_message(0, data);
    }
}

fn update_camera_position(
    local_player: Option<Res<LocalPlayer>>,
    player_query: Query<&Transform, With<PlayerController>>,
    mut camera_query: Query<&mut Transform, (With<Camera3d>, Without<PlayerController>)>,
    camera_rotation: Res<CameraRotation>,
) {
    if let Some(local_player) = local_player {
        if let Ok(player_transform) = player_query.get(local_player.0) {
            if let Ok(mut camera_transform) = camera_query.get_single_mut() {
                // 🔥 Camera MOLTO più alta e lontana (third-person)
                let eye_height = 2.5; // Molto più alta
                let back_offset = 3.5; // Molto più lontana
                let up_offset = 1.0;   // Guardando leggermente dall'alto
                
                let yaw_quat = Quat::from_rotation_y(camera_rotation.yaw);
                let backward = yaw_quat * Vec3::new(0.0, 0.0, back_offset);
                
                camera_transform.translation = player_transform.translation 
                    + Vec3::new(0.0, eye_height + up_offset, 0.0)
                    + backward;
                
                // Camera guarda leggermente verso il basso
                let look_down = -0.3; // Pitch verso il basso
                camera_transform.rotation = Quat::from_rotation_y(camera_rotation.yaw) 
                    * Quat::from_rotation_x(camera_rotation.pitch + look_down);
            }
        }
    }
}

fn apply_local_prediction(
    mut query: Query<(&mut Transform, &mut PlayerPhysics, &mut PlayerController)>,
    local_player: Option<Res<LocalPlayer>>,
    input_history: Res<InputHistory>,
    time: Res<Time>,
) {
    if let Some(local_player) = local_player {
        if let Ok((mut transform, mut physics, mut controller)) = query.get_mut(local_player.0) {
            // Applica l'ultimo input ricevuto
            if let Some((_, input)) = input_history.inputs.back() {
                let dt = time.delta_seconds();
                
                controller.grounded = transform.translation.y <= PLAYER_HEIGHT / 2.0 + 0.01;
                
                apply_player_movement(
                    input,
                    &mut transform,
                    &mut physics,
                    &controller,
                    dt
                );
            }
        }
    }
}

fn spawn_voxel_player(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    position: Vec3,
    base_color: Color,
) -> Entity {
    use game_shared::*;
    use std::f32::consts::PI;

    // 🔥 VARIANTI CASUALI per personaggi stilosi
    let style_variants = [
        // Stile 1: Cool guy con occhiali
        (
            Color::srgb(0.96, 0.82, 0.70), // Skin chiara
            Color::srgb(1.0, 0.85, 0.0),   // Capelli biondi
            Color::srgb(0.1, 0.6, 0.9),    // Shirt azzurra
            Color::srgb(0.2, 0.2, 0.8),    // Pantaloni blu
            true, // Ha occhiali
        ),
        // Stile 2: Ragazza capelli rossi
        (
            Color::srgb(0.98, 0.78, 0.69), // Skin media
            Color::srgb(0.8, 0.2, 0.2),    // Capelli rossi
            Color::srgb(0.9, 0.5, 0.7),    // Shirt rosa
            Color::srgb(0.3, 0.2, 0.6),    // Pantaloni viola
            false,
        ),
        // Stile 3: Skater
        (
            Color::srgb(0.85, 0.65, 0.50), // Skin scura
            Color::srgb(0.15, 0.12, 0.10), // Capelli neri
            Color::srgb(0.9, 0.1, 0.1),    // Shirt rossa
            Color::srgb(0.15, 0.15, 0.15), // Pantaloni neri
            true, // Ha cappellino
        ),
    ];
    
    // Scegli variante basata sul colore base
    let variant_index = if base_color.to_srgba().red > 0.5 { 0 } else if base_color.to_srgba().green > 0.5 { 1 } else { 2 };
    let (skin_color, hair_color, shirt_color, pants_color, has_accessory) = style_variants[variant_index];

    // Materiali
    let skin_mat = materials.add(skin_color);
    let hair_mat = materials.add(hair_color);
    let shirt_mat = materials.add(shirt_color);
    let pants_mat = materials.add(pants_color);
    let shoe_mat = materials.add(Color::srgb(0.95, 0.95, 0.95)); // Scarpe bianche
    let accessory_mat = materials.add(Color::srgb(0.1, 0.1, 0.1));

    // Parent entity
    let parent = commands.spawn((
        SpatialBundle {
            transform: Transform::from_translation(position),
            ..default()
        },
        PlayerPhysics::default(),
    )).id();

    // TESTA
    let head = commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(HEAD_SIZE, HEAD_SIZE, HEAD_SIZE)),
        material: skin_mat.clone(),
        transform: Transform::from_xyz(0.0, HEAD_Y_OFFSET, 0.0),
        ..default()
    }).id();

    // 🔥 OCCHI STILOSI - Grandi e espressivi
    let eye_size = VOXEL_SCALE * 1.8;
    let eye_white_mat = materials.add(Color::srgb(1.0, 1.0, 1.0));
    let pupil_mat = materials.add(Color::srgb(0.05, 0.05, 0.1));
    
    let left_eye_white = commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(eye_size, eye_size * 1.2, VOXEL_SCALE * 0.8)),
        material: eye_white_mat.clone(),
        transform: Transform::from_xyz(-0.13, 0.12, HEAD_SIZE/2.0 + 0.03),
        ..default()
    }).id();
    
    commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(eye_size * 0.6, eye_size * 0.6, VOXEL_SCALE * 0.4)),
        material: pupil_mat.clone(),
        transform: Transform::from_xyz(0.0, -0.05, 0.05),
        ..default()
    }).set_parent(left_eye_white);
    
    let right_eye_white = commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(eye_size, eye_size * 1.2, VOXEL_SCALE * 0.8)),
        material: eye_white_mat,
        transform: Transform::from_xyz(0.13, 0.12, HEAD_SIZE/2.0 + 0.03),
        ..default()
    }).id();
    
    commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(eye_size * 0.6, eye_size * 0.6, VOXEL_SCALE * 0.4)),
        material: pupil_mat,
        transform: Transform::from_xyz(0.0, -0.05, 0.05),
        ..default()
    }).set_parent(right_eye_white);
    
    commands.entity(head).push_children(&[left_eye_white, right_eye_white]);

    // 🔥 BOCCA SORRIDENTE più ampia
    let mouth_mat = materials.add(Color::srgb(0.85, 0.3, 0.35));
    commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(0.20, 0.05, 0.04)),
        material: mouth_mat,
        transform: Transform::from_xyz(0.0, -0.10, HEAD_SIZE/2.0 + 0.02),
        ..default()
    }).set_parent(head);

    // 🔥 CAPELLI STILOSI - Più voluminosi
    let hair_main = commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(HEAD_SIZE * 1.1, 0.25, HEAD_SIZE * 1.1)),
        material: hair_mat.clone(),
        transform: Transform::from_xyz(0.0, HEAD_SIZE/2.0 + 0.10, 0.0),
        ..default()
    }).id();
    
    // Ciuffo frontale
    commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(0.15, 0.12, 0.12)),
        material: hair_mat.clone(),
        transform: Transform::from_xyz(0.08, 0.10, HEAD_SIZE/2.0 + 0.05),
        ..default()
    }).set_parent(head);
    
    commands.entity(head).add_child(hair_main);

    // 🔥 ACCESSORI (occhiali o cappellino)
    if has_accessory && variant_index == 0 {
        // Occhiali da sole
        let glasses_mat = materials.add(Color::srgb(0.1, 0.1, 0.1));
        commands.spawn(PbrBundle {
            mesh: meshes.add(Cuboid::new(0.30, 0.12, 0.04)),
            material: glasses_mat,
            transform: Transform::from_xyz(0.0, 0.10, HEAD_SIZE/2.0 + 0.04),
            ..default()
        }).set_parent(head);
    } else if has_accessory {
        // Cappellino
        commands.spawn(PbrBundle {
            mesh: meshes.add(Cuboid::new(HEAD_SIZE * 1.15, 0.10, HEAD_SIZE * 1.15)),
            material: accessory_mat.clone(),
            transform: Transform::from_xyz(0.0, HEAD_SIZE/2.0 + 0.22, 0.0),
            ..default()
        }).set_parent(head);
        
        // Visiera
        commands.spawn(PbrBundle {
            mesh: meshes.add(Cuboid::new(HEAD_SIZE * 1.2, 0.02, 0.25)),
            material: accessory_mat,
            transform: Transform::from_xyz(0.0, HEAD_SIZE/2.0 + 0.18, HEAD_SIZE/2.0 + 0.15),
            ..default()
        }).set_parent(head);
    }

    // CORPO
    let body = commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(BODY_WIDTH, BODY_HEIGHT, BODY_DEPTH)),
        material: shirt_mat.clone(),
        transform: Transform::from_xyz(0.0, BODY_Y_OFFSET, 0.0),
        ..default()
    }).id();

    // BRACCIA
    let arm_offset_x = BODY_WIDTH / 2.0 + ARM_WIDTH / 2.0;
    commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(ARM_WIDTH, ARM_HEIGHT, ARM_DEPTH)),
        material: shirt_mat.clone(),
        transform: Transform::from_xyz(-arm_offset_x, ARM_Y_OFFSET, 0.0),
        ..default()
    }).set_parent(body);

    commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(ARM_WIDTH, ARM_HEIGHT, ARM_DEPTH)),
        material: shirt_mat.clone(),
        transform: Transform::from_xyz(arm_offset_x, ARM_Y_OFFSET, 0.0),
        ..default()
    }).set_parent(body);

    // GAMBE
    let leg_offset_x = BODY_WIDTH * 0.25;
    commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(LEG_WIDTH, LEG_HEIGHT, LEG_DEPTH)),
        material: pants_mat.clone(),
        transform: Transform::from_xyz(-leg_offset_x, LEG_Y_OFFSET, 0.0),
        ..default()
    }).set_parent(body);

    commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(LEG_WIDTH, LEG_HEIGHT, LEG_DEPTH)),
        material: pants_mat,
        transform: Transform::from_xyz(leg_offset_x, LEG_Y_OFFSET, 0.0),
        ..default()
    }).set_parent(body);

    // 🔥 SCARPE STILOSE - Tipo sneaker
    let shoe_height = 0.10;
    let shoe_platform = 0.04;
    
    // Scarpa sinistra
    let left_shoe = commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(LEG_WIDTH * 1.15, shoe_height, LEG_DEPTH * 1.3)),
        material: shoe_mat.clone(),
        transform: Transform::from_xyz(-leg_offset_x, shoe_height/2.0, 0.05),
        ..default()
    }).id();
    
    // Suola scarpa sinistra
    commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(LEG_WIDTH * 1.2, shoe_platform, LEG_DEPTH * 1.35)),
        material: materials.add(Color::srgb(0.2, 0.2, 0.2)),
        transform: Transform::from_xyz(0.0, -shoe_height/2.0 - shoe_platform/2.0, 0.0),
        ..default()
    }).set_parent(left_shoe);
    
    // Scarpa destra
    let right_shoe = commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(LEG_WIDTH * 1.15, shoe_height, LEG_DEPTH * 1.3)),
        material: shoe_mat,
        transform: Transform::from_xyz(leg_offset_x, shoe_height/2.0, 0.05),
        ..default()
    }).id();
    
    // Suola scarpa destra
    commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(LEG_WIDTH * 1.2, shoe_platform, LEG_DEPTH * 1.35)),
        material: materials.add(Color::srgb(0.2, 0.2, 0.2)),
        transform: Transform::from_xyz(0.0, -shoe_height/2.0 - shoe_platform/2.0, 0.0),
        ..default()
    }).set_parent(right_shoe);

    commands.entity(body).push_children(&[left_shoe, right_shoe]);
    commands.entity(parent).push_children(&[head, body]);

    parent
}

fn receive_network_messages(
    mut commands: Commands,
    mut client: ResMut<RenetClient>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut synchronized_entities: ResMut<SynchronizedEntities>,
    mut input_history: ResMut<InputHistory>,
    local_player: Option<Res<LocalPlayer>>,
    our_client_id: Res<OurClientId>,
    mut local_query: Query<(&mut Transform, &mut PlayerPhysics), With<PlayerController>>,
    mut remote_query: Query<(&mut Transform, &mut PlayerPhysics), Without<PlayerController>>,
) {
    while let Some(message) = client.receive_message(0) {
        if let Ok(msg) = bincode::deserialize::<NetworkMessage>(&message) {
            match msg {
                NetworkMessage::PlayerConnected { entity_id, client_id } => {
                    println!("👤 CLIENT: Player {} connesso (entity: {})", client_id, entity_id);
                    
                    if our_client_id.0 == client_id {
                        let player_entity = spawn_voxel_player(
                            &mut commands,
                            &mut meshes,
                            &mut materials,
                            Vec3::new(0.0, 2.0, 0.0),
                            Color::srgb(0.2, 0.8, 0.2),
                        );
                        
                        commands.entity(player_entity).insert(PlayerController::default());
                        commands.insert_resource(LocalPlayer(player_entity));
                        synchronized_entities.map.insert(entity_id, player_entity);
                    } else {
                        let remote_entity = spawn_voxel_player(
                            &mut commands,
                            &mut meshes,
                            &mut materials,
                            Vec3::new(0.0, 2.0, 0.0),
                            Color::srgb(0.8, 0.2, 0.2),
                        );
                        
                        synchronized_entities.map.insert(entity_id, remote_entity);
                    }
                }
                
                NetworkMessage::PlayerDisconnected { entity_id } => {
                    if let Some(entity) = synchronized_entities.map.remove(&entity_id) {
                        commands.entity(entity).despawn();
                    }
                }
                
                NetworkMessage::PlayerStateUpdate(state) => {
                    if let Some(local_player) = &local_player {
                        if let Some(&entity) = synchronized_entities.map.get(&state.entity_id) {
                            if entity == local_player.0 {
                                input_history.remove_until(state.sequence_number);
                                
                                if let Ok((mut transform, mut physics)) = local_query.get_mut(entity) {
                                    transform.translation = state.position;
                                    transform.rotation = state.rotation;
                                    physics.velocity = state.velocity;
                                }
                                
                                continue;
                            }
                        }
                    }
                    
                    if let Some(&entity) = synchronized_entities.map.get(&state.entity_id) {
                        if let Ok((mut transform, mut physics)) = remote_query.get_mut(entity) {
                            transform.translation = state.position;
                            transform.rotation = state.rotation;
                            physics.velocity = state.velocity;
                        }
                    }
                }
                
                NetworkMessage::RigidBodyUpdate { entity_id, position, rotation } => {
                    if let Some(&entity) = synchronized_entities.map.get(&entity_id) {
                        if let Ok((mut transform, _)) = remote_query.get_mut(entity) {
                            transform.translation = position;
                            transform.rotation = rotation;
                        }
                    } else {
                        let cube_entity = commands.spawn(PbrBundle {
                            mesh: meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
                            material: materials.add(Color::srgb(0.8, 0.2, 0.2)),
                            transform: Transform::from_translation(position).with_rotation(rotation),
                            ..default()
                        }).id();
                        
                        synchronized_entities.map.insert(entity_id, cube_entity);
                    }
                }
                
                _ => {}
            }
        }
    }
}

fn update_transport(
    mut client: ResMut<RenetClient>,
    mut transport: ResMut<Transport>,
    time: Res<Time>,
) {
    let delta = time.delta();
    client.update(delta);
    
    let _ = transport.0.update(delta, &mut *client);
    let _ = transport.0.send_packets(&mut *client);
}

fn client_tick(time: Res<Time>, client: Res<RenetClient>) {
    let elapsed = time.elapsed_seconds();
    let prev_elapsed = elapsed - time.delta_seconds();
    
    if (elapsed / 2.0).floor() != (prev_elapsed / 2.0).floor() {
        if client.is_connected() {
            println!("✅ CLIENT: Connesso - Tick: {:.2}s", elapsed);
        } else {
            println!("⏳ CLIENT: In attesa di connessione...");
        }
    }
}

fn setup_level(
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

    // Camera FPS con FOV aumentato
    let camera = commands.spawn(Camera3dBundle {
        projection: Projection::Perspective(PerspectiveProjection {
            fov: 90.0_f32.to_radians(),
            ..default()
        }),
        transform: Transform::from_xyz(0.0, 2.0, 0.0),
        ..default()
    }).id();
    
    // Arma
    let weapon = commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(0.12, 0.06, 0.35)),
        material: materials.add(Color::srgb(0.15, 0.15, 0.18)),
        transform: Transform::from_xyz(0.35, -0.25, -0.5),
        ..default()
    }).id();
    
    // Mirino arma
    let sight = commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(0.04, 0.04, 0.04)),
        material: materials.add(Color::srgb(0.8, 0.1, 0.1)),
        transform: Transform::from_xyz(0.0, 0.05, -0.12),
        ..default()
    }).id();
    
    commands.entity(weapon).add_child(sight);
    commands.entity(camera).add_child(weapon);
}