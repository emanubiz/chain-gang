// game_client/src/debug.rs

use bevy::prelude::*;
use bevy_renet::renet::RenetClient;

pub fn client_tick(time: Res<Time>, client: Res<RenetClient>) {
    let elapsed = time.elapsed_seconds();
    let prev_elapsed = elapsed - time.delta_seconds();
    
    if (elapsed / 2.0).floor() != (prev_elapsed / 2.0).floor() {
        if client.is_connected() {
            println!("✅ CLIENT: Connesso - Tick: {:.2}s", elapsed);
        } else {
            println!("⏳ CLIENT: In attesa di connessione...");
        }
    }
}