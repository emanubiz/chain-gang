use bevy::prelude::*;
use bevy_renet::{
    renet::{RenetClient, ConnectionConfig, ClientAuthentication},
    transport::{NetcodeClientPlugin, NetcodeClientTransport, ClientAuthentication as TransportAuth},
    RenetClientPlugin,
};
use game_shared::{connection_config, hello_shared, PROTOCOL_ID};
use std::{net::UdpSocket, time::SystemTime};

fn main() {
    println!("🎮 CLIENT: Connessione al server...");
    hello_shared();

    let mut app = App::new();

    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "CHAIN GANG - Client".into(),
            resolution: (800., 600.).into(),
            ..default()
        }),
        ..default()
    }));
    
    // Plugin di rete
    app.add_plugins(RenetClientPlugin);
    app.add_plugins(NetcodeClientPlugin);

    // Setup risorse rete
    app.insert_resource(new_renet_client());
    app.insert_resource(new_netcode_client_transport());

    // Setup grafica base (Cubo e Luce)
    app.insert_resource(ClearColor(Color::srgb(0.1, 0.1, 0.15)));
    app.add_systems(Startup, setup_level);

    app.run();
}

fn new_renet_client() -> RenetClient {
    RenetClient::new(connection_config())
}

fn new_netcode_client_transport() -> NetcodeClientTransport {
    // Si connette a localhost:5000
    let server_addr = "127.0.0.1:5000".parse().unwrap();
    let socket = UdpSocket::bind("127.0.0.1:0").unwrap(); // Porta 0 = dammene una a caso libera
    
    let current_time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap();
    let client_id = current_time.as_millis() as u64; // ID temporaneo basato sul tempo
    
    let authentication = TransportAuth::Unsecure {
        server_addr,
        client_id,
        user_data: None,
        protocol_id: PROTOCOL_ID,
    };

    NetcodeClientTransport::new(current_time, authentication, socket).unwrap()
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
    // Luce e Camera
    commands.spawn(PointLightBundle {
        point_light: PointLight { intensity: 1500.0, shadows_enabled: true, ..default() },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(-5.0, 6.0, 8.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
}