// game_shared/src/components.rs

use bevy::prelude::*;
// Non c'è bisogno di `use serde::{Deserialize, Serialize};` qui,
// perché i componenti stessi non vengono serializzati/deserializzati direttamente in questo file.
// Sono i messaggi di rete che li contengono a dover essere serializzabili.

/// Componente che rappresenta un giocatore nel gioco
#[derive(Component)]
pub struct Player {
    pub id: u64,
    pub username: String,
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
            jump_force: 7.5,
            grounded: false,
        }
    }
}

/// Componente per la fisica del giocatore
#[derive(Component, Clone, Copy)]
pub struct PlayerPhysics {
    pub velocity: Vec3,
    pub gravity: f32, // <--- Corretto da f332 a f32
}

impl Default for PlayerPhysics {
    fn default() -> Self {
        Self {
            velocity: Vec3::ZERO,
            gravity: -18.0,
        }
    }
}

/// Componente per la fisica generale (usato dal cubo)
#[derive(Component)]
pub struct PhysicsBody {
    pub velocity: Vec3,
    pub gravity: f32,
    pub bounciness: f32,
}

/// Componente per il collider a scatola (usato dal cubo e pavimento)
#[derive(Component)]
pub struct BoxCollider {
    pub half_extents: Vec3,
}