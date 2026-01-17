// game_shared/src/lib.rs

use bevy::prelude::*;

#[derive(Component)]
pub struct Player {
    pub id: u64,
    pub username: String,
}

pub const PROTOCOL_ID: u64 = 7;
pub const SERVER_PORT: u16 = 5000;
pub const SERVER_ADDR: &str = "127.0.0.1";

pub fn hello_shared() {
    println!("🔗 SHARED: Libreria condivisa caricata correttamente!");
}