// game_client/src/camera.rs

use bevy::prelude::*;
use bevy::window::{CursorGrabMode, PrimaryWindow};
use game_shared::{PLAYER_HEIGHT, PlayerController};
use super::player::LocalPlayer;

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
            sensitivity: 0.003,
        }
    }
}

#[derive(Resource)]
pub struct CameraSettings {
    pub distance: f32,
    pub height: f32,
    pub pitch_offset: f32,
}

impl Default for CameraSettings {
    fn default() -> Self {
        Self {
            distance: 2.0,
            height: 1.5,
            pitch_offset: -0.3,
        }
    }
}

/// Sistema per gestire il grab/release del mouse
pub fn toggle_mouse_grab(
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse_button: Res<ButtonInput<bevy::input::mouse::MouseButton>>,
    mut window_query: Query<&mut Window, With<PrimaryWindow>>,
) {
    if let Ok(mut window) = window_query.get_single_mut() {
        // ESC per rilasciare il mouse
        if keyboard.just_pressed(KeyCode::Escape) {
            window.cursor.grab_mode = CursorGrabMode::None;
            window.cursor.visible = true;
            println!("🖱️  Mouse sbloccato - Click sinistro per riagganciare");
        }
        
        // Click sinistro per agganciare il mouse (solo se non è già agganciato)
        if mouse_button.just_pressed(bevy::input::mouse::MouseButton::Left) {
            if matches!(window.cursor.grab_mode, CursorGrabMode::None) {
                window.cursor.grab_mode = CursorGrabMode::Locked;
                window.cursor.visible = false;
                println!("🖱️  Mouse agganciato - ESC per rilasciare");
            }
        }
    }
}

/// Sistema per gestire il mouse look
pub fn handle_mouse_look(
    mut camera_rotation: ResMut<CameraRotation>,
    mut mouse_motion: EventReader<bevy::input::mouse::MouseMotion>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    // Leggi il movimento del mouse solo se il cursore è grabbed
    if let Ok(window) = window_query.get_single() {
        if !matches!(window.cursor.grab_mode, CursorGrabMode::Locked) {
            mouse_motion.clear();
            return;
        }
    }

    for motion in mouse_motion.read() {
        camera_rotation.yaw -= motion.delta.x * camera_rotation.sensitivity;
        camera_rotation.pitch -= motion.delta.y * camera_rotation.sensitivity;
        
        // Limita il pitch per evitare che la camera si ribalti
        camera_rotation.pitch = camera_rotation.pitch.clamp(
            -std::f32::consts::FRAC_PI_2 + 0.1,
            std::f32::consts::FRAC_PI_2 - 0.1
        );
    }
}

/// Sistema per aggiornare la posizione della camera
pub fn update_camera_position(
    local_player: Option<Res<LocalPlayer>>,
    player_query: Query<&Transform, With<PlayerController>>,
    mut camera_query: Query<&mut Transform, (With<Camera3d>, Without<PlayerController>)>,
    camera_rotation: Res<CameraRotation>,
    camera_settings: Res<CameraSettings>,
) {
    if let Some(local_player) = local_player {
        if let Ok(player_transform) = player_query.get(local_player.0) {
            if let Ok(mut camera_transform) = camera_query.get_single_mut() {
                // Calcola la rotazione combinata
                let combined_rotation = Quat::from_rotation_y(camera_rotation.yaw)
                    * Quat::from_rotation_x(camera_rotation.pitch + camera_settings.pitch_offset);

                // Calcola la posizione della camera dietro al giocatore
                let backward_vector = combined_rotation * Vec3::NEG_Z;
                camera_transform.translation = player_transform.translation
                    + Vec3::Y * camera_settings.height
                    + backward_vector * camera_settings.distance;

                // Fai guardare la camera verso il giocatore
                let look_target = player_transform.translation + Vec3::Y * (PLAYER_HEIGHT / 2.0);
                camera_transform.look_at(look_target, Vec3::Y);
            }
        }
    }
}