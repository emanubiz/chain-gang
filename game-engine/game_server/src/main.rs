// game_server/src/main.rs - Step 1.2: Synchronized Physics (Simple Manual Physics)

use bevy::prelude::*;
use bevy::app::ScheduleRunnerPlugin;
use game_shared::{hello_shared, PROTOCOL_ID, SERVER_PORT, SERVER_ADDR, PhysicsMessage};
use bevy_renet::renet::{ConnectionConfig, RenetServer, ServerEvent};
use bevy_renet::renet::transport::{NetcodeServerTransport, ServerAuthentication, ServerConfig};
use bevy_renet::RenetServerPlugin;
use std::net::UdpSocket;
use std::time::{Duration, SystemTime};

// Wrapper Resource per NetcodeServerTransport
#[derive(Resource)]
struct Transport(NetcodeServerTransport);

// Risorsa per tracciare l'entità del cubo sincronizzato
#[derive(Resource)]
struct SynchronizedCube(Entity);

// Componente per la fisica manuale
#[derive(Component)]
struct PhysicsBody {
    velocity: Vec3,
    gravity: f32,
    bounciness: f32,
}

// Componente per i collider (AABB semplice)
#[derive(Component)]
struct BoxCollider {
    half_extents: Vec3,
}

fn main() {
    println!("🔥 SERVER: Avvio in corso...");
    hello_shared();

    App::new()
        .add_plugins(
            MinimalPlugins.set(ScheduleRunnerPlugin::run_loop(
                Duration::from_secs_f64(1.0 / 60.0) // 60 tick al secondo
            ))
        )
        .add_plugins(RenetServerPlugin)
        .add_systems(Startup, (setup_network, setup_physics).chain())
        .add_systems(Update, (
            update_transport,
            handle_server_events,
            apply_physics,        // Sistema di fisica manuale
            sync_physics_to_clients,
            server_tick,
        ).chain())
        .run();
}

fn setup_network(mut commands: Commands) {
    let server_addr = format!("{}:{}", SERVER_ADDR, SERVER_PORT).parse().unwrap();
    let current_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();

    // Crea socket UDP
    let socket = UdpSocket::bind(server_addr)
        .expect("Impossibile bindare il socket UDP del server");

    // Crea RenetServer
    let server = RenetServer::new(ConnectionConfig::default());

    // Configurazione transport
    let server_config = ServerConfig {
        current_time,
        max_clients: 64,
        protocol_id: PROTOCOL_ID,
        public_addresses: vec![server_addr],
        authentication: ServerAuthentication::Unsecure,
    };

    // Crea transport
    let transport = NetcodeServerTransport::new(server_config, socket)
        .expect("Impossibile creare NetcodeServerTransport");

    commands.insert_resource(server);
    commands.insert_resource(Transport(transport));
}

fn setup_physics(mut commands: Commands) {
    println!("🔧 SERVER: Setup fisica manuale in corso...");
    
    // Spawna il pavimento (collider statico a y = 0)
    let floor = commands.spawn((
        Transform::from_xyz(0.0, 0.0, 0.0),
        BoxCollider {
            half_extents: Vec3::new(10.0, 0.5, 10.0),
        },
    )).id();
    println!("✅ SERVER: Pavimento creato: {:?}", floor);

    // Spawna il cubo che cade
    let cube_entity = commands.spawn((
        Transform::from_xyz(0.0, 5.0, 0.0),
        PhysicsBody {
            velocity: Vec3::ZERO,
            gravity: -9.81,
            bounciness: 0.7, // Coefficiente di rimbalzo
        },
        BoxCollider {
            half_extents: Vec3::new(0.5, 0.5, 0.5),
        },
    )).id();

    commands.insert_resource(SynchronizedCube(cube_entity));
    
    println!("✅ SERVER: Cubo fisico spawnato con ID: {:?}", cube_entity);
}

fn apply_physics(
    mut query: Query<(&mut Transform, &mut PhysicsBody, &BoxCollider)>,
    time: Res<Time>,
) {
    let dt = time.delta_seconds();
    
    for (mut transform, mut body, collider) in query.iter_mut() {
        // Applica gravità
        body.velocity.y += body.gravity * dt;
        
        // Applica velocità
        transform.translation += body.velocity * dt;
        
        // Collisione semplice con il pavimento (y = 0.5, che è floor_y + floor_half_height + cube_half_height)
        let ground_level = 0.5 + collider.half_extents.y;
        
        if transform.translation.y <= ground_level {
            transform.translation.y = ground_level;
            
            // Rimbalzo
            body.velocity.y = -body.velocity.y * body.bounciness;
            
            // Se la velocità è troppo bassa, ferma il cubo
            if body.velocity.y.abs() < 0.1 {
                body.velocity.y = 0.0;
            }
        }
    }
}

fn update_transport(
    mut server: ResMut<RenetServer>,
    mut transport: ResMut<Transport>,
    time: Res<Time>,
) {
    let delta = time.delta();
    
    // Aggiorna il transport
    if let Err(e) = transport.0.update(delta, &mut *server) {
        eprintln!("❌ Errore transport: {:?}", e);
    }
    
    server.update(delta);
}

fn sync_physics_to_clients(
    mut server: ResMut<RenetServer>,
    synced_cube: Res<SynchronizedCube>,
    query: Query<(&Transform, &PhysicsBody)>,
    time: Res<Time>,
) {
    // Ottieni la transform del cubo
    if let Ok((transform, body)) = query.get(synced_cube.0) {
        let message = PhysicsMessage::RigidBodyUpdate {
            entity_id: synced_cube.0.index() as u64,
            position: transform.translation,
            rotation: transform.rotation,
        };

        // Debug: stampa la posizione del cubo ogni 2 secondi
        if (time.elapsed_seconds() / 2.0).floor() != ((time.elapsed_seconds() - time.delta_seconds()) / 2.0).floor() {
            let client_count = server.clients_id().len();
            if client_count > 0 {
                println!("🔄 Sincronizzando con {} client(i)", client_count);
            }
            println!("📦 Cubo: pos={:.2?}, vel={:.2?}, rot={:.2?}", transform.translation, body.velocity, transform.rotation);
        }

        // Serializza e invia a tutti i client connessi
        if let Ok(message_data) = bincode::serialize(&message) {
            let message_size = message_data.len();
            for client_id in server.clients_id() {
                 println!("[SERVER SEND] In coda messaggio per client {}, dimensione: {} bytes", client_id, message_size);
                server.send_message(client_id, 0, message_data.clone());
            }
        }
    }
}

fn handle_server_events(mut server_events: EventReader<ServerEvent>) {
    for event in server_events.read() {
        match event {
            ServerEvent::ClientConnected { client_id } => {
                println!("✅ SERVER: Client {} si è connesso.", client_id);
            }
            ServerEvent::ClientDisconnected { client_id, reason } => {
                println!("❌ SERVER: Client {} si è disconnesso: {:?}", client_id, reason);
            }
        }
    }
}

fn server_tick(time: Res<Time>) {
    let elapsed = time.elapsed_seconds();
    let prev_elapsed = elapsed - time.delta_seconds();
    
    if (elapsed / 2.0).floor() != (prev_elapsed / 2.0).floor() {
        println!("🔄 SERVER: Tick: {:.2}s", elapsed);
    }
}