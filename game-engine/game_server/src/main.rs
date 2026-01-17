// game_server/src/main.rs

use bevy::prelude::*;
use game_shared::{hello_shared, PROTOCOL_ID, SERVER_PORT, SERVER_ADDR};
use bevy_renet::renet::{ConnectionConfig, RenetServer, ServerEvent};
use bevy_renet::renet::transport::{
    NetcodeServerTransport, ServerAuthentication, ServerConfig,
};
use bevy_renet::RenetServerPlugin;
use std::net::UdpSocket;
use std::time::SystemTime;

// Wrapper Resource per NetcodeServerTransport
#[derive(Resource)]
struct Transport(NetcodeServerTransport);

fn main() {
    println!("🔥 SERVER: Avvio in corso...");
    hello_shared();

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

    App::new()
        .add_plugins(MinimalPlugins)
        .add_plugins(RenetServerPlugin)
        .insert_resource(server)
        .insert_resource(Transport(transport))
        .add_systems(Update, (update_system, server_tick, handle_server_events))
        .run();
}

fn update_system(
    mut server: ResMut<RenetServer>,
    mut transport: ResMut<Transport>,
    time: Res<Time>,
) {
    let delta = time.delta();
    
    // Aggiorna il transport - dereference ResMut per ottenere &mut RenetServer
    if let Err(e) = transport.0.update(delta, &mut *server) {
        eprintln!("Errore transport: {:?}", e);
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
    if time.elapsed_seconds() % 2.0 < 0.02 {
        println!("🔄 SERVER: Tick: {:.2}", time.elapsed_seconds());
    }
}