// game_shared/src/weapon_types.rs

use serde::{Deserialize, Serialize};

/// Tipi di armi disponibili
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum WeaponType {
    Pistol,
    Rifle,
    Shotgun,
}

/// Statistiche di un'arma
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct WeaponStats {
    pub damage: f32,
    pub fire_rate: f32,      // Secondi tra uno sparo e l'altro
    pub range: f32,          // Distanza massima efficace
    pub accuracy: f32,       // 0.0 - 1.0 (spread)
    pub projectile_speed: f32, // Velocità proiettile (per future implementazioni)
}

impl WeaponStats {
    pub fn from_type(weapon_type: WeaponType) -> Self {
        match weapon_type {
            WeaponType::Pistol => Self {
                damage: 25.0,
                fire_rate: 0.3,
                range: 30.0,
                accuracy: 0.9,
                projectile_speed: 100.0,
            },
            WeaponType::Rifle => Self {
                damage: 35.0,
                fire_rate: 0.15,
                range: 50.0,
                accuracy: 0.95,
                projectile_speed: 150.0,
            },
            WeaponType::Shotgun => Self {
                damage: 60.0,
                fire_rate: 0.8,
                range: 15.0,
                accuracy: 0.6,
                projectile_speed: 80.0,
            },
        }
    }
}

/// Componente per tracciare la salute del giocatore
#[derive(Debug, Clone, Copy, Serialize, Deserialize, bevy::prelude::Component)]
pub struct PlayerHealth {
    pub current: f32,
    pub max: f32,
}

impl Default for PlayerHealth {
    fn default() -> Self {
        Self {
            current: 100.0,
            max: 100.0,
        }
    }
}

impl PlayerHealth {
    pub fn take_damage(&mut self, damage: f32) {
        self.current = (self.current - damage).max(0.0);
    }

    pub fn is_alive(&self) -> bool {
        self.current > 0.0
    }

    pub fn heal(&mut self, amount: f32) {
        self.current = (self.current + amount).min(self.max);
    }
}