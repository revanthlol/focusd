use serde::Deserialize;
use std::fs;
use std::path::PathBuf;
use std::collections::HashMap; // Import this

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    #[serde(default = "default_interval")]
    pub interval: u64,

    #[allow(dead_code)]
    #[serde(default = "default_idle_timeout")]
    pub idle_timeout: u64,

    // NEW: Map raw AppIDs to pretty names
    #[serde(default)]
    pub alias: HashMap<String, String>, 
}

fn default_interval() -> u64 { 1 }
fn default_idle_timeout() -> u64 { 300 }

impl Config {
    pub fn load() -> Self {
        let config_path = Self::get_path();
        
        if !config_path.exists() {
            return Config { 
                interval: default_interval(),
                idle_timeout: default_idle_timeout(),
                alias: HashMap::new(),
            };
        }

        let contents = fs::read_to_string(config_path).unwrap_or_default();
        match toml::from_str(&contents) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Warning: Failed to parse config.toml: {}", e);
                // Return default on error so app doesn't crash
                Config {
                    interval: default_interval(),
                    idle_timeout: default_idle_timeout(),
                    alias: HashMap::new(),
                }
            }
        }
    }

    fn get_path() -> PathBuf {
        let mut path = dirs::config_dir().expect("Could not determine config dir");
        path.push("focusd");
        if !path.exists() {
            let _ = fs::create_dir_all(&path);
        }
        path.push("config.toml");
        path
    }
}