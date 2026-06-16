use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub api_key: String,
    #[serde(default = "default_base_url")]
    pub base_url: String,
    #[serde(default = "default_model")]
    pub model: String,
    /// Tauri global-shortcut syntax, e.g. "Control+Alt+KeyD".
    #[serde(default = "default_hotkey")]
    pub hotkey: String,

    /// "api" (Typhoon cloud) or "local" (run the model on this machine).
    #[serde(default = "default_backend")]
    pub backend: String,
    /// Python interpreter used for the local backend.
    #[serde(default = "default_python")]
    pub python_path: String,
    /// HuggingFace model id for the local backend.
    #[serde(default = "default_local_model")]
    pub local_model: String,
    /// "auto" | "cpu" | "cuda" for the local backend.
    #[serde(default = "default_device")]
    pub device: String,
}

fn default_base_url() -> String {
    "https://api.opentyphoon.ai/v1".into()
}
fn default_model() -> String {
    "typhoon-asr-realtime".into()
}
fn default_hotkey() -> String {
    "Control+Alt+KeyD".into()
}
/// Default local launcher: uv builds a cached, pinned env with typhoon-asr on
/// first run — no manual install or venv needed, just `uv` on the system.
pub const DEFAULT_PYTHON_CMD: &str = "uv run --python 3.10 --with typhoon-asr python";

fn default_backend() -> String {
    "local".into()
}
fn default_python() -> String {
    DEFAULT_PYTHON_CMD.into()
}
fn default_local_model() -> String {
    "scb10x/typhoon-asr-realtime".into()
}
fn default_device() -> String {
    "auto".into()
}

impl Default for Config {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            base_url: default_base_url(),
            model: default_model(),
            hotkey: default_hotkey(),
            backend: "local".into(),
            python_path: DEFAULT_PYTHON_CMD.into(),
            local_model: default_local_model(),
            device: default_device(),
        }
    }
}

/// Directory that holds config.json and the bundled local-inference script.
pub fn data_dir() -> PathBuf {
    let mut dir = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    dir.push("ai.bedrock.tpkwhisper");
    let _ = std::fs::create_dir_all(&dir);
    dir
}

fn config_path() -> PathBuf {
    let mut p = data_dir();
    p.push("config.json");
    p
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
