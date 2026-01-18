// game_shared/src/lib.rs

// Dichiarazione dei moduli interni del crate game_shared
pub mod character_constants;
pub mod components;
pub mod network_messages;
pub mod utils;
pub mod config;

// Re-esportazione degli elementi più usati per facilitare l'importazione nei crate dipendenti
pub use character_constants::*;
pub use components::*;
pub use network_messages::*;
pub use utils::*;
pub use config::*;