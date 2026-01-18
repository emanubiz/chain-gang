// game_server/src/main.rs - Step 1.3: Player Movement (Server)

use bevy::prelude::*;
use bevy::app::ScheduleRunnerPlugin;
// Importa direttamente gli elementi necessari dal crate game_shared modularizzato
use game_shared::{
    hello_shared,
    GameConfig,
    Player, PlayerController, PlayerPhysics,
    PROTOCOL_ID,
    NetworkMessage,
    PhysicsBody, BoxCollider,
    PLAYER_HEIGHT, // Per la collisione con il pavimento
};
use bevy_renet::renet::{ConnectionConfig, RenetServer, ServerEvent};
use bevy_renet::renet::transport::{NetcodeServerTransport, ServerAuthentication, ServerConfig};
use bevy_renet::RenetServerPlugin;
use std::collections::HashMap;
use std::net::UdpSocket;
use std::time::{Duration, SystemTime};

#[derive(Resource)]
struct Transport(NetcodeServerTransport);

/// Mappa Client ID -> Player Entity
#[derive(Resource, Default)]
struct PlayerRegistry {
    map: HashMap<u64, Entity>,
}

/// Risorsa per il cubo che cade (separato dai giocatori)
#[derive(Resource)]
struct FallingCube(Entity);

fn main() {
    println!("🔥 SERVER: Avvio in corso...");
    hello_shared();

    App::new()
        .add_plugins(
            MinimalPlugins.set(ScheduleRunnerPlugin::run_loop(
                Duration::from_secs_f64(1.0 / 60.0)
            ))
        )
        .add_plugins(RenetServerPlugin)
        .insert_resource(PlayerRegistry::default())
        .add_systems(Startup, (setup_network, setup_level).chain())
        .add_systems(Update, (
            handle_server_events,
            handle_player_inputs,
            apply_player_physics,
            apply_cube_physics,
            sync_to_clients,
            update_transport,
            server_tick,
        ).chain())
        .run();
}

fn setup_network(mut commands: Commands) {
    let config = GameConfig::get();
    let server_addr = format!("{}:{}", config.server_addr, config.server_port).parse().unwrap();
    let current_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();

    let socket = UdpSocket::bind(server_addr)
        .expect("Impossibile bindare il socket UDP del server");

    println!("✅ SERVER: Socket UDP bindato su {}", server_addr);

    let server = RenetServer::new(ConnectionConfig::default());

    let server_config = ServerConfig {
        current_time,
        max_clients: 64,
        protocol_id: PROTOCOL_ID,
        public_addresses: vec![server_addr],
        authentication: ServerAuthentication::Unsecure,
    };

    let transport = NetcodeServerTransport::new(server_config, socket)
        .expect("Impossibile creare NetcodeServerTransport");

    commands.insert_resource(server);
    commands.insert_resource(Transport(transport));
}

fn setup_level(mut commands: Commands) {
    println!("🔧 SERVER: Setup livello in corso...");
    
    // Pavimento (collider statico)
    commands.spawn((
        Transform::from_xyz(0.0, 0.0, 0.0),
        BoxCollider {
            half_extents: Vec3::new(10.0, 0.5, 10.0),
        },
    ));

    // Cubo che cade (fisica separata dai giocatori)
    let cube_entity = commands.spawn((
        Transform::from_xyz(0.0, 5.0, 0.0),
        PhysicsBody {
            velocity: Vec3::ZERO,
            gravity: -9.81,
            bounciness: 0.7,
        },
        BoxCollider {
            half_extents: Vec3::new(0.5, 0.5, 0.5),
        },
    )).id();

    commands.insert_resource(FallingCube(cube_entity));
    
    println!("✅ SERVER: Livello pronto");
}

fn handle_server_events(
    mut commands: Commands,
    mut server: ResMut<RenetServer>,
    mut server_events: EventReader<ServerEvent>,
    mut player_registry: ResMut<PlayerRegistry>,
    existing_players_query: Query<(Entity, &Transform, &Player)>,
) {
    for event in server_events.read() {
        match event {
            ServerEvent::ClientConnected { client_id } => {
                let client_id_u64 = client_id.raw();
                println!("✅ SERVER: Client {} connesso!", client_id_u64);
                
                // Spawna l'entità del giocatore
                let player_entity = commands.spawn((
                    Transform::from_xyz(0.0, 2.0, 0.0),
                    PlayerController::default(),
                    PlayerPhysics::default(),
                    Player {
                        id: client_id_u64,
                        username: format!("Player{}", client_id_u64),
                    },
                )).id();
                
                player_registry.map.insert(client_id_u64, player_entity);
                
                // 1. Invia al nuovo client la lista dei giocatori già presenti
                for (entity, transform, player) in existing_players_query.iter() {
                    let msg = NetworkMessage::PlayerConnected {
                        entity_id: entity.index() as u64,
                        client_id: player.id,
                    };
                    if let Ok(data) = bincode::serialize(&msg) {
                        server.send_message(*client_id, 0, data.clone());
                    }
                    
                    // Invia anche la posizione corrente
                    let state_msg = NetworkMessage::PlayerStateUpdate(game_shared::network_messages::PlayerState {
                        entity_id: entity.index() as u64,
                        position: transform.translation,
                        velocity: Vec3::ZERO,
                        rotation: transform.rotation,
                        sequence_number: 0,
                    });
                    if let Ok(data) = bincode::serialize(&state_msg) {
                        server.send_message(*client_id, 0, data);
                    }
                }
                
                // 2. Notifica TUTTI i client del nuovo giocatore
                let msg = NetworkMessage::PlayerConnected {
                    entity_id: player_entity.index() as u64,
                    client_id: client_id_u64,
                };
                
                if let Ok(data) = bincode::serialize(&msg) {
                    server.broadcast_message(0, data);
                }
                
                println!("👤 SERVER: Spawned player entity {:?} for client {}", player_entity, client_id_u64);
            }
            
            ServerEvent::ClientDisconnected { client_id, reason } => {
                let client_id_u64 = client_id.raw();
                println!("❌ SERVER: Client {} disconnesso: {:?}", client_id_u64, reason);
                
                if let Some(player_entity) = player_registry.map.remove(&client_id_u64) {
                    commands.entity(player_entity).despawn();
                    
                    // Notifica tutti i client
                    let msg = NetworkMessage::PlayerDisconnected {
                        entity_id: player_entity.index() as u64,
                    };
                    
                    if let Ok(data) = bincode::serialize(&msg) {
                        server.broadcast_message(0, data);
                    }
                }
            }
        }
    }
}

