// game_client/src/camera.rs

use bevy::prelude::*;
// Importa direttamente i tipi necessari da game_shared
use game_shared::{PLAYER_HEIGHT, PlayerController};

// Importa dal modulo locale 'player'
use super::player::LocalPlayer;

#[derive(Resource, Default)]
pub struct CameraRotation {
    pub yaw: f32,   // Rotazione orizzontale
    pub pitch: f32, // Rotazione verticale
}

pub fn handle_mouse_look(
    mut camera_rotation: ResMut<CameraRotation>,
    mut mouse_motion: EventReader<bevy::input::mouse::MouseMotion>,
    mouse_button: Res<ButtonInput<bevy::input::mouse::MouseButton>>,
) {
    if mouse_button.pressed(bevy::input::mouse::MouseButton::Right) {
        for motion in mouse_motion.read() {
            let sensitivity = 0.003;
            
            camera_rotation.yaw -= motion.delta.x * sensitivity;
            camera_rotation.pitch -= motion.delta.y * sensitivity;
            
            camera_rotation.pitch = camera_rotation.pitch.clamp(-1.5, 1.5);
        }
    }
}

pub fn update_camera_position(
    local_player: Option<Res<LocalPlayer>>,
    player_query: Query<&Transform, With<PlayerController>>,
    mut camera_query: Query<&mut Transform, (With<Camera3d>, Without<PlayerController>)>,
    camera_rotation: Res<CameraRotation>,
) {
    if let Some(local_player) = local_player {
        if let Ok(player_transform) = player_query.get(local_player.0) {
            if let Ok(mut camera_transform) = camera_query.get_single_mut() {
                let camera_distance = 2.0; // 🎯 Ridotto da 3.5 a 2.0 per avvicinare la telecamera
                let camera_height = 1.5;   // 🎯 Ridotto da 2.0 a 1.5 per abbassarla leggermente
                let camera_pitch_offset = -0.3; 

                let combined_rotation = Quat::from_rotation_y(camera_rotation.yaw)
                    * Quat::from_rotation_x(camera_rotation.pitch + camera_pitch_offset);

                let backward_vector = combined_rotation * Vec3::NEG_Z;
                camera_transform.translation = player_transform.translation
                    + Vec3::Y * camera_height
                    + backward_vector * camera_distance;

                camera_transform.look_at(player_transform.translation + Vec3::Y * (PLAYER_HEIGHT / 2.0), Vec3::Y);
            }
        }
    }
}