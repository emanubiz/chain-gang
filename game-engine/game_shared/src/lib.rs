use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Componente che rappresenta un giocatore nel gioco
#[derive(Component)]
pub struct Player {
    pub id: u64,
    pub username: String,
}

/// Input del giocatore (inviato dal client al server)
#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default)]
pub struct PlayerInput {
    pub move_direction: Vec2,  // X e Z (forward/backward, left/right)
    pub jump: bool,
    pub yaw: f32,              // Rotazione orizzontale (mouse X)
    pub pitch: f32,            // Rotazione verticale (mouse Y)
    pub sequence_number: u32,  // Per il client-side prediction
}

/// Stato del giocatore (inviato dal server ai client)
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct PlayerState {
    pub entity_id: u64,
    pub position: Vec3,
    pub velocity: Vec3,
    pub rotation: Quat,
    pub sequence_number: u32,  // L'ultimo input processato dal server
}

/// Messaggi di rete scambiati tra server e client
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum NetworkMessage {
    /// Client -> Server: Input del giocatore
    PlayerInput(PlayerInput),
    
    /// Server -> Client: Stato aggiornato del giocatore
    PlayerStateUpdate(PlayerState),
    
    /// Server -> Client: Aggiornamento di un corpo rigido generico (cubo che cade)
    RigidBodyUpdate {
        entity_id: u64,
        position: Vec3,
        rotation: Quat,
    },
    
    /// Server -> All: Un nuovo giocatore si è connesso
    PlayerConnected {
        entity_id: u64,
        client_id: u64,
    },
    
    /// Server -> All: Un giocatore si è disconnesso
    PlayerDisconnected {
        entity_id: u64,
    },
}

/// Componente per il movimento del giocatore (usato sia su client che server)
#[derive(Component, Clone, Copy)]
pub struct PlayerController {
    pub move_speed: f32,
    pub jump_force: f32,
    pub grounded: bool,
}

impl Default for PlayerController {
    fn default() -> Self {
        Self {
            move_speed: 5.0,
            jump_force: 7.5, // 🔥 Forza salto aumentata per compensare gravità
            grounded: false,
        }
    }
}

/// Componente per la fisica del giocatore
#[derive(Component, Clone, Copy)]
pub struct PlayerPhysics {
    pub velocity: Vec3,
    pub gravity: f32,
}

impl Default for PlayerPhysics {
    fn default() -> Self {
        Self {
            velocity: Vec3::ZERO,
            gravity: -18.0, // 🔥 Gravità aumentata per salti più realistici
        }
    }
}

/// Costanti di rete
pub const PROTOCOL_ID: u64 = 7;
pub const SERVER_PORT: u16 = 5000;
pub const SERVER_ADDR: &str = "127.0.0.1";

/// Costanti di gameplay
pub const PLAYER_RADIUS: f32 = 0.5;
pub const PLAYER_HEIGHT: f32 = 1.8;

// ── NUOVE COSTANTI VOXEL PERSONAGGIO ───────────────────────────────────────
pub const VOXEL_SCALE: f32 = 0.18;

pub const HEAD_SIZE: f32 = 0.44; // 🔥 Testa leggermente più grande
pub const HEAD_Y_OFFSET: f32 = 1.32; // 🔥 Abbassata per ancoraggio al suolo

pub const BODY_WIDTH: f32 = 0.40; // 🔥 Corpo un po' più largo
pub const BODY_DEPTH: f32 = 0.28;
pub const BODY_HEIGHT: f32 = 0.68;
pub const BODY_Y_OFFSET: f32 = 0.64; // 🔥 Abbassato

pub const ARM_WIDTH: f32 = 0.16;
pub const ARM_DEPTH: f32 = 0.16;
pub const ARM_HEIGHT: f32 = 0.62;
pub const ARM_Y_OFFSET: f32 = 0.64; // 🔥 Abbassato

pub const LEG_WIDTH: f32 = 0.18;
pub const LEG_DEPTH: f32 = 0.18;
pub const LEG_HEIGHT: f32 = 0.68; // 🔥 Gambe leggermente più lunghe
pub const LEG_Y_OFFSET: f32 = 0.0; // 🔥 Ancorate al suolo

// Funzione condivisa per applicare il movimento del giocatore
pub fn apply_player_movement(
    input: &PlayerInput,
    transform: &mut Transform,
    physics: &mut PlayerPhysics,
    controller: &PlayerController,
    dt: f32,
) {
    // Applica rotazione dal mouse (solo yaw, pitch è gestito dalla camera)
    transform.rotation = Quat::from_rotation_y(input.yaw);
    
    // Calcola la direzione di movimento nel mondo
    let forward = transform.forward();
    let right = transform.right();
    
    // Movimento orizzontale (solo X e Z)
    let move_dir = (forward * input.move_direction.y + right * input.move_direction.x)
        .normalize_or_zero();
    
    // Applica velocità orizzontale
    physics.velocity.x = move_dir.x * controller.move_speed;
    physics.velocity.z = move_dir.z * controller.move_speed;
    
    // Salto
    if input.jump && controller.grounded {
        physics.velocity.y = controller.jump_force;
    }
    
    // Applica gravità
    physics.velocity.y += physics.gravity * dt;
    
    // Applica velocità alla posizione
    transform.translation += physics.velocity * dt;
    
    // Controllo collisione con il pavimento (semplificato)
    if transform.translation.y <= PLAYER_HEIGHT / 2.0 {
        transform.translation.y = PLAYER_HEIGHT / 2.0;
        physics.velocity.y = 0.0;
    }
}

pub fn hello_shared() {
    println!("🔗 SHARED: Libreria condivisa caricata correttamente!");
}