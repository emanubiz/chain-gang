// game_shared/src/config.rs

use serde::Deserialize;
use std::{path::PathBuf, sync::OnceLock};

#[derive(Deserialize, Debug)]
pub struct GameConfig {
    pub server_addr: String,
    pub server_port: u16,
}

static CONFIG: OnceLock<GameConfig> = OnceLock::new();

impl GameConfig {
    fn load() -> Self {
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let mut workspace_root = manifest_dir.clone();

        // Traverse up to find the workspace root
        loop {
            let cargo_toml = workspace_root.join("Cargo.toml");
            if cargo_toml.exists() {
                let content = std::fs::read_to_string(&cargo_toml)
                    .expect("Failed to read Cargo.toml");
                if content.contains("[workspace]") {
                    break;
                }
            }
            if !workspace_root.pop() {
                panic!("Could not find workspace root");
            }
        }

        let config_path = workspace_root.join("config").join("development.toml");

        let config = config::Config::builder()
            .add_source(config::File::from(config_path))
            .build()
            .expect("Failed to build config");

        config.try_deserialize()
            .expect("Failed to deserialize config")
    }

    pub fn get() -> &'static Self {
        CONFIG.get_or_init(Self::load)
    }
}