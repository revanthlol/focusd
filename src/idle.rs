use std::process::Command;
use std::env;

/// Checks if the session is reported as "Idle" by systemd-logind
/// This works nicely on Hyprland/Wayland if the idle manager is running.
pub fn is_session_idle() -> bool {
    let session_id = match env::var("XDG_SESSION_ID") {
        Ok(id) => id,
        Err(_) => return false, // If we can't find session, assume active
    };

    // Command: loginctl show-session $ID -p IdleHint
    let output = Command::new("loginctl")
        .arg("show-session")
        .arg(&session_id)
        .arg("-p")
        .arg("IdleHint")
        .output();

    match output {
        Ok(o) if o.status.success() => {
            let stdout = String::from_utf8_lossy(&o.stdout);
            // Output format: "IdleHint=yes" or "IdleHint=no"
            stdout.trim() == "IdleHint=yes"
        },
        _ => false, // Default to not idle on error
    }
}