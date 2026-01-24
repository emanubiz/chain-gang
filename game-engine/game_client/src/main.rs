// game_client/src/main.rs

use bevy::prelude::*;
use bevy_renet::RenetClientPlugin;
use game_shared::hello_shared;

mod network;
mod player;
mod camera;
mod debug;
mod level;

fn main() {
    println!("🎮 CLIENT: Avvio grafica e connessione...");
    hello_shared();

    App::new()
        .add_plugins(
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "CHAIN GANG - Client".into(),
                    resolution: (1280., 720.).into(),
                    // Imposta il cursore come grabbed all'avvio
                    cursor: bevy::window::Cursor {
                        grab_mode: bevy::window::CursorGrabMode::Locked,
                        visible: false,
                        ..default()
                    },
                    ..default()
                }),
                ..default()
            }),
        )
        .insert_resource(ClearColor(Color::srgb(0.15, 0.18, 0.25)))
        .add_plugins(RenetClientPlugin)
        
        // Risorse del client
        .insert_resource(network::SynchronizedEntities::default())
        .insert_resource(player::InputHistory::default())
        .insert_resource(camera::CameraRotation::default())
        .insert_resource(camera::CameraSettings::default())

        // Setup iniziale
        .add_systems(Startup, (
            level::setup_level,
            network::setup_network,
        ).chain())
        
        // Update loop
        .add_systems(Update, (
            network::update_transport,
            camera::toggle_mouse_grab,
            camera::handle_mouse_look,
            player::handle_input,
            player::apply_local_prediction,
            // Usa update_camera_position OPPURE smooth_camera_follow
            camera::update_camera_position,
            // camera::smooth_camera_follow, // <-- Decommenta per camera smooth
            network::receive_network_messages,
            debug::client_tick,
        ).chain())
        .run();
}