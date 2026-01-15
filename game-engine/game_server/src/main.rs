use bevy::prelude::*;
use game_shared::hello_shared;
use std::time::Duration; // Ora lo usiamo per la sleep

fn main() {
    println!("🔥 SERVER: Avvio in corso (Versione Semplificata)...");
    hello_shared();

    App::new()
        .add_plugins(MinimalPlugins)
        .add_systems(Update, server_tick)
        .run();
}

fn server_tick(time: Res<Time>) {
    // Stampiamo ogni 2 secondi circa usando println! standard
    if time.elapsed_seconds() % 2.0 < 0.02 {
        println!("Server is running... Tick: {:.2}", time.elapsed_seconds());
    }
    
    // Piccolo trucco per non far fondere la CPU in questo test semplice
    // (senza grafica il loop può andare troppo veloce)
    std::thread::sleep(Duration::from_millis(10));
}