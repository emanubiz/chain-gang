// game_shared/src/network_messages.rs

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use crate::weapon_types::WeaponType;

/// Costanti di rete
pub const PROTOCOL_ID: u64 = 7;

/// Input del giocatore (inviato dal client al server)
#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default)]
pub struct PlayerInput {
    pub move_direction: Vec2,
    pub jump: bool,
    pub yaw: f32,
    pub pitch: f32,
    pub sequence_number: u32,
}

/// Stato del giocatore (inviato dal server ai client)
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct PlayerState {
    pub entity_id: u64,
    pub position: Vec3,
    pub velocity: Vec3,
    pub rotation: Quat,
    pub sequence_number: u32,
}

/// Messaggi di rete scambiati tra server e client
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum NetworkMessage {
    /// Client -> Server: Input del giocatore
    PlayerInput(PlayerInput),
    
    /// Server -> Client: Stato aggiornato del giocatore
    PlayerStateUpdate(PlayerState),
    
    /// Server -> Client: Aggiornamento di un corpo rigido generico
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

    /// Client -> Server: Il giocatore ha sparato
    PlayerShoot {
        origin: Vec3,
        direction: Vec3,
        weapon_type: WeaponType,
    },

    /// Server -> All: Un proiettile ha colpito qualcosa
    ProjectileHit {
        position: Vec3,
        damage: f32,
    },

    /// Server -> Client: Aggiornamento salute
    HealthUpdate {
        entity_id: u64,
        current_health: f32,
        max_health: f32,
    },

    /// Server -> All: Un giocatore è morto
    PlayerDied {
        entity_id: u64,
        killer_id: Option<u64>,
    },

    /// Server -> Client: Respawn del giocatore
    PlayerRespawn {
        entity_id: u64,
        position: Vec3,
    },
}