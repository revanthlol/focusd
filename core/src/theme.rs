use std::collections::HashMap;
use std::fs;
use serde::Deserialize;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FocusdTheme {
    // All values are hex strings e.g. "#5eead4"
    pub primary: String,
    pub primary_foreground: String,
    pub secondary: String,
    pub secondary_foreground: String,
    pub background: String,
    pub foreground: String,
    pub card: String,
    pub card_foreground: String,
    pub muted: String,
    pub muted_foreground: String,
    pub accent: String,
    pub accent_foreground: String,
    pub border: String,
    pub destructive: String,
    pub ring: String,
    // Semantic extras for CLI bars
    pub success: String,
    pub warning: String,
}

impl FocusdTheme {
    pub fn default_dark() -> Self {
        Self {
            background: "#0a0a0f".to_string(),
            foreground: "#e4e4ef".to_string(),
            card: "#12121a".to_string(),
            card_foreground: "#e4e4ef".to_string(),
            primary: "#5eead4".to_string(),
            primary_foreground: "#003731".to_string(),
            secondary: "#1a1a26".to_string(),
            secondary_foreground: "#e4e4ef".to_string(),
            muted: "#1a1a26".to_string(),
            muted_foreground: "#6b6b80".to_string(),
            accent: "#a78bfa".to_string(),
            accent_foreground: "#0a0a0f".to_string(),
            border: "#2a2a3a".to_string(),
            destructive: "#f87171".to_string(),
            ring: "#5eead4".to_string(),
            success: "#34d399".to_string(),
            warning: "#fbbf24".to_string(),
        }
    }

    /// Returns a HashMap of CSS var name -> HSL string for every field.
    pub fn to_hsl_map(&self) -> HashMap<String, String> {
        let mut map = HashMap::new();
        map.insert("--background".to_string(), hex_to_hsl(&self.background));
        map.insert("--foreground".to_string(), hex_to_hsl(&self.foreground));
        map.insert("--card".to_string(), hex_to_hsl(&self.card));
        map.insert("--card-foreground".to_string(), hex_to_hsl(&self.card_foreground));
        map.insert("--primary".to_string(), hex_to_hsl(&self.primary));
        map.insert("--primary-foreground".to_string(), hex_to_hsl(&self.primary_foreground));
        map.insert("--secondary".to_string(), hex_to_hsl(&self.secondary));
        map.insert("--secondary-foreground".to_string(), hex_to_hsl(&self.secondary_foreground));
        map.insert("--muted".to_string(), hex_to_hsl(&self.muted));
        map.insert("--muted-foreground".to_string(), hex_to_hsl(&self.muted_foreground));
        map.insert("--accent".to_string(), hex_to_hsl(&self.accent));
        map.insert("--accent-foreground".to_string(), hex_to_hsl(&self.accent_foreground));
        map.insert("--border".to_string(), hex_to_hsl(&self.border));
        map.insert("--destructive".to_string(), hex_to_hsl(&self.destructive));
        map.insert("--ring".to_string(), hex_to_hsl(&self.ring));
        map.insert("--success".to_string(), hex_to_hsl(&self.success));
        map.insert("--warning".to_string(), hex_to_hsl(&self.warning));
        map
    }
}

#[derive(Deserialize)]
struct MatugenColorValue {
    default: String,
}

#[derive(Deserialize)]
struct MatugenOutput {
    colors: HashMap<String, MatugenColorValue>,
}

