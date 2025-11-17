use std::path::PathBuf;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::cli::Args;

#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfig {
    pub threshold_ms: u64,
    #[serde(default)]
    pub key_overrides: Vec<KeyConfig>,
    #[serde(default)]
    pub keyboard_overrides: Vec<KeyboardConfig>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct KeyConfig {
    /// The key code
    pub code: u16,
    pub threshold_ms: u64,
    #[serde(default)]
    pub excluded: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct KeyboardConfig {
    pub keyboard_name: String,
    pub threshold_ms: u64,
    #[serde(default)]
    pub key_overrides: Vec<KeyConfig>,
}

impl AppConfig {
    pub fn load() -> Result<Option<Self>> {
        // TODO: Better path handling
        let path = PathBuf::from("config.toml");
        if !path.exists() {
            return Ok(None);
        }

        let raw = std::fs::read_to_string(path)?;
        let config = toml::from_str(&raw)?;
        return Ok(config);
    }

    pub fn save(&self) -> Result<()> {
        let path = "config.toml";
        let raw = toml::to_string(&self)?;
        std::fs::write(path, &raw)?;

        Ok(())
    }

    pub fn merge_args(&mut self, args: &Args) {
        if let Some(threshold) = args.threshold {
            self.threshold_ms = threshold;
        }
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            threshold_ms: 30,
            key_overrides: vec![],
            keyboard_overrides: vec![],
        }
    }
}
