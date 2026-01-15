use bevy::prelude::*;
use bevy_renet::{
    renet::{
        RenetServer, ConnectionConfig, ServerAuthentication, ServerConfig,
    },
    transport::{NetcodeServerPlugin, NetcodeServerTransport, ServerAuthentication as TransportAuth, ServerConfig as TransportConfig},
    RenetServerPlugin,
};
use game_shared::{connection_config, hello_shared, PROTOCOL_ID};
use std::{net::UdpSocket, time::SystemTime};

fn main() {
    println!("🔥 SERVER: Avvio sistema di rete...");
    hello_shared();

    let mut app = App::new();
    
    app.add_plugins(MinimalPlugins); // Niente grafica
    app.add_plugins(RenetServerPlugin); // Logica pacchetti
    app.add_plugins(NetcodeServerPlugin); // Logica UDP/Internet

    // Configuriamo la rete all'avvio
    app.insert_resource(new_renet_server());
    app.insert_resource(new_netcode_server_transport());

    app.add_systems(Update, server_events); // Sistema che stampa chi si connette

    app.run();
}

fn new_renet_server() -> RenetServer {
    RenetServer::new(connection_config())
}

fn new_netcode_server_transport() -> NetcodeServerTransport {
    // Ascolta su localhost (127.0.0.1) porta 5000
    let public_addr = "127.0.0.1:5000".parse().unwrap();
    let socket = UdpSocket::bind(public_addr).unwrap();
    
    let current_time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap();
    
    let server_config = TransportConfig {
        current_time,
        max_clients: 4, // Max 4 giocatori
        protocol_id: PROTOCOL_ID,
        public_addresses: vec![public_addr],
        authentication: TransportAuth::Unsecure, // Per ora niente crittografia complessa
    };

    NetcodeServerTransport::new(server_config, socket).unwrap()
}

// Questo sistema stampa a video quando qualcuno entra!
fn server_events(mut server_events: EventReader<bevy_renet::renet::ServerEvent>) {
    for event in server_events.read() {
        match event {
            bevy_renet::renet::ServerEvent::ClientConnected { client_id } => {
                println!("✅ UN GIOCATORE E' ENTRATO! ID: {}", client_id);
            }
            bevy_renet::renet::ServerEvent::ClientDisconnected { client_id, reason } => {
                println!("❌ UN GIOCATORE E' USCITO! ID: {} | Motivo: {:?}", client_id, reason);
            }
        }
    }
}