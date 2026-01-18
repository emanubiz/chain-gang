// game_shared/src/utils.rs

use bevy::prelude::*;
use crate::components::{PlayerController, PlayerPhysics};
use crate::network_messages::PlayerInput;
use crate::character_constants::PLAYER_HEIGHT;

/// Funzione condivisa per applicare il movimento del giocatore
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
    let forward = transform.forward(); // Vettore che punta "in avanti" per il giocatore
    let right = transform.right();     // Vettore che punta "a destra" per il giocatore
    
    // Movimento orizzontale (solo X e Z)
    // 🔥 CORREZIONE PROPOSTA: Invertiamo i segni per allineare l'input WASD al movimento atteso.
    // Se W (+Y) ti portava indietro e S (-Y) avanti: inverti i segni di Y.
    // Se A (-X) ti portava a destra e D (+X) a sinistra: inverti i segni di X.
    let move_dir = (forward * -input.move_direction.y + right * -input.move_direction.x) // <--- HO INVERTITO I SEGNI QUI
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