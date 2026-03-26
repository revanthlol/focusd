use serde::Deserialize;
use std::fs;
use std::path::PathBuf;
use std::collections::HashMap;

#[derive(Deserialize, Debug, Clone, Default)]
pub struct AppConfig {
    pub name: Option<String>,
    pub category: Option<String>,
}

#[derive(Deserialize, Debug, Clone, Default)]
pub struct ThemeColors {
    pub primary: Option<String>,
    pub primary_foreground: Option<String>,
    pub secondary: Option<String>,
    pub secondary_foreground: Option<String>,
    pub background: Option<String>,
    pub foreground: Option<String>,
    pub card: Option<String>,
    pub card_foreground: Option<String>,
    pub muted: Option<String>,
    pub muted_foreground: Option<String>,
    pub accent: Option<String>,
    pub accent_foreground: Option<String>,
    pub border: Option<String>,
    pub destructive: Option<String>,
    pub ring: Option<String>,
    pub success: Option<String>,
    pub warning: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    #[serde(default = "default_interval")]
    pub interval: u64,

    #[serde(default = "default_idle_timeout")]
    pub idle_timeout: u64,

    #[serde(default = "default_theme")]
    pub theme: String,

    #[serde(default)]
    pub alias: HashMap<String, String>,

    #[serde(default)]
    pub apps: HashMap<String, AppConfig>,

    #[serde(default)]
    pub theme_colors: ThemeColors,
}

fn default_interval() -> u64 { 1 }
fn default_idle_timeout() -> u64 { 300 }
fn default_theme() -> String { "dark".to_string() }

impl Config {
    pub fn load() -> Self {
        let config_path = Self::get_path();
        
        if !config_path.exists() {
            return Config {
                interval: default_interval(),
                idle_timeout: default_idle_timeout(),
                theme: default_theme(),
                alias: HashMap::new(),
                apps: HashMap::new(),
                theme_colors: ThemeColors::default(),
            };
        }

        let contents = fs::read_to_string(config_path).unwrap_or_default();
        match toml::from_str(&contents) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Warning: Failed to parse config.toml: {}", e);
                Config {
                    interval: default_interval(),
                    idle_timeout: default_idle_timeout(),
                    theme: default_theme(),
                    alias: HashMap::new(),
                    apps: HashMap::new(),
                    theme_colors: ThemeColors::default(),
                }
            }
        }
    }

    pub fn resolve_alias(&self, app_id: &str) -> Option<String> {
        // Check apps map first
        if let Some(app_cfg) = self.apps.get(app_id) {
            if let Some(name) = &app_cfg.name {
                return Some(name.clone());
            }
        }
        // Fall back to alias map
        self.alias.get(app_id).cloned()
    }

    pub fn resolve_category(&self, app_id: &str) -> Option<String> {
        self.apps.get(app_id).and_then(|app_cfg| app_cfg.category.clone())
    }

    pub fn get_path() -> PathBuf {
        let mut path = dirs::config_dir().expect("Could not determine config dir");
        path.push("focusd");
        if !path.exists() {
            let _ = fs::create_dir_all(&path);
        }
        path.push("config.toml");
        path
    }
}