fn handle_player_inputs(
    mut server: ResMut<RenetServer>,
    player_registry: Res<PlayerRegistry>,
    mut query: Query<(&mut Transform, &mut PlayerPhysics, &mut PlayerController)>,
    time: Res<Time>,
) {
    let dt = time.delta_seconds();
    
    // Per ogni client connesso, processa gli input
    for client_id in server.clients_id() {
        let client_id_u64 = client_id.raw();
        
        while let Some(message) = server.receive_message(client_id, 0) {
            if let Ok(NetworkMessage::PlayerInput(input)) = bincode::deserialize::<NetworkMessage>(&message) {
                // Trova l'entità del giocatore
                if let Some(&player_entity) = player_registry.map.get(&client_id_u64) {
                    if let Ok((mut transform, mut physics, mut controller)) = query.get_mut(player_entity) {
                        // Controlla se il giocatore è a terra
                        controller.grounded = transform.translation.y <= PLAYER_HEIGHT / 2.0 + 0.01;
                        
                        // Applica il movimento (funzione condivisa con il client)
                        game_shared::apply_player_movement( // Usa il percorso completo per la funzione
                            &input,
                            &mut transform,
                            &mut physics,
                            &controller,
                            dt
                        );
                    }
                }
            }
        }
    }
}

fn apply_player_physics(
    mut query: Query<(&mut Transform, &mut PlayerPhysics, &mut PlayerController)>,
) {
    for (mut transform, mut physics, mut controller) in query.iter_mut() {
        // Collisione con il pavimento
        if transform.translation.y <= PLAYER_HEIGHT / 2.0 {
            transform.translation.y = PLAYER_HEIGHT / 2.0;
            physics.velocity.y = 0.0;
            controller.grounded = true;
        } else {
            controller.grounded = false;
        }
    }
}

fn apply_cube_physics(
    mut query: Query<(&mut Transform, &mut PhysicsBody, &BoxCollider)>,
    time: Res<Time>,
) {
    let dt = time.delta_seconds();
    
    for (mut transform, mut body, collider) in query.iter_mut() {
        body.velocity.y += body.gravity * dt;
        transform.translation += body.velocity * dt;
        
        let ground_level = 0.5 + collider.half_extents.y;
        
        if transform.translation.y <= ground_level {
            transform.translation.y = ground_level;
            body.velocity.y = -body.velocity.y * body.bounciness;
            
            if body.velocity.y.abs() < 0.1 {
                body.velocity.y = 0.0;
            }
        }
    }
}

fn sync_to_clients(
    mut server: ResMut<RenetServer>,
    player_query: Query<(Entity, &Transform, &PlayerPhysics, &Player)>,
    cube_query: Query<(Entity, &Transform), With<PhysicsBody>>,
    falling_cube: Res<FallingCube>,
) {
    // Invia lo stato di ogni giocatore
    for (entity, transform, physics, _player) in player_query.iter() {
        let state = game_shared::network_messages::PlayerState { // Specifica il percorso completo
            entity_id: entity.index() as u64,
            position: transform.translation,
            velocity: physics.velocity,
            rotation: transform.rotation,
            sequence_number: 0, // TODO: tracciare sequence number
        };
        
        let msg = NetworkMessage::PlayerStateUpdate(state);
        
        if let Ok(data) = bincode::serialize(&msg) {
            server.broadcast_message(0, data);
        }
    }
    
    // Invia lo stato del cubo che cade
    if let Ok((entity, transform)) = cube_query.get(falling_cube.0) {
        let msg = NetworkMessage::RigidBodyUpdate {
            entity_id: entity.index() as u64,
            position: transform.translation,
            rotation: transform.rotation,
        };
        
        if let Ok(data) = bincode::serialize(&msg) {
            server.broadcast_message(0, data);
        }
    }
}

fn update_transport(
    mut server: ResMut<RenetServer>,
    mut transport: ResMut<Transport>,
    time: Res<Time>,
) {
    let delta = time.delta();
    server.update(delta);
    
    if let Err(e) = transport.0.update(delta, &mut *server) {
        eprintln!("❌ Errore transport update: {:?}", e);
    }
    
    transport.0.send_packets(&mut *server);
}

fn server_tick(time: Res<Time>) {
    let elapsed = time.elapsed_seconds();
    let prev_elapsed = elapsed - time.delta_seconds();
    
    if (elapsed / 2.0).floor() != (prev_elapsed / 2.0).floor() {
        println!("🔄 SERVER: Tick: {:.2}s", elapsed);
    }
}