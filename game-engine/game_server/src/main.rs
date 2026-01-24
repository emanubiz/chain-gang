// game_server/src/main.rs

use bevy::prelude::*;
use bevy::app::ScheduleRunnerPlugin;
use game_shared::{
    hello_shared, GameConfig, Player, PlayerController, PlayerPhysics, PlayerHealth,
    PROTOCOL_ID, NetworkMessage, PhysicsBody, BoxCollider, PLAYER_HEIGHT, WeaponStats,
};
use bevy_renet::renet::{ConnectionConfig, RenetServer, ServerEvent};
use bevy_renet::renet::transport::{NetcodeServerTransport, ServerAuthentication, ServerConfig};
use bevy_renet::RenetServerPlugin;
use std::collections::HashMap;
use std::net::UdpSocket;
use std::time::{Duration, SystemTime};

#[derive(Resource)]
struct Transport(NetcodeServerTransport);

#[derive(Resource, Default)]
struct PlayerRegistry {
    map: HashMap<u64, Entity>,
}

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
            handle_shooting,
            apply_player_physics,
            apply_cube_physics,
            check_player_deaths,
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
    
    commands.spawn((
        Transform::from_xyz(0.0, 0.0, 0.0),
        BoxCollider {
            half_extents: Vec3::new(25.0, 0.5, 25.0), // Mappa più grande: 50x50
        },
    ));

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
    
    println!("✅ SERVER: Livello pronto (Mappa 50x50)");
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
                
                let player_entity = commands.spawn((
                    Transform::from_xyz(0.0, 2.0, 0.0),
                    PlayerController::default(),
                    PlayerPhysics::default(),
                    PlayerHealth::default(),
                    Player {
                        id: client_id_u64,
                        username: format!("Player{}", client_id_u64),
                    },
                )).id();
                
                player_registry.map.insert(client_id_u64, player_entity);
                
                for (entity, transform, player) in existing_players_query.iter() {
                    let msg = NetworkMessage::PlayerConnected {
                        entity_id: entity.index() as u64,
                        client_id: player.id,
                    };
                    if let Ok(data) = bincode::serialize(&msg) {
                        server.send_message(*client_id, 0, data.clone());
                    }
                    
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
    
    for client_id in server.clients_id() {
        let client_id_u64 = client_id.raw();
        
        // CANALE 0 solo per PlayerInput
        while let Some(message) = server.receive_message(client_id, 0) {
            if let Ok(NetworkMessage::PlayerInput(input)) = bincode::deserialize::<NetworkMessage>(&message) {
                if let Some(&player_entity) = player_registry.map.get(&client_id_u64) {
                    if let Ok((mut transform, mut physics, mut controller)) = query.get_mut(player_entity) {
                        controller.grounded = transform.translation.y <= PLAYER_HEIGHT / 2.0 + 0.01;
                        
                        game_shared::apply_player_movement(
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

fn handle_shooting(
    mut server: ResMut<RenetServer>,
    player_registry: Res<PlayerRegistry>,
    mut player_query: Query<(&Transform, &mut PlayerHealth, &Player)>,
) {
    let mut shots_to_process = Vec::new();

    for client_id in server.clients_id() {
        let client_id_u64 = client_id.raw();
        
        // USA CANALE 1 per shooting invece di 0
        while let Some(message) = server.receive_message(client_id, 1) {
            if let Ok(NetworkMessage::PlayerShoot { origin, direction, weapon_type }) = bincode::deserialize::<NetworkMessage>(&message) {
                shots_to_process.push((client_id_u64, origin, direction, weapon_type));
                println!("🔫 SERVER: Ricevuto sparo da client {}", client_id_u64);
            }
        }
    }

    for (shooter_id, origin, direction, weapon_type) in shots_to_process {
        let stats = WeaponStats::from_type(weapon_type);
        
        println!("💥 SERVER: Player {} sparato con {:?}", shooter_id, weapon_type);

        let mut closest_hit: Option<(Entity, Vec3, f32)> = None;
        let mut min_distance = f32::MAX;

        for (target_transform, _, target_player) in player_query.iter() {
            if target_player.id == shooter_id {
                continue;
            }

            let to_target = target_transform.translation - origin;
            let distance = to_target.length();

            if distance > stats.range {
                continue;
            }

            let dot = direction.normalize().dot(to_target.normalize());
            if dot < 0.95 {
                continue;
            }

            let closest_point = origin + direction.normalize() * distance;
            let distance_to_ray = (target_transform.translation - closest_point).length();

            if distance_to_ray < PLAYER_HEIGHT && distance < min_distance {
                min_distance = distance;
                closest_hit = Some((
                    player_registry.map.iter()
                        .find(|(_, &e)| player_query.get(e).map(|(_, _, p)| p.id) == Ok(target_player.id))
                        .map(|(_, &e)| e)
                        .unwrap(),
                    target_transform.translation,
                    stats.damage
                ));
            }
        }

        if let Some((hit_entity, hit_pos, damage)) = closest_hit {
            if let Ok((_, mut health, target_player)) = player_query.get_mut(hit_entity) {
                health.take_damage(damage);
                
                println!("🎯 SERVER: Player {} colpito da Player {}! Danno: {} HP, Rimasti: {}/{}", 
                    target_player.id, shooter_id, damage, health.current, health.max);

                // Invia aggiornamento vita al player colpito
                let health_msg = NetworkMessage::HealthUpdate {
                    entity_id: hit_entity.index() as u64,
                    current_health: health.current,
                    max_health: health.max,
                };
                if let Ok(data) = bincode::serialize(&health_msg) {
                    server.broadcast_message(0, data);
                }

                // Invia feedback visivo a tutti
                let hit_msg = NetworkMessage::ProjectileHit {
                    position: hit_pos,
                    damage,
                };
                if let Ok(data) = bincode::serialize(&hit_msg) {
                    server.broadcast_message(0, data);
                }
            }
        } else {
            println!("💨 SERVER: Player {} ha mancato il colpo", shooter_id);
        }
    }
}

fn check_player_deaths(
    mut server: ResMut<RenetServer>,
    mut commands: Commands,
    query: Query<(Entity, &PlayerHealth, &Player)>,
) {
    for (entity, health, player) in query.iter() {
        if !health.is_alive() {
            println!("💀 SERVER: Player {} morto!", player.id);

            let msg = NetworkMessage::PlayerDied {
                entity_id: entity.index() as u64,
                killer_id: None,
            };
            if let Ok(data) = bincode::serialize(&msg) {
                server.broadcast_message(0, data);
            }

            commands.entity(entity)
                .insert(PlayerHealth::default())
                .insert(Transform::from_xyz(0.0, 2.0, 0.0));

            let respawn_msg = NetworkMessage::PlayerRespawn {
                entity_id: entity.index() as u64,
                position: Vec3::new(0.0, 2.0, 0.0),
            };
            if let Ok(data) = bincode::serialize(&respawn_msg) {
                server.broadcast_message(0, data);
            }
        }
    }
}

fn apply_player_physics(
    mut query: Query<(&mut Transform, &mut PlayerPhysics, &mut PlayerController)>,
) {
    for (mut transform, mut physics, mut controller) in query.iter_mut() {
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
    for (entity, transform, physics, _player) in player_query.iter() {
        let state = game_shared::network_messages::PlayerState {
            entity_id: entity.index() as u64,
            position: transform.translation,
            velocity: physics.velocity,
            rotation: transform.rotation,
            sequence_number: 0,
        };
        
        let msg = NetworkMessage::PlayerStateUpdate(state);
        
        if let Ok(data) = bincode::serialize(&msg) {
            server.broadcast_message(0, data);
        }
    }
    
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