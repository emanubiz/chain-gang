// game_client/src/player.rs

use bevy::prelude::*;
// Importa direttamente i tipi necessari da game_shared
use game_shared::{
    PlayerController, PlayerPhysics, PlayerInput, NetworkMessage,
    // Importa le costanti del personaggio che servono a spawn_voxel_player
    VOXEL_SCALE, HEAD_SIZE, HEAD_Y_OFFSET, BODY_WIDTH, BODY_DEPTH, BODY_HEIGHT, BODY_Y_OFFSET,
    ARM_WIDTH, ARM_DEPTH, ARM_HEIGHT, ARM_Y_OFFSET, LEG_WIDTH, LEG_DEPTH, LEG_HEIGHT, LEG_Y_OFFSET,
    PLAYER_HEIGHT, // Per apply_local_prediction
    apply_player_movement, // Per apply_local_prediction
};
use bevy_renet::renet::RenetClient;
use std::collections::VecDeque;

use super::camera::CameraRotation;

#[derive(Resource)]
pub struct LocalPlayer(pub Entity);

/// Storico degli input inviati (per reconciliation)
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
        
        // Mantieni solo gli ultimi 100 input
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
    
    // Correzione per invertire i controlli WASD
    // W (Avanti) dovrebbe aumentare Y
    // S (Indietro) dovrebbe diminuire Y
    // A (Sinistra) dovrebbe diminuire X
    // D (Destra) dovrebbe aumentare X

    if keyboard.pressed(KeyCode::KeyW) {
        move_direction.y += 1.0; // W dovrebbe muovere in avanti
    }
    if keyboard.pressed(KeyCode::KeyS) {
        move_direction.y -= 1.0; // S dovrebbe muovere all'indietro
    }
    if keyboard.pressed(KeyCode::KeyA) {
        move_direction.x -= 1.0; // A dovrebbe muovere a sinistra
    }
    if keyboard.pressed(KeyCode::KeyD) {
        move_direction.x += 1.0; // D dovrebbe muovere a destra
    }
    
    // Se i controlli erano completamente invertiti rispetto a questi,
    // potresti aver bisogno di invertire i segni di tutti i += 1.0 e -= 1.0
    // O più semplicemente, scambiare i valori:
    /*
    if keyboard.pressed(KeyCode::KeyW) {
        move_direction.y -= 1.0; // Se W andava indietro
    }
    if keyboard.pressed(KeyCode::KeyS) {
        move_direction.y += 1.0; // Se S andava avanti
    }
    if keyboard.pressed(KeyCode::KeyA) {
        move_direction.x += 1.0; // Se A andava a destra
    }
    if keyboard.pressed(KeyCode::KeyD) {
        move_direction.x -= 1.0; // Se D andava a sinistra
    }
    */
    // La versione sopra è quella che dovrebbe già essere corretta per WASD standard.
    // Se la tua descrizione significa che i segni di default sono sbagliati, allora sono già a posto.
    // Se invece i tasti stessi sono scambiati (cioè premi W e vai con S), allora devi SCAMBIARE le righe:

    /*
    // Esempio se W va indietro e S va avanti:
    if keyboard.pressed(KeyCode::KeyW) {
        move_direction.y -= 1.0; // W va indietro
    }
    if keyboard.pressed(KeyCode::KeyS) {
        move_direction.y += 1.0; // S va avanti
    }
    // Esempio se A va a destra e D va a sinistra:
    if keyboard.pressed(KeyCode::KeyA) {
        move_direction.x += 1.0; // A va a destra
    }
    if keyboard.pressed(KeyCode::KeyD) {
        move_direction.x -= 1.0; // D va a sinistra
    }
    */
    // Ho lasciato la versione standard corretta in cui W=+Y, S=-Y, A=-X, D=+X.
    // Se la tua esperienza è che sono invertiti *rispetto a questa*, allora la versione
    // che hai postato è quella giusta per i controlli standard.
    // Se invece significa che premi W e l'input Y diventa -1.0, allora il problema è a monte.

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
                
                controller.grounded = transform.translation.y <= PLAYER_HEIGHT / 2.0 + 0.01;
                
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

