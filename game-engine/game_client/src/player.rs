// game_client/src/player.rs

use bevy::prelude::*;
use game_shared::{
    PlayerController, PlayerPhysics, PlayerInput, NetworkMessage,
    VOXEL_SCALE, HEAD_SIZE, HEAD_Y_OFFSET, BODY_WIDTH, BODY_DEPTH, BODY_HEIGHT, BODY_Y_OFFSET,
    ARM_WIDTH, ARM_DEPTH, ARM_HEIGHT, ARM_Y_OFFSET, LEG_WIDTH, LEG_DEPTH, LEG_HEIGHT, LEG_Y_OFFSET,
    SHOE_Y_OFFSET,
    apply_player_movement,
};
use bevy_renet::renet::RenetClient;
use std::collections::VecDeque;

use super::camera::CameraRotation;

#[derive(Resource)]
pub struct LocalPlayer(pub Entity);

#[derive(Resource, Default)]
pub struct InputHistory {
    pub inputs: VecDeque<(u32, PlayerInput)>,
    pub next_sequence: u32,
}

impl InputHistory {
    pub fn add(&mut self, input: PlayerInput) -> u32 {
        let seq = self.next_sequence;
        self.next_sequence += 1;

        let mut input_with_seq = input;
        input_with_seq.sequence_number = seq;

        self.inputs.push_back((seq, input_with_seq));

        while self.inputs.len() > 100 {
            self.inputs.pop_front();
        }

        seq
    }

    pub fn remove_until(&mut self, sequence_number: u32) {
        while let Some((seq, _)) = self.inputs.front() {
            if *seq <= sequence_number {
                self.inputs.pop_front();
            } else {
                break;
            }
        }
    }
}

pub fn handle_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut client: ResMut<RenetClient>,
    mut input_history: ResMut<InputHistory>,
    camera_rotation: Res<CameraRotation>,
    local_player: Option<Res<LocalPlayer>>,
) {
    if local_player.is_none() || !client.is_connected() {
        return;
    }

    let mut move_direction = Vec2::ZERO;

    if keyboard.pressed(KeyCode::KeyW) {
        move_direction.y += 1.0;
    }
    if keyboard.pressed(KeyCode::KeyS) {
        move_direction.y -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyA) {
        move_direction.x -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyD) {
        move_direction.x += 1.0;
    }

    if move_direction.length() > 0.0 {
        move_direction = move_direction.normalize();
    }

    let jump = keyboard.pressed(KeyCode::Space);

    let input = PlayerInput {
        move_direction,
        jump,
        yaw: camera_rotation.yaw,
        pitch: camera_rotation.pitch,
        sequence_number: 0,
    };

    let seq = input_history.add(input);

    let mut input_with_seq = input;
    input_with_seq.sequence_number = seq;

    let msg = NetworkMessage::PlayerInput(input_with_seq);
    if let Ok(data) = bincode::serialize(&msg) {
        client.send_message(0, data);
    }
}

pub fn apply_local_prediction(
    mut query: Query<(&mut Transform, &mut PlayerPhysics, &mut PlayerController)>,
    local_player: Option<Res<LocalPlayer>>,
    input_history: Res<InputHistory>,
    time: Res<Time>,
) {
    if let Some(local_player) = local_player {
        if let Ok((mut transform, mut physics, mut controller)) = query.get_mut(local_player.0) {
            if let Some((_, input)) = input_history.inputs.back() {
                let dt = time.delta_seconds();

                // Y=0 = piedi a terra
                controller.grounded = transform.translation.y <= 0.01;

                apply_player_movement(
                    input,
                    &mut transform,
                    &mut physics,
                    &controller,
                    dt
                );
            }
        }
    }
}

