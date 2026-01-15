use bevy::prelude::*;

// Componente "Player" usato sia dal server che dal client.
#[derive(Component)]
pub struct Player {
    pub id: u64,
    pub username: String,
}

pub fn hello_shared() {
    println!("🔗 SHARED: Libreria condivisa caricata correttamente!");
}