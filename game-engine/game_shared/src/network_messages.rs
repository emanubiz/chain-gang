// game_shared/src/network_messages.rs

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Costanti di rete
pub const PROTOCOL_ID: u64 = 7;
pub const SERVER_PORT: u16 = 5000;
pub const SERVER_ADDR: &str = "127.0.0.1";

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