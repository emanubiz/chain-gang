// game_shared/src/components.rs

use bevy::prelude::*;

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
            move_speed: 20.0,  // VELOCITA
            jump_force: 24.0,  // SALTO
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
            gravity: -150.0, // GRAVITÀ
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