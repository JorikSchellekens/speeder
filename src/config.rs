use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub speed: SpeedConfig,
    pub display: DisplayConfig,
    pub hotkeys: HotkeyConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeedConfig {
    pub start_wpm: u32,
    pub target_wpm: u32,
    pub warmup_words: u32,  // Number of words to reach target speed
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayConfig {
    pub font_size: f32,
    pub orp_position: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotkeyConfig {
    pub start_reading: Vec<String>,
    pub pause_resume: Vec<String>,
    pub speed_up: Vec<String>,
    pub speed_down: Vec<String>,
    pub quit: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            speed: SpeedConfig {
                start_wpm: 300,
                target_wpm: 400,
                warmup_words: 10,  // Reach full speed after 10 words
            },
            display: DisplayConfig {
                font_size: 48.0,
                orp_position: 0.33,
            },
            hotkeys: HotkeyConfig {
                start_reading: vec!["cmd".to_string(), "shift".to_string(), "r".to_string()],
                pause_resume: vec!["space".to_string()],
                speed_up: vec!["up".to_string()],
                speed_down: vec!["down".to_string()],
                quit: vec!["escape".to_string()],
            },
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path()?;

        if !config_path.exists() {
            let default_config = Config::default();
            let toml = toml::to_string_pretty(&default_config)?;
            fs::write(&config_path, toml)?;
            return Ok(default_config);
        }

        let contents = fs::read_to_string(&config_path)?;
        let config: Config = toml::from_str(&contents)?;
        Ok(config)
    }

    fn config_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not find config directory"))?;
        let app_dir = config_dir.join("speed-reader");

        if !app_dir.exists() {
            fs::create_dir_all(&app_dir)?;
        }

        Ok(app_dir.join("config.toml"))
    }
}
