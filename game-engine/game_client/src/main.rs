// game_client/src/main.rs

use bevy::prelude::*;
use bevy_renet::RenetClientPlugin;
use game_shared::hello_shared;

mod network;
mod player;
mod camera;
mod debug;
mod level;
mod weapon;
mod hud;  // <--- QUESTO MANCAVA

fn main() {
    println!("🎮 CLIENT: Avvio grafica e connessione...");
    hello_shared();

    App::new()
        .add_plugins(
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "CHAIN GANG - Client".into(),
                    resolution: (1280., 720.).into(),
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
        
        .insert_resource(network::SynchronizedEntities::default())
        .insert_resource(player::InputHistory::default())
        .insert_resource(camera::CameraRotation::default())
        .insert_resource(camera::CameraSettings::default())
        .insert_resource(hud::PlayerHealthUI::default())

        .add_systems(Startup, (
            level::setup_level,
            network::setup_network,
            hud::setup_hud,
        ).chain())
        
        .add_systems(Update, (
            network::update_transport,
            network::receive_network_messages,
            camera::toggle_mouse_grab,
            camera::handle_mouse_look,
            player::handle_input,
            player::apply_local_prediction,
            camera::update_camera_position,
            weapon::handle_shooting,
            weapon::update_projectiles,
            weapon::cleanup_muzzle_flash,
            weapon::cleanup_hit_markers,
            hud::update_health_ui,
            debug::client_tick,
        ).chain())
        .run();
}