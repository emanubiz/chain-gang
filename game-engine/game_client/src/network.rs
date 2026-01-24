// game_client/src/network.rs

use bevy::prelude::*;
use bevy_renet::renet::{ConnectionConfig, RenetClient};
use bevy_renet::renet::transport::{NetcodeClientTransport, ClientAuthentication};

use game_shared::{
    PROTOCOL_ID, GameConfig, NetworkMessage, PlayerController, PlayerPhysics, WeaponType,
};
use std::collections::HashMap;
use std::net::UdpSocket;
use std::time::SystemTime;

use super::player::{spawn_voxel_player, LocalPlayer, InputHistory};
use super::weapon::spawn_weapon;

#[derive(Resource)]
pub struct Transport(pub NetcodeClientTransport);

#[derive(Resource)]
pub struct OurClientId(pub u64);

#[derive(Resource, Default)]
pub struct SynchronizedEntities {
    pub map: HashMap<u64, Entity>,
}

pub fn setup_network(mut commands: Commands) {
    let config = GameConfig::get();
    let server_addr = format!("{}:{}", config.server_addr, config.server_port).parse().unwrap();
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

pub fn update_transport(
    mut client: ResMut<RenetClient>,
    mut transport: ResMut<Transport>,
    time: Res<Time>,
) {
    let delta = time.delta();
    client.update(delta);
    
    let _ = transport.0.update(delta, &mut *client);
    let _ = transport.0.send_packets(&mut *client);
}

pub fn receive_network_messages(
    mut commands: Commands,
    mut client: ResMut<RenetClient>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut synchronized_entities: ResMut<SynchronizedEntities>,
    mut input_history: ResMut<InputHistory>,
    mut health_ui: ResMut<crate::hud::PlayerHealthUI>,
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
                    
                    let player_color = if our_client_id.0 == client_id {
                        Color::srgb(0.2, 0.8, 0.2)
                    } else {
                        Color::srgb(0.8, 0.2, 0.2)
                    };

                    let player_entity = spawn_voxel_player(
                        &mut commands,
                        &mut meshes,
                        &mut materials,
                        Vec3::new(0.0, 0.0, 0.0), // Spawna a Y=0 (piedi a terra)
                        player_color,
                    );
                    
                    // Se è il nostro player locale
                    if our_client_id.0 == client_id {
                        commands.entity(player_entity).insert(PlayerController::default());
                        commands.insert_resource(LocalPlayer(player_entity));

                        // Spawna l'arma e attaccala al player
                        // Per ora la attacchiamo direttamente al player entity
                        // TODO: in futuro trovare il braccio destro specifico
                        let weapon = spawn_weapon(
                            &mut commands,
                            &mut meshes,
                            &mut materials,
                            WeaponType::Rifle, // Default: Rifle
                        );
                        
                        // Attacca l'arma al player principale
                        commands.entity(player_entity).add_child(weapon);
                        println!("🔫 CLIENT: Arma Rifle spawnata per il player locale");
                    }
                    
                    synchronized_entities.map.insert(entity_id, player_entity);
                }
                
                NetworkMessage::PlayerDisconnected { entity_id } => {
                    if let Some(entity) = synchronized_entities.map.remove(&entity_id) {
                        commands.entity(entity).despawn_recursive();
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

                NetworkMessage::HealthUpdate { entity_id, current_health, max_health } => {
                    println!("💚 CLIENT: Health update - Entity {}: {}/{}", 
                        entity_id, current_health, max_health);
                    
                    // Aggiorna UI se è il nostro player
                    if let Some(local_player) = &local_player {
                        if let Some(&entity) = synchronized_entities.map.get(&entity_id) {
                            if entity == local_player.0 {
                                health_ui.current = current_health;
                                health_ui.max = max_health;
                            }
                        }
                    }
                }

                NetworkMessage::PlayerDied { entity_id, killer_id } => {
                    println!("💀 CLIENT: Player {} morto (killer: {:?})", entity_id, killer_id);
                }

                NetworkMessage::PlayerRespawn { entity_id, position } => {
                    println!("♻️  CLIENT: Player {} respawnato a {:?}", entity_id, position);
                }

                NetworkMessage::ProjectileHit { position, damage } => {
                    println!("🎯 HIT! Damage: {} at {:?}", damage, position);
                    
                    // Spawna un effetto visivo temporaneo
                    commands.spawn(PbrBundle {
                        mesh: meshes.add(Sphere::new(0.1)),
                        material: materials.add(Color::srgb(1.0, 0.0, 0.0)),
                        transform: Transform::from_translation(position),
                        ..default()
                    }).insert(super::weapon::HitMarker {
                        lifetime: 0.5,
                        elapsed: 0.0,
                    });
                }
                
                _ => {}
            }
        }
    }
}