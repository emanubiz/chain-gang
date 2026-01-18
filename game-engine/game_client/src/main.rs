// game_client/src/main.rs

use bevy::prelude::*;
use bevy_renet::RenetClientPlugin;
use game_shared::hello_shared; // Importa solo la funzione specifica da game_shared

// Dichiara i moduli locali al crate game_client
mod network;
mod player;
mod camera;
mod debug;
mod level;

fn main() {
    println!("🎮 CLIENT: Avvio grafica e connessione...");
    hello_shared(); // Chiamata alla funzione condivisa

    App::new()
        .add_plugins(
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "CHAIN GANG - Client".into(),
                    resolution: (1280., 720.).into(),
                    ..default()
                }),
                ..default()
            }),
        )
        .insert_resource(ClearColor(Color::srgb(0.15, 0.18, 0.25)))
        .add_plugins(RenetClientPlugin)
        
        // Risorse specifiche del client (ora con prefisso del modulo)
        .insert_resource(network::SynchronizedEntities::default())
        .insert_resource(player::InputHistory::default())
        .insert_resource(camera::CameraRotation::default())

        // Aggiungi i sistemi dai moduli (ora con prefisso del modulo)
        .add_systems(Startup, (level::setup_level, network::setup_network).chain())
        .add_systems(Update, (
            network::update_transport,
            camera::handle_mouse_look,
            player::handle_input,
            player::apply_local_prediction,
            camera::update_camera_position,
            network::receive_network_messages,
            debug::client_tick,
        ).chain())
        .run();
}