pub fn spawn_voxel_player(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    position: Vec3,
    base_color: Color,
) -> Entity {
    // Tutte le costanti del personaggio sono importate da game_shared

    let style_variants = [
        (
            Color::srgb(0.96, 0.82, 0.70),
            Color::srgb(1.0, 0.85, 0.0),
            Color::srgb(0.1, 0.6, 0.9),
            Color::srgb(0.2, 0.2, 0.8),
            true,
            false,
        ),
        (
            Color::srgb(0.98, 0.78, 0.69),
            Color::srgb(0.8, 0.2, 0.2),
            Color::srgb(0.9, 0.5, 0.7),
            Color::srgb(0.3, 0.2, 0.6),
            false,
            false,
        ),
        (
            Color::srgb(0.85, 0.65, 0.50),
            Color::srgb(0.15, 0.12, 0.10),
            Color::srgb(0.9, 0.1, 0.1),
            Color::srgb(0.15, 0.15, 0.15),
            false,
            true,
        ),
    ];
    
    let variant_index = match (base_color.to_srgba().red > 0.5, base_color.to_srgba().green > 0.5, base_color.to_srgba().blue > 0.5) {
        (_, true, _) => 0,
        (true, _, _) => 1,
        (_, _, true) => 2,
        _ => 0,
    };
    
    let (skin_color, hair_color, shirt_color, pants_color, has_glasses, has_cap) = style_variants[variant_index];

    let skin_mat = materials.add(skin_color);
    let hair_mat = materials.add(hair_color);
    let shirt_mat = materials.add(shirt_color);
    let pants_mat = materials.add(pants_color);
    let shoe_mat = materials.add(Color::srgb(0.95, 0.95, 0.95));
    let accessory_mat = materials.add(Color::srgb(0.1, 0.1, 0.1));

    let parent = commands.spawn((
        SpatialBundle {
            transform: Transform::from_translation(position),
            ..default()
        },
        PlayerPhysics::default(),
    )).id();

    let head = commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(HEAD_SIZE, HEAD_SIZE, HEAD_SIZE)),
        material: skin_mat.clone(),
        transform: Transform::from_xyz(0.0, HEAD_Y_OFFSET, 0.0),
        ..default()
    }).id();

    let eye_size = VOXEL_SCALE * 1.8;
    let eye_white_mat = materials.add(Color::srgb(1.0, 1.0, 1.0));
    let pupil_mat = materials.add(Color::srgb(0.05, 0.05, 0.1));
    
    let left_eye_white = commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(eye_size, eye_size * 1.2, VOXEL_SCALE * 0.8)),
        material: eye_white_mat.clone(),
        transform: Transform::from_xyz(-0.13, 0.12, HEAD_SIZE/2.0 + 0.03),
        ..default()
    }).id();
    
    commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(eye_size * 0.6, eye_size * 0.6, VOXEL_SCALE * 0.4)),
        material: pupil_mat.clone(),
        transform: Transform::from_xyz(0.0, -0.05, 0.05),
        ..default()
    }).set_parent(left_eye_white);
    
    let right_eye_white = commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(eye_size, eye_size * 1.2, VOXEL_SCALE * 0.8)),
        material: eye_white_mat,
        transform: Transform::from_xyz(0.13, 0.12, HEAD_SIZE/2.0 + 0.03),
        ..default()
    }).id();
    
    commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(eye_size * 0.6, eye_size * 0.6, VOXEL_SCALE * 0.4)),
        material: pupil_mat,
        transform: Transform::from_xyz(0.0, -0.05, 0.05),
        ..default()
    }).set_parent(right_eye_white);
    
    commands.entity(head).push_children(&[left_eye_white, right_eye_white]);

    let mouth_mat = materials.add(Color::srgb(0.85, 0.3, 0.35));
    commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(0.20, 0.05, 0.04)),
        material: mouth_mat,
        transform: Transform::from_xyz(0.0, -0.10, HEAD_SIZE/2.0 + 0.02),
        ..default()
    }).set_parent(head);

    let hair_main = commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(HEAD_SIZE * 1.1, 0.25, HEAD_SIZE * 1.1)),
        material: hair_mat.clone(),
        transform: Transform::from_xyz(0.0, HEAD_SIZE/2.0 + 0.10, 0.0),
        ..default()
    }).id();
    
    commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(0.15, 0.12, 0.12)),
        material: hair_mat.clone(),
        transform: Transform::from_xyz(0.08, 0.10, HEAD_SIZE/2.0 + 0.05),
        ..default()
    }).set_parent(head);
    
    commands.entity(head).add_child(hair_main);

    if has_glasses {
        let glasses_mat = materials.add(Color::srgb(0.1, 0.1, 0.1));
        commands.spawn(PbrBundle {
            mesh: meshes.add(Cuboid::new(0.30, 0.12, 0.04)),
            material: glasses_mat,
            transform: Transform::from_xyz(0.0, 0.10, HEAD_SIZE/2.0 + 0.04),
            ..default()
        }).set_parent(head);
    } else if has_cap {
        commands.spawn(PbrBundle {
            mesh: meshes.add(Cuboid::new(HEAD_SIZE * 1.15, 0.10, HEAD_SIZE * 1.15)),
            material: accessory_mat.clone(),
            transform: Transform::from_xyz(0.0, HEAD_SIZE/2.0 + 0.22, 0.0),
            ..default()
        }).set_parent(head);
        
        commands.spawn(PbrBundle {
            mesh: meshes.add(Cuboid::new(HEAD_SIZE * 1.2, 0.02, 0.25)),
            material: accessory_mat,
            transform: Transform::from_xyz(0.0, HEAD_SIZE/2.0 + 0.18, HEAD_SIZE/2.0 + 0.15),
            ..default()
        }).set_parent(head);
    }

    let body = commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(BODY_WIDTH, BODY_HEIGHT, BODY_DEPTH)),
        material: shirt_mat.clone(),
        transform: Transform::from_xyz(0.0, BODY_Y_OFFSET, 0.0),
        ..default()
    }).id();

    let arm_offset_x = BODY_WIDTH / 2.0 + ARM_WIDTH / 2.0;
    commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(ARM_WIDTH, ARM_HEIGHT, ARM_DEPTH)),
        material: shirt_mat.clone(),
        transform: Transform::from_xyz(-arm_offset_x, ARM_Y_OFFSET, 0.0),
        ..default()
    }).set_parent(body);

    commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(ARM_WIDTH, ARM_HEIGHT, ARM_DEPTH)),
        material: shirt_mat.clone(),
        transform: Transform::from_xyz(arm_offset_x, ARM_Y_OFFSET, 0.0),
        ..default()
    }).set_parent(body);

    let leg_offset_x = BODY_WIDTH * 0.25;
    commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(LEG_WIDTH, LEG_HEIGHT, LEG_DEPTH)),
        material: pants_mat.clone(),
        transform: Transform::from_xyz(-leg_offset_x, LEG_Y_OFFSET, 0.0),
        ..default()
    }).set_parent(body);

    commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(LEG_WIDTH, LEG_HEIGHT, LEG_DEPTH)),
        material: pants_mat,
        transform: Transform::from_xyz(leg_offset_x, LEG_Y_OFFSET, 0.0),
        ..default()
    }).set_parent(body);

    let shoe_height = 0.10;
    let shoe_platform = 0.04;
    
    let left_shoe = commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(LEG_WIDTH * 1.15, shoe_height, LEG_DEPTH * 1.3)),
        material: shoe_mat.clone(),
        transform: Transform::from_xyz(-leg_offset_x, shoe_height/2.0, 0.05),
        ..default()
    }).id();
    
    commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(LEG_WIDTH * 1.2, shoe_platform, LEG_DEPTH * 1.35)),
        material: materials.add(Color::srgb(0.2, 0.2, 0.2)),
        transform: Transform::from_xyz(0.0, -shoe_height/2.0 - shoe_platform/2.0, 0.0),
        ..default()
    }).set_parent(left_shoe);
    
    let right_shoe = commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(LEG_WIDTH * 1.15, shoe_height, LEG_DEPTH * 1.3)),
        material: shoe_mat,
        transform: Transform::from_xyz(leg_offset_x, shoe_height/2.0, 0.05),
        ..default()
    }).id();
    
    commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(LEG_WIDTH * 1.2, shoe_platform, LEG_DEPTH * 1.35)),
        material: materials.add(Color::srgb(0.2, 0.2, 0.2)),
        transform: Transform::from_xyz(0.0, -shoe_height/2.0 - shoe_platform/2.0, 0.0),
        ..default()
    }).set_parent(right_shoe);

    commands.entity(body).push_children(&[left_shoe, right_shoe]);
    commands.entity(parent).push_children(&[head, body]);

    parent
}