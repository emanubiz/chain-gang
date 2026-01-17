// game_client/src/main.rs - Step 1.2: Synchronized Physics

use bevy::prelude::*;
use game_shared::{hello_shared, PROTOCOL_ID, SERVER_PORT, SERVER_ADDR, PhysicsMessage};
use bevy_renet::renet::{ConnectionConfig, RenetClient};
use bevy_renet::renet::transport::{NetcodeClientTransport, ClientAuthentication};
use bevy_renet::RenetClientPlugin;
use bevy_rapier3d::prelude::*;
use std::collections::HashMap;
use std::net::UdpSocket;
use std::time::SystemTime;

// Wrapper Resource per NetcodeClientTransport
#[derive(Resource)]
struct Transport(NetcodeClientTransport);

// Mappa delle entità sincronizzate (ID di rete -> Entity locale)
#[derive(Resource, Default)]
struct SynchronizedEntities {
    map: HashMap<u64, Entity>,
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
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins(RapierDebugRenderPlugin::default())
        .insert_resource(SynchronizedEntities::default())
        .add_systems(Startup, (setup_level, setup_network).chain())
        .add_systems(Update, (
            update_transport,
            receive_physics_messages,
            client_tick,
        ))
        .run();
}

fn setup_network(mut commands: Commands) {
    let server_addr = format!("{}:{}", SERVER_ADDR, SERVER_PORT).parse().unwrap();
    let current_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();

    // Generiamo un ID client casuale
    let client_id = current_time.as_millis() as u64;

    // Binda un socket UDP locale per il client
    let client_socket = UdpSocket::bind("0.0.0.0:0")
        .expect("Impossibile bindare il socket UDP del client");

    // Crea RenetClient
    let client = RenetClient::new(ConnectionConfig::default());

    // Crea il transport layer
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
}

fn update_transport(
    mut client: ResMut<RenetClient>,
    mut transport: ResMut<Transport>,
    time: Res<Time>,
) {
    let delta = time.delta();
    
    // Aggiorna il transport
    if let Err(e) = transport.0.update(delta, &mut *client) {
        eprintln!("❌ Errore transport: {:?}", e);
    }
}

fn receive_physics_messages(
    mut commands: Commands,
    mut client: ResMut<RenetClient>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut synchronized_entities: ResMut<SynchronizedEntities>,
    mut query: Query<&mut Transform>,
    time: Res<Time>,
) {
    let mut message_count = 0;
    
    // Ricevi tutti i messaggi dal canale 0 (fisica)
    while let Some(message) = client.receive_message(0) {
        message_count += 1;
        
        if let Ok(physics_msg) = bincode::deserialize::<PhysicsMessage>(&message) {
            match physics_msg {
                PhysicsMessage::RigidBodyUpdate { entity_id, position, rotation } => {
                    // Se l'entità esiste già, aggiorna la sua posizione
                    if let Some(local_entity) = synchronized_entities.map.get(&entity_id) {
                        if let Ok(mut transform) = query.get_mut(*local_entity) {
                            transform.translation = position;
                            transform.rotation = rotation;
                        }
                    } else {
                        // Altrimenti spawna una nuova entità visuale
                        println!("📦 CLIENT: Spawn cubo sincronizzato (ID: {}) a posizione {:?}", entity_id, position);
                        
                        let local_entity = commands.spawn(PbrBundle {
                            mesh: meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
                            material: materials.add(Color::srgb(0.8, 0.2, 0.2)),
                            transform: Transform::from_translation(position)
                                .with_rotation(rotation),
                            ..default()
                        }).id();
                        
                        synchronized_entities.map.insert(entity_id, local_entity);
                    }
                }
            }
        }
    }
    
    // Debug: stampa ogni 2 secondi quanti messaggi riceviamo
    if message_count > 0 && (time.elapsed_seconds() / 2.0).floor() != ((time.elapsed_seconds() - time.delta_seconds()) / 2.0).floor() {
        println!("📨 CLIENT: Ricevuti {} messaggi di fisica", message_count);
    }
}

fn client_tick(time: Res<Time>) {
    let elapsed = time.elapsed_seconds();
    let prev_elapsed = elapsed - time.delta_seconds();
    
    if (elapsed / 2.0).floor() != (prev_elapsed / 2.0).floor() {
        println!("🔄 CLIENT: Tick: {:.2}s", elapsed);
    }
}

fn setup_level(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Pavimento visuale (il server ha il collider fisico)
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

    // Camera
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(-5.0, 6.0, 8.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
}