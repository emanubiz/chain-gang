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
                    resolution: (800., 600.).into(),
                    ..default()
                }),
                ..default()
            }),
        )
        .insert_resource(ClearColor(Color::srgb(0.1, 0.1, 0.15)))
        .add_plugins(RenetClientPlugin)
        .insert_resource(SynchronizedEntities::default())
        .insert_resource(InputHistory::default())
        .add_systems(Startup, (setup_level, setup_network).chain())
        .add_systems(Update, (
            update_transport,
            handle_input,
            apply_local_prediction,
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

fn handle_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut client: ResMut<RenetClient>,
    mut input_history: ResMut<InputHistory>,
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

/// Spawna un personaggio voxel umano (stile Minecraft)
fn spawn_voxel_player(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    position: Vec3,
    color: Color,
) -> Entity {
    // Materiale del personaggio
    let material = materials.add(color);
    
    // Parent entity (punto di controllo centrale)
    let player_entity = commands.spawn((
        PlayerPhysics::default(),
        SpatialBundle {
            transform: Transform::from_translation(position),
            ..default()
        },
    )).id();
    
    // Testa (cubo 0.4x0.4x0.4)
    let head = commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(0.4, 0.4, 0.4)),
        material: material.clone(),
        transform: Transform::from_xyz(0.0, 0.7, 0.0), // Sopra il corpo
        ..default()
    }).id();
    
    // Corpo (0.5 wide, 0.7 tall, 0.3 deep)
    let body = commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(0.5, 0.7, 0.3)),
        material: material.clone(),
        transform: Transform::from_xyz(0.0, 0.2, 0.0),
        ..default()
    }).id();
    
    // Braccio sinistro
    let left_arm = commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(0.2, 0.6, 0.2)),
        material: material.clone(),
        transform: Transform::from_xyz(-0.4, 0.2, 0.0),
        ..default()
    }).id();
    
    // Braccio destro
    let right_arm = commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(0.2, 0.6, 0.2)),
        material: material.clone(),
        transform: Transform::from_xyz(0.4, 0.2, 0.0),
        ..default()
    }).id();
    
    // Gamba sinistra
    let left_leg = commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(0.2, 0.6, 0.2)),
        material: material.clone(),
        transform: Transform::from_xyz(-0.15, -0.5, 0.0),
        ..default()
    }).id();
    
    // Gamba destra
    let right_leg = commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(0.2, 0.6, 0.2)),
        material: material.clone(),
        transform: Transform::from_xyz(0.15, -0.5, 0.0),
        ..default()
    }).id();
    
    // Attacca tutte le parti al parent
    commands.entity(player_entity).push_children(&[
        head,
        body,
        left_arm,
        right_arm,
        left_leg,
        right_leg,
    ]);
    
    player_entity
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
                    
                    // Se è il giocatore locale, salvalo
                    if our_client_id.0 == client_id {
                        let player_entity = spawn_voxel_player(
                            &mut commands,
                            &mut meshes,
                            &mut materials,
                            Vec3::new(0.0, 2.0, 0.0),
                            Color::srgb(0.2, 0.8, 0.2), // Verde per il locale
                        );
                        
                        // Aggiungi il controller solo al locale
                        commands.entity(player_entity).insert(PlayerController::default());
                        commands.insert_resource(LocalPlayer(player_entity));
                        synchronized_entities.map.insert(entity_id, player_entity);
                    } else {
                        // Giocatore remoto
                        let remote_entity = spawn_voxel_player(
                            &mut commands,
                            &mut meshes,
                            &mut materials,
                            Vec3::new(0.0, 2.0, 0.0),
                            Color::srgb(0.8, 0.2, 0.2), // Rosso per i remoti
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
                    // Se è il giocatore locale, fai reconciliation
                    if let Some(local_player) = &local_player {
                        if let Some(&entity) = synchronized_entities.map.get(&state.entity_id) {
                            if entity == local_player.0 {
                                // Reconciliation: rimuovi gli input già processati
                                input_history.remove_until(state.sequence_number);
                                
                                // Aggiorna con lo stato del server
                                if let Ok((mut transform, mut physics)) = local_query.get_mut(entity) {
                                    transform.translation = state.position;
                                    transform.rotation = state.rotation;
                                    physics.velocity = state.velocity;
                                    
                                    // TODO: Riapplica gli input pendenti
                                }
                                
                                continue;
                            }
                        }
                    }
                    
                    // Giocatori remoti: aggiorna direttamente
                    if let Some(&entity) = synchronized_entities.map.get(&state.entity_id) {
                        if let Ok((mut transform, mut physics)) = remote_query.get_mut(entity) {
                            transform.translation = state.position;
                            transform.rotation = state.rotation;
                            physics.velocity = state.velocity;
                        }
                    }
                }
                
                NetworkMessage::RigidBodyUpdate { entity_id, position, rotation } => {
                    // Cubo che cade - usa una query semplice senza filtri
                    if let Some(&entity) = synchronized_entities.map.get(&entity_id) {
                        // Prova prima con remote_query (il cubo non ha PlayerController)
                        if let Ok((mut transform, _)) = remote_query.get_mut(entity) {
                            transform.translation = position;
                            transform.rotation = rotation;
                        }
                    } else {
                        // Spawna il cubo
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
    // Pavimento
    commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(20.0, 1.0, 20.0)),
        material: materials.add(Color::srgb(0.3, 0.5, 0.3)),
        transform: Transform::from_xyz(0.0, -0.5, 0.0),
        ..default()
    });

    // Luce
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });

    // Camera che segue il giocatore (semplificata per ora)
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(-5.0, 6.0, 8.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
}