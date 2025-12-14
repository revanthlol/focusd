use std::process::Command;
use serde::Deserialize;

#[derive(Deserialize)]
struct HyprWindow {
    class: String,
    title: String,
}

pub fn get_focused_window() -> Option<(String, String)> {
    // 1. Run 'hyprctl activewindow -j'
    let output = Command::new("hyprctl")
        .arg("activewindow")
        .arg("-j")
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    // 2. Parse JSON
    let output_str = String::from_utf8_lossy(&output.stdout);
    
    // Check if Hyprland returned "{}" (empty object implies no focus)
    if output_str.trim() == "{}" {
        return None;
    }

    match serde_json::from_str::<HyprWindow>(&output_str) {
        Ok(window) => {
            if window.class.is_empty() {
                return None;
            }
            // Hyprland 'class' is the stable App ID.
            Some((window.class, window.title))
        }
        Err(_) => None,
    }
}