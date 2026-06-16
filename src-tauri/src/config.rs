use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub api_key: String,
    pub base_url: String,
    pub model: String,
    /// Tauri global-shortcut syntax, e.g. "Control+Alt+KeyD".
    pub hotkey: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            base_url: "https://api.opentyphoon.ai/v1".into(),
            model: "typhoon-asr-realtime".into(),
            hotkey: "Control+Alt+KeyD".into(),
        }
    }
}

fn config_path() -> PathBuf {
    let mut dir = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    dir.push("ai.bedrock.tpkwhisper");
    let _ = std::fs::create_dir_all(&dir);
    dir.push("config.json");
    dir
}

impl Config {
    pub fn load() -> Self {
        match std::fs::read_to_string(config_path()) {
            Ok(s) => serde_json::from_str(&s).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let s = serde_json::to_string_pretty(self)?;
        std::fs::write(config_path(), s)?;
        Ok(())
    }
}
