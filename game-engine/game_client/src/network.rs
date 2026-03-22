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
    /// Contatore connessioni: usato per assegnare la skin in ordine
    pub player_count: u32,
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
    mut hit_marker_ui: ResMut<crate::hud::HitMarkerUI>,
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

                    // Skin basata sull'ordine di connessione (0-3 ciclico)
                    let variant_index = (synchronized_entities.player_count % 4) as usize;
                    synchronized_entities.player_count += 1;

                    let player_entity = spawn_voxel_player(
                        &mut commands,
                        &mut meshes,
                        &mut materials,
                        Vec3::ZERO,
                        variant_index,
                    );

                    if our_client_id.0 == client_id {
                        // ── Giocatore locale ───────────────────────────────
                        commands.entity(player_entity).insert(PlayerController::default());
                        commands.insert_resource(LocalPlayer(player_entity));

                        // Nascondi il modello 3D del giocatore locale:
                        // la visuale FPS non deve vedere il proprio corpo
                        commands.entity(player_entity).insert(Visibility::Hidden);

                        // L'arma FPS (vista in prima persona) è attaccata alla camera
                        // in setup_fps_weapon() — vedi weapon.rs
                        println!("✅ CLIENT: Giocatore locale spawned (skin {})", variant_index);
                    } else {
                        // ── Giocatore remoto ───────────────────────────────
                        // Spawna arma 3D sul modello (visibile agli altri)
                        spawn_weapon(
                            &mut commands,
                            &mut meshes,
                            &mut materials,
                            WeaponType::Rifle,
                            player_entity,
                        );
                        println!("👾 CLIENT: Giocatore remoto spawned (skin {})", variant_index);
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
                                    // Applica correzione server solo se significativa (riduce micro-jitter)
                                    let correction = (state.position - transform.translation).length();
                                    if correction > 0.15 {
                                        transform.translation = state.position;
                                    }
                                    transform.rotation = state.rotation;
                                    physics.velocity = state.velocity;
                                }
                                continue;
                            }
                        }
                    }

                    if let Some(&entity) = synchronized_entities.map.get(&state.entity_id) {
                        if let Ok((mut transform, mut physics)) = remote_query.get_mut(entity) {
                            // Lerp per movimento fluido dei giocatori remoti (riduce stuttering)
                            transform.translation = transform.translation.lerp(state.position, 0.25);
                            transform.rotation = transform.rotation.slerp(state.rotation, 0.25);
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
                            material: materials.add(Color::srgb(0.75, 0.18, 0.18)),
                            transform: Transform::from_translation(position).with_rotation(rotation),
                            ..default()
                        }).id();
                        synchronized_entities.map.insert(entity_id, cube_entity);
                    }
                }

                NetworkMessage::HealthUpdate { entity_id, current_health, max_health } => {
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
                    if let Some(&entity) = synchronized_entities.map.get(&entity_id) {
                        // Nascondi il modello solo per i giocatori remoti
                        // (il locale è già Hidden in FPS)
                        let is_local = local_player.as_ref().map(|lp| lp.0 == entity).unwrap_or(false);
                        if !is_local {
                            commands.entity(entity).insert(Visibility::Hidden);
                        }
                    }
                }

                NetworkMessage::PlayerRespawn { entity_id, position } => {
                    println!("♻️  CLIENT: Player {} respawnato a {:?}", entity_id, position);
                    if let Some(&entity) = synchronized_entities.map.get(&entity_id) {
                        // Teletrasporta direttamente senza lerp
                        if let Ok((mut transform, _)) = remote_query.get_mut(entity) {
                            transform.translation = position;
                        }
                        if let Ok((mut transform, _)) = local_query.get_mut(entity) {
                            transform.translation = position;
                        }

                        // Rendi di nuovo visibile il giocatore remoto al punto di respawn
                        let is_local = local_player.as_ref().map(|lp| lp.0 == entity).unwrap_or(false);
                        if !is_local {
                            commands.entity(entity).insert(Visibility::Inherited);
                        }
                    }
                }

                NetworkMessage::ProjectileHit { position, damage } => {
                    println!("🎯 HIT! -{:.0} HP a {:?}", damage, position);
                    // Scintille 3D al punto di impatto
                    super::weapon::spawn_hit_effect(
                        &mut commands,
                        &mut meshes,
                        &mut materials,
                        position,
                        damage,
                    );
                    // Mostra il numero danno nel HUD (resetta se già attivo)
                    hit_marker_ui.damage = damage;
                    hit_marker_ui.elapsed = 0.0;
                    hit_marker_ui.active = true;
                }

                _ => {}
            }
        }
    }
}
