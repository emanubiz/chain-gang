// game_client/src/camera.rs

use bevy::prelude::*;
use bevy::window::{CursorGrabMode, PrimaryWindow};
use game_shared::PlayerController;
use super::player::LocalPlayer;

/// Eye height of the player (from ground)
pub const EYE_HEIGHT: f32 = 1.60;

#[derive(Resource)]
pub struct CameraRotation {
    pub yaw: f32,
    pub pitch: f32,
    pub sensitivity: f32,
}

impl Default for CameraRotation {
    fn default() -> Self {
        Self {
            yaw: 0.0,
            pitch: 0.0,
            sensitivity: 0.0018,
        }
    }
}

// Kept for compatibility with main.rs (insert_resource)
#[derive(Resource, Default)]
pub struct CameraSettings;

/// Grab/release the mouse
pub fn toggle_mouse_grab(
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse_button: Res<ButtonInput<bevy::input::mouse::MouseButton>>,
    mut window_query: Query<&mut Window, With<PrimaryWindow>>,
) {
    if let Ok(mut window) = window_query.get_single_mut() {
        if keyboard.just_pressed(KeyCode::Escape) {
            window.cursor.grab_mode = CursorGrabMode::None;
            window.cursor.visible = true;
        }

        if mouse_button.just_pressed(bevy::input::mouse::MouseButton::Left) {
            if matches!(window.cursor.grab_mode, CursorGrabMode::None) {
                window.cursor.grab_mode = CursorGrabMode::Locked;
                window.cursor.visible = false;
            }
        }
    }
}

/// Mouse look — update yaw/pitch
pub fn handle_mouse_look(
    mut camera_rotation: ResMut<CameraRotation>,
    mut mouse_motion: EventReader<bevy::input::mouse::MouseMotion>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    if let Ok(window) = window_query.get_single() {
        if !matches!(window.cursor.grab_mode, CursorGrabMode::Locked) {
            mouse_motion.clear();
            return;
        }
    }

    for motion in mouse_motion.read() {
        camera_rotation.yaw -= motion.delta.x * camera_rotation.sensitivity;
        camera_rotation.pitch -= motion.delta.y * camera_rotation.sensitivity;
        camera_rotation.pitch = camera_rotation.pitch.clamp(
            -std::f32::consts::FRAC_PI_2 + 0.05,
            std::f32::consts::FRAC_PI_2 - 0.05,
        );
    }
}

/// FPS Camera: positioned at eye level, looks where mouse points
pub fn update_camera_position(
    local_player: Option<Res<LocalPlayer>>,
    player_query: Query<&Transform, With<PlayerController>>,
    mut camera_query: Query<&mut Transform, (With<Camera3d>, Without<PlayerController>)>,
    camera_rotation: Res<CameraRotation>,
) {
    if let Some(local_player) = local_player {
        if let Ok(player_transform) = player_query.get(local_player.0) {
            if let Ok(mut camera_transform) = camera_query.get_single_mut() {
                // Eye position: feet + EYE_HEIGHT
                camera_transform.translation =
                    player_transform.translation + Vec3::Y * EYE_HEIGHT;

                // Direct rotation: yaw horizontal, pitch vertical
                camera_transform.rotation =
                    Quat::from_rotation_y(camera_rotation.yaw)
                    * Quat::from_rotation_x(camera_rotation.pitch);
            }
        }
    }
}
