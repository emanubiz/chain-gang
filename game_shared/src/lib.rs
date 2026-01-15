use bevy::prelude::*;
use bevy_renet::renet::{
    ChannelConfig, ClientId, ConnectionConfig, SendType,
};
use std::time::Duration;

pub const PROTOCOL_ID: u64 = 7; // La password segreta del gioco

// Definiamo i tipi di canali
pub enum Channel {
    Reliable,   // Per cose importanti (Join, Chat, Spawn)
    Unreliable, // Per cose veloci (Movimento, Posizione)
}

impl From<Channel> for u8 {
    fn from(channel: Channel) -> Self {
        match channel {
            Channel::Reliable => 0,
            Channel::Unreliable => 1,
        }
    }
}

impl Channel {
    pub fn id(self) -> u8 {
        self.into()
    }
}

pub fn connection_config() -> ConnectionConfig {
    ConnectionConfig {
        available_bytes_per_tick: 1024 * 1024,
        client_channels_config: vec![
            ChannelConfig {
                channel_id: Channel::Unreliable.id(),
                max_memory_usage_bytes: 5 * 1024 * 1024,
                send_type: SendType::Unreliable,
            },
            ChannelConfig {
                channel_id: Channel::Reliable.id(),
                max_memory_usage_bytes: 5 * 1024 * 1024,
                send_type: SendType::ReliableOrdered {
                    resend_time: Duration::from_millis(300),
                },
            },
        ],
        server_channels_config: vec![
            ChannelConfig {
                channel_id: Channel::Unreliable.id(),
                max_memory_usage_bytes: 10 * 1024 * 1024,
                send_type: SendType::Unreliable,
            },
            ChannelConfig {
                channel_id: Channel::Reliable.id(),
                max_memory_usage_bytes: 10 * 1024 * 1024,
                send_type: SendType::ReliableOrdered {
                    resend_time: Duration::from_millis(300),
                },
            },
        ],
    }
}

pub fn hello_shared() {
    println!("🔗 SHARED: Configurazione Rete Caricata.");
}