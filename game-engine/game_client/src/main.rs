// game_client/src/main.rs

use bevy::prelude::*;
use game_shared::{hello_shared, PROTOCOL_ID, SERVER_PORT, SERVER_ADDR};
use bevy_renet::renet::{ConnectionConfig, RenetClient};
use bevy_renet::renet::transport::{NetcodeClientTransport, ClientAuthentication};
use bevy_renet::RenetClientPlugin;
use std::net::UdpSocket;
use std::time::SystemTime;

// Wrapper Resource per NetcodeClientTransport
#[derive(Resource)]
struct Transport(NetcodeClientTransport);

fn main() {
    println!("🎮 CLIENT: Avvio grafica e connessione...");
    hello_shared();

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
        .add_systems(Startup, setup_level)
        .add_plugins(RenetClientPlugin)
        .insert_resource(client)
        .insert_resource(Transport(transport))
        .add_systems(Update, (update_system, handle_client_events, client_tick))
        .run();
}

fn update_system(
    mut client: ResMut<RenetClient>,
    mut transport: ResMut<Transport>,
    time: Res<Time>,
) {
    let delta = time.delta();
    
    // Aggiorna il transport - dereference ResMut per ottenere &mut RenetClient
    if let Err(e) = transport.0.update(delta, &mut *client) {
        eprintln!("Errore transport: {:?}", e);
    }
}

fn client_tick(time: Res<Time>) {
    if time.elapsed_seconds() % 2.0 < 0.02 {
        println!("🔄 CLIENT: Tick: {:.2}", time.elapsed_seconds());
    }
}

fn handle_client_events(client: Res<RenetClient>) {
    if client.is_connected() {
        // Connesso
    } else if client.is_disconnected() {
        // Disconnesso
    }
}

fn setup_level(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(20.0, 1.0, 20.0)),
        material: materials.add(Color::srgb(0.3, 0.5, 0.3)),
        transform: Transform::from_xyz(0.0, -0.5, 0.0),
        ..default()
    });

    commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
        material: materials.add(Color::srgb(0.8, 0.2, 0.2)),
        transform: Transform::from_xyz(0.0, 0.5, 0.0),
        ..default()
    });

    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });

    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(-5.0, 6.0, 8.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
}