/// Spawna un personaggio voxel.
/// variant_index 0..3 seleziona la skin in base all'ordine di connessione.
/// parent entity a Y=0 = piedi a terra.
pub fn spawn_voxel_player(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    position: Vec3,
    variant_index: usize,
) -> Entity {
    // 4 personaggi distinti
    // (skin, hair, shirt, pants, has_glasses, has_cap)
    let style_variants: [(Color, Color, Color, Color, bool, bool); 4] = [
        // 0 — Scout: chiaro, capelli biondi, occhiali, camicia blu
        (
            Color::srgb(0.96, 0.82, 0.70),
            Color::srgb(1.0, 0.85, 0.0),
            Color::srgb(0.1, 0.55, 0.90),
            Color::srgb(0.15, 0.15, 0.75),
            true,
            false,
        ),
        // 1 — Brawler: carnagione media, capelli rossi, niente accessori
        (
            Color::srgb(0.98, 0.78, 0.69),
            Color::srgb(0.82, 0.18, 0.12),
            Color::srgb(0.85, 0.30, 0.10),
            Color::srgb(0.25, 0.10, 0.45),
            false,
            false,
        ),
        // 2 — Veteran: carnagione scura, capelli neri, berretto, camicia rossa
        (
            Color::srgb(0.50, 0.35, 0.25),
            Color::srgb(0.10, 0.08, 0.06),
            Color::srgb(0.80, 0.08, 0.08),
            Color::srgb(0.10, 0.10, 0.10),
            false,
            true,
        ),
        // 3 — Sniper: carnagione media, capelli castani, mimetica verde
        (
            Color::srgb(0.88, 0.72, 0.58),
            Color::srgb(0.40, 0.25, 0.12),
            Color::srgb(0.18, 0.45, 0.15),
            Color::srgb(0.15, 0.30, 0.12),
            false,
            false,
        ),
    ];

    let idx = variant_index % 4;
    let (skin_color, hair_color, shirt_color, pants_color, has_glasses, has_cap) = style_variants[idx];

    let skin_mat = materials.add(skin_color);
    let hair_mat = materials.add(hair_color);
    let shirt_mat = materials.add(shirt_color);
    let pants_mat = materials.add(pants_color);
    let shoe_mat = materials.add(Color::srgb(0.92, 0.92, 0.92));
    let sole_mat = materials.add(Color::srgb(0.18, 0.18, 0.18));
    let accessory_mat = materials.add(Color::srgb(0.10, 0.10, 0.10));

    // Parent: posizione piedi (Y = position.y = 0 quando a terra)
    let parent = commands.spawn((
        SpatialBundle {
            transform: Transform::from_translation(position),
            ..default()
        },
        PlayerPhysics::default(),
    )).id();

    // ── TESTA ──────────────────────────────────────────────────────────
    let head = commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(HEAD_SIZE, HEAD_SIZE, HEAD_SIZE)),
        material: skin_mat.clone(),
        transform: Transform::from_xyz(0.0, HEAD_Y_OFFSET, 0.0),
        ..default()
    }).id();

    let eye_size = VOXEL_SCALE * 1.8;
    let eye_white_mat = materials.add(Color::srgb(1.0, 1.0, 1.0));
    let pupil_mat = materials.add(Color::srgb(0.05, 0.05, 0.10));

    let left_eye = commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(eye_size, eye_size * 1.2, VOXEL_SCALE * 0.8)),
        material: eye_white_mat.clone(),
        transform: Transform::from_xyz(-0.13, 0.12, HEAD_SIZE / 2.0 + 0.03),
        ..default()
    }).id();
    commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(eye_size * 0.6, eye_size * 0.6, VOXEL_SCALE * 0.4)),
        material: pupil_mat.clone(),
        transform: Transform::from_xyz(0.0, -0.04, 0.05),
        ..default()
    }).set_parent(left_eye);

    let right_eye = commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(eye_size, eye_size * 1.2, VOXEL_SCALE * 0.8)),
        material: eye_white_mat,
        transform: Transform::from_xyz(0.13, 0.12, HEAD_SIZE / 2.0 + 0.03),
        ..default()
    }).id();
    commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(eye_size * 0.6, eye_size * 0.6, VOXEL_SCALE * 0.4)),
        material: pupil_mat,
        transform: Transform::from_xyz(0.0, -0.04, 0.05),
        ..default()
    }).set_parent(right_eye);

    let mouth_mat = materials.add(Color::srgb(0.80, 0.25, 0.30));
    let mouth = commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(0.18, 0.04, 0.04)),
        material: mouth_mat,
        transform: Transform::from_xyz(0.0, -0.10, HEAD_SIZE / 2.0 + 0.02),
        ..default()
    }).id();

    let hair_top = commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(HEAD_SIZE * 1.1, 0.24, HEAD_SIZE * 1.1)),
        material: hair_mat.clone(),
        transform: Transform::from_xyz(0.0, HEAD_SIZE / 2.0 + 0.10, 0.0),
        ..default()
    }).id();
    let hair_back = commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(0.14, 0.14, 0.12)),
        material: hair_mat.clone(),
        transform: Transform::from_xyz(0.06, 0.10, HEAD_SIZE / 2.0 + 0.05),
        ..default()
    }).id();

    commands.entity(head).push_children(&[left_eye, right_eye, mouth, hair_top, hair_back]);

    if has_glasses {
        let glasses = commands.spawn(PbrBundle {
            mesh: meshes.add(Cuboid::new(0.32, 0.11, 0.04)),
            material: accessory_mat.clone(),
            transform: Transform::from_xyz(0.0, 0.11, HEAD_SIZE / 2.0 + 0.045),
            ..default()
        }).id();
        commands.entity(head).add_child(glasses);
    } else if has_cap {
        let cap_base = commands.spawn(PbrBundle {
            mesh: meshes.add(Cuboid::new(HEAD_SIZE * 1.18, 0.09, HEAD_SIZE * 1.18)),
            material: accessory_mat.clone(),
            transform: Transform::from_xyz(0.0, HEAD_SIZE / 2.0 + 0.20, 0.0),
            ..default()
        }).id();
        let cap_brim = commands.spawn(PbrBundle {
            mesh: meshes.add(Cuboid::new(HEAD_SIZE * 1.22, 0.03, 0.26)),
            material: accessory_mat.clone(),
            transform: Transform::from_xyz(0.0, HEAD_SIZE / 2.0 + 0.17, HEAD_SIZE / 2.0 + 0.16),
            ..default()
        }).id();
        commands.entity(head).push_children(&[cap_base, cap_brim]);
    }

    // ── CORPO ──────────────────────────────────────────────────────────
    let body = commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(BODY_WIDTH, BODY_HEIGHT, BODY_DEPTH)),
        material: shirt_mat.clone(),
        transform: Transform::from_xyz(0.0, BODY_Y_OFFSET, 0.0),
        ..default()
    }).id();

    // Dettaglio: tasche sulla camicia
    let pocket_mat = materials.add({
        let c = shirt_color.to_srgba();
        Color::srgb((c.red * 0.75).min(1.0), (c.green * 0.75).min(1.0), (c.blue * 0.75).min(1.0))
    });
    let pocket = commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(0.10, 0.08, 0.03)),
        material: pocket_mat,
        transform: Transform::from_xyz(-0.10, 0.16, BODY_DEPTH / 2.0 + 0.02),
        ..default()
    }).id();
    commands.entity(body).add_child(pocket);

    // ── BRACCIA ────────────────────────────────────────────────────────
    let arm_offset_x = BODY_WIDTH / 2.0 + ARM_WIDTH / 2.0;
    let left_arm = commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(ARM_WIDTH, ARM_HEIGHT, ARM_DEPTH)),
        material: shirt_mat.clone(),
        transform: Transform::from_xyz(-arm_offset_x, ARM_Y_OFFSET, 0.0),
        ..default()
    }).id();
    // mano sinistra
    let left_hand = commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(ARM_WIDTH * 1.1, ARM_WIDTH * 0.9, ARM_DEPTH * 1.1)),
        material: skin_mat.clone(),
        transform: Transform::from_xyz(0.0, -ARM_HEIGHT / 2.0 - ARM_WIDTH * 0.4, 0.0),
        ..default()
    }).id();
    commands.entity(left_arm).add_child(left_hand);

    let right_arm = commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(ARM_WIDTH, ARM_HEIGHT, ARM_DEPTH)),
        material: shirt_mat.clone(),
        transform: Transform::from_xyz(arm_offset_x, ARM_Y_OFFSET, 0.0),
        ..default()
    }).id();
    // mano destra
    let right_hand = commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(ARM_WIDTH * 1.1, ARM_WIDTH * 0.9, ARM_DEPTH * 1.1)),
        material: skin_mat.clone(),
        transform: Transform::from_xyz(0.0, -ARM_HEIGHT / 2.0 - ARM_WIDTH * 0.4, 0.0),
        ..default()
    }).id();
    commands.entity(right_arm).add_child(right_hand);

    commands.entity(body).push_children(&[left_arm, right_arm]);

    // ── GAMBE ──────────────────────────────────────────────────────────
    let leg_offset_x = BODY_WIDTH * 0.25;
    let left_leg = commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(LEG_WIDTH, LEG_HEIGHT, LEG_DEPTH)),
        material: pants_mat.clone(),
        transform: Transform::from_xyz(-leg_offset_x, LEG_Y_OFFSET, 0.0),
        ..default()
    }).id();
    let right_leg = commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(LEG_WIDTH, LEG_HEIGHT, LEG_DEPTH)),
        material: pants_mat,
        transform: Transform::from_xyz(leg_offset_x, LEG_Y_OFFSET, 0.0),
        ..default()
    }).id();
    commands.entity(body).push_children(&[left_leg, right_leg]);

    // ── SCARPE ─────────────────────────────────────────────────────────
    // SHOE_Y_OFFSET = -1.07 relativo al centro corpo (BODY_Y_OFFSET=1.16)
    // → shoe center assoluto = 1.16 - 1.07 = 0.09
    // → shoe bottom = 0.09 - 0.05 = 0.04 (sopra piattaforma)
    // → platform bottom = 0.0  ✓ (a terra)
    let shoe_height = 0.10_f32;
    let shoe_platform = 0.04_f32;

    let left_shoe = commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(LEG_WIDTH * 1.15, shoe_height, LEG_DEPTH * 1.35)),
        material: shoe_mat.clone(),
        transform: Transform::from_xyz(-leg_offset_x, SHOE_Y_OFFSET, 0.06),
        ..default()
    }).id();
    commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(LEG_WIDTH * 1.22, shoe_platform, LEG_DEPTH * 1.42)),
        material: sole_mat.clone(),
        transform: Transform::from_xyz(0.0, -shoe_height / 2.0 - shoe_platform / 2.0, 0.0),
        ..default()
    }).set_parent(left_shoe);

    let right_shoe = commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(LEG_WIDTH * 1.15, shoe_height, LEG_DEPTH * 1.35)),
        material: shoe_mat,
        transform: Transform::from_xyz(leg_offset_x, SHOE_Y_OFFSET, 0.06),
        ..default()
    }).id();
    commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(LEG_WIDTH * 1.22, shoe_platform, LEG_DEPTH * 1.42)),
        material: sole_mat,
        transform: Transform::from_xyz(0.0, -shoe_height / 2.0 - shoe_platform / 2.0, 0.0),
        ..default()
    }).set_parent(right_shoe);

    commands.entity(body).push_children(&[left_shoe, right_shoe]);
    commands.entity(parent).push_children(&[head, body]);

    parent
}