pub fn load_matugen_theme() -> Option<FocusdTheme> {
    let mut cache_dir = dirs::cache_dir()?;
    cache_dir.push("matugen/colors.json");
    
    if !cache_dir.exists() {
        return None;
    }

    let content = fs::read_to_string(cache_dir).ok()?;
    let matugen: MatugenOutput = serde_json::from_str(&content).ok()?;
    
    let get = |key: &str| matugen.colors.get(key).map(|v| v.default.clone());

    let mut theme = FocusdTheme::default_dark();
    
    if let Some(c) = get("primary")            { theme.primary = c.clone(); theme.ring = c; }
    if let Some(c) = get("on_primary")         { theme.primary_foreground = c; }
    if let Some(c) = get("surface")            { theme.background = c; }
    if let Some(c) = get("on_background")      { theme.foreground = c; }
    if let Some(c) = get("surface_variant")    { theme.card = c.clone(); theme.muted = c; }
    if let Some(c) = get("on_surface")         { theme.card_foreground = c; }
    if let Some(c) = get("primary_container")  { theme.secondary = c; }
    if let Some(c) = get("on_surface_variant") { theme.secondary_foreground = c.clone(); theme.muted_foreground = c; }
    if let Some(c) = get("tertiary")           { theme.accent = c; }
    if let Some(c) = get("on_tertiary")        { theme.accent_foreground = c; }
    if let Some(c) = get("outline")            { theme.border = c; }
    if let Some(c) = get("error")              { theme.destructive = c; }
    if let Some(c) = get("secondary")          { theme.success = c; }
    if let Some(c) = get("tertiary")           { theme.warning = c; }

    Some(theme)
}

pub fn resolve_theme(config: &crate::config::Config) -> FocusdTheme {
    let mut theme = FocusdTheme::default_dark();
    
    match config.theme.as_str() {
        "matugen" => {
            if let Some(mt) = load_matugen_theme() {
                theme = mt;
            }
        }
        "custom" => {
            let tc = &config.theme_colors;
            if let Some(c) = &tc.primary { theme.primary = c.clone(); }
            if let Some(c) = &tc.primary_foreground { theme.primary_foreground = c.clone(); }
            if let Some(c) = &tc.secondary { theme.secondary = c.clone(); }
            if let Some(c) = &tc.secondary_foreground { theme.secondary_foreground = c.clone(); }
            if let Some(c) = &tc.background { theme.background = c.clone(); }
            if let Some(c) = &tc.foreground { theme.foreground = c.clone(); }
            if let Some(c) = &tc.card { theme.card = c.clone(); }
            if let Some(c) = &tc.card_foreground { theme.card_foreground = c.clone(); }
            if let Some(c) = &tc.muted { theme.muted = c.clone(); }
            if let Some(c) = &tc.muted_foreground { theme.muted_foreground = c.clone(); }
            if let Some(c) = &tc.accent { theme.accent = c.clone(); }
            if let Some(c) = &tc.accent_foreground { theme.accent_foreground = c.clone(); }
            if let Some(c) = &tc.border { theme.border = c.clone(); }
            if let Some(c) = &tc.destructive { theme.destructive = c.clone(); }
            if let Some(c) = &tc.ring { theme.ring = c.clone(); }
            if let Some(c) = &tc.success { theme.success = c.clone(); }
            if let Some(c) = &tc.warning { theme.warning = c.clone(); }
        }
        _ => {} 
    }
    
    theme
}


/// Convert "#rrggbb" to "H S% L%" string (shadcn CSS var format)
/// e.g. "#5eead4" -> "174 71% 64%"
pub fn hex_to_hsl(hex: &str) -> String {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 {
        return "0 0% 0%".to_string();
    }

    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0) as f64 / 255.0;
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0) as f64 / 255.0;
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0) as f64 / 255.0;

    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let l = (max + min) / 2.0;

    let (h, s);

    if (max - min).abs() < f64::EPSILON {
        s = 0.0;
        h = 0.0;
    } else {
        let d = max - min;
        s = if l > 0.5 { d / (2.0 - max - min) } else { d / (max + min) };
        
        let h_raw = if (max - r).abs() < f64::EPSILON {
            (g - b) / d + if g < b { 6.0 } else { 0.0 }
        } else if (max - g).abs() < f64::EPSILON {
            (b - r) / d + 2.0
        } else {
            (r - g) / d + 4.0
        };
        h = h_raw / 6.0 * 360.0;
    }

    format!("{:.0} {:.0}% {:.0}%", h, s * 100.0, l * 100.0)
}
