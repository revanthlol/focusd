// Internal Modules (Local to CLI)
mod x11;
mod hyprland;

// External Modules (From Core)
use focusd_core::{db, config};

use clap::{Parser, Subcommand};
use colored::*;
use std::{thread, time::Duration};
use std::env;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};
use chrono::{NaiveDate, Duration as ChronoDuration};
use focusd_core::theme::{FocusdTheme, resolve_theme};
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{self, ClearType},
};

/// focusd - Privacy respecting screen time tracker
#[derive(Parser)]
#[command(name = "focusd")]
#[command(version = "0.4")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Daemon { #[arg(long)] verbose: bool },
    Today {
        #[arg(long)] json: bool,
        #[arg(long)] theme: Option<String>,
    },
    Week {
        #[arg(long)] json: bool,
        #[arg(long)] theme: Option<String>,
    },
    Export {
        #[arg(long, default_value = "json")]
        format: String,
    },
    Listen, 
    Month {
        #[arg(long)] json: bool,
        #[arg(long)] theme: Option<String>,
    },
    Range { 
        #[arg(long)] from: String,  // YYYY-MM-DD
        #[arg(long)] to: String,    // YYYY-MM-DD  
        #[arg(long)] json: bool,
        #[arg(long)] theme: Option<String>,
    },
    Top {
        #[arg(long, default_value = "7")]  days: i64,
        #[arg(long, default_value = "10")] limit: usize,
        #[arg(long)] json: bool,
        #[arg(long)] theme: Option<String>,
    },
    Stats,
    Doctor,
    Config {
        #[command(subcommand)] action: ConfigAction,
    },
    Watch {
        /// Refresh interval in seconds
        #[arg(long, default_value = "2")]
        interval: u64,
    },
}

#[derive(Subcommand)]
enum ConfigAction {
    Path,
    Edit,
    Init,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    
    if std::env::var("NO_COLOR").is_ok() {
        colored::control::set_override(false);
    }

    let config = config::Config::load();
    let db = db::Db::init()?;

    match cli.command {
        Commands::Daemon { verbose } => {
            run_daemon(&db, &config, verbose)?;
        }
        Commands::Listen => {
            // Debug Loop
            let is_hyprland = is_hyprland();
            let env_name = if is_hyprland { "Hyprland" } else { "X11/Other" };
            println!("Environment: {}", env_name.yellow());

            let x11_backend = if !is_hyprland { x11::X11Backend::new().ok() } else { None };

            loop {
                let window_opt = if is_hyprland {
                    hyprland::get_focused_window()
                } else {
                     x11_backend.as_ref().and_then(|b| b.get_focused_window())
                };

                match window_opt {
                    Some((app, title)) => println!("Focused: [{}] {}", app.blue(), title),
                    None => println!("Focused: None/Idle (or unknown)"),
                }

                if is_session_idle(None) {
                    println!("{}", ">> IDLE (OS reported user away) <<".red());
                }

                thread::sleep(Duration::from_secs(config.interval));
            }
        }
        Commands::Today { json, theme } => {
            if json {
                let data = db.get_usage_since(0)?;
                println!("{}", serde_json::to_string_pretty(&data)?);
                return Ok(());
            }
            let theme_opt = get_cli_theme(&config, theme.as_deref());
            print_report(&db, "Today", 0, theme_opt)?;
        }
        Commands::Week { json, theme } => {
            if json {
                let data = db.get_usage_since(7)?;
                println!("{}", serde_json::to_string_pretty(&data)?);
                return Ok(());
            }
            let theme_opt = get_cli_theme(&config, theme.as_deref());
            print_report(&db, "Last 7 Days", 7, theme_opt)?;
        }
        Commands::Export { format } => {
            let data = db.export_json()?;
            if format == "csv" {
                println!("date,app,seconds");
                for entry in data {
                    println!("{},{},{}", entry.date, entry.app, entry.seconds);
                }
            } else {
                let json = serde_json::to_string_pretty(&data)?;
                println!("{}", json);
            }
        }
        Commands::Month { json, theme } => {
            if json {
                let data = db.get_usage_since(30)?;
                println!("{}", serde_json::to_string_pretty(&data)?);
                return Ok(());
            }
            let theme_opt = get_cli_theme(&config, theme.as_deref());
            print_report(&db, "This Month", 30, theme_opt)?;
        }
        Commands::Range { from, to, json, theme } => {
            let start = NaiveDate::parse_from_str(&from, "%Y-%m-%d")
                .map_err(|_| anyhow::anyhow!("Invalid start date: {}. Expected YYYY-MM-DD", from))?;
            let end = NaiveDate::parse_from_str(&to, "%Y-%m-%d")
                .map_err(|_| anyhow::anyhow!("Invalid end date: {}. Expected YYYY-MM-DD", to))?;
            
            let data = db.get_app_usage_range(start, end)?;
            if json {
                println!("{}", serde_json::to_string_pretty(&data)?);
                return Ok(());
            }
            let theme_opt = get_cli_theme(&config, theme.as_deref());
            display_usage_table(data, &format!("{} → {}", from, to), theme_opt);
        }
        Commands::Top { days, limit, json, theme } => {
            let mut data = db.get_usage_since(days)?;
            data.truncate(limit);
            if json {
                println!("{}", serde_json::to_string_pretty(&data)?);
                return Ok(());
            }
            let theme_opt = get_cli_theme(&config, theme.as_deref());
            display_usage_table(data, &format!("Top {} apps — last {} days", limit, days), theme_opt);
        }
        Commands::Stats => {
            let stats = db.get_global_stats()?;
            let (current_s, longest_s) = db.get_streak_info()?;
            
            println!("\n{} — All Time Stats\n", "focusd".bold());
            println!("  {:<18} {}", "Days tracked:".bold(), stats.total_days_tracked.to_string().cyan());
            
            let h = stats.total_seconds_all_time / 3600;
            let m = (stats.total_seconds_all_time % 3600) / 60;
            println!("  {:<18} {}h {}m", "Total time:".bold(), h.to_string().cyan(), m.to_string().cyan());
            
            println!("  {:<18} {}", "Distinct apps:".bold(), stats.total_distinct_apps.to_string().cyan());
            println!("  {:<18} {}", "First tracked:".bold(), stats.first_tracked_date.unwrap_or_else(|| "Never".to_string()).cyan());
            println!("  {:<18} {} days", "Current streak:".bold(), current_s.to_string().green());
            println!("  {:<18} {} days", "Longest streak:".bold(), longest_s.to_string().green());
            println!();
        }
        Commands::Doctor => {
            run_doctor()?;
        }
        Commands::Config { action } => {
            let config_path = config::Config::get_path();
            match action {
                ConfigAction::Path => println!("{}", config_path.display()),
                ConfigAction::Edit => {
                    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "xdg-open".to_string());
                    Command::new(editor).arg(config_path).status()?;
                }
                ConfigAction::Init => {
                    if config_path.exists() {
                        println!("Config already exists at {}", config_path.display());
                    } else {
                        let template = r#"# focusd configuration
interval = 1
idle_timeout = 300
theme = "dark"

[alias]
# "code" = "VS Code"
# "firefox" = "Firefox"

# New style (takes priority over [alias]):
# [apps.code]
# name = "VS Code"
# category = "Development"
"#;
                        std::fs::write(&config_path, template)?;
                        println!("Initialized default config at: {}", config_path.display().to_string().green());
                        println!("\n---\n{}---\n", template);
                    }
                }
            }
        }
        Commands::Watch { interval } => {
            run_watch(&db, interval)?;
        }
    }
    Ok(())
}

fn is_hyprland() -> bool {
    env::var("HYPRLAND_INSTANCE_SIGNATURE").is_ok()
}

fn run_daemon(db: &db::Db, config: &config::Config, verbose: bool) -> anyhow::Result<()> {
    let is_hyprland = is_hyprland();
    
    let db_path = dirs::data_local_dir().expect("Data dir missing").join("focusd/focusd.db");
    let config_path = config::Config::get_path();

    println!("{}","focusd daemon".bold().cyan());
    println!("{}", "─────────────────────────────".dimmed());
    println!("  {:<12} : {}", "Backend".dimmed(), if is_hyprland { "Hyprland" } else { "X11" });
    println!("  {:<12} : {}", "DB".dimmed(), db_path.display());
    println!("  {:<12} : {}", "Config".dimmed(), config_path.display());
    println!("  {:<12} : {}s", "Interval".dimmed(), config.interval);
    println!("  {:<12} : {}s", "Idle after".dimmed(), config.idle_timeout);
    println!("  {}", "Polling...".green());

    let x11_backend = if !is_hyprland {
        match x11::X11Backend::new() {
            Ok(b) => Some(b),
            Err(e) => {
                eprintln!("Warning: Failed to init X11: {}", e);
                None
            }
        }
    } else {
        None
    };

    loop {
        thread::sleep(Duration::from_secs(config.interval));

        if is_session_idle(Some(config.idle_timeout)) {
            continue;
        }

        let window_opt = if is_hyprland {
            hyprland::get_focused_window()
        } else {
            match &x11_backend {
                Some(b) => b.get_focused_window(),
                None => None,
            }
        };

        if let Some((app_id, title)) = window_opt {
            // Skip logging if app_id is completely empty/whitespace (fixes blank line bug)
            if app_id.trim().is_empty() {
                continue;
            }

            if let Err(e) = db.log_usage(&app_id, &title, config.interval) {
                eprintln!("Error writing to DB: {}", e);
            } else if verbose {
                eprintln!("[tick] {} — {}s", app_id, config.interval);
            }
        }
    }
}

/// Determine if session is idle, optionally checking a timeout in seconds.
fn is_session_idle(timeout_secs: Option<u64>) -> bool {
    let session_id = match env::var("XDG_SESSION_ID") {
        Ok(id) => id,
        Err(_) => return false,
    };

    let output = Command::new("loginctl")
        .arg("show-session")
        .arg(&session_id)
        .arg("-p")
        .arg("IdleHint")
        .arg("-p")
        .arg("IdleSinceHint")
        .arg("--value")
        .output();

    let output = match output {
        Ok(o) if o.status.success() => o,
        _ => return false,
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut lines = stdout.lines();

    // Line 1: IdleHint (yes/no)
    if lines.next().map(|s| s.trim()) != Some("yes") {
        return false;
    }

    // Line 2: IdleSinceHint (timestamp in micros since epoch, or 0)
    let idle_since_micros_str = match lines.next() {
        Some(s) => s.trim(),
        None => return true, // Hint is "yes", just don't know for how long
    };

    let idle_since_micros: u128 = match idle_since_micros_str.parse() {
        Ok(n) if n > 0 => n,
        _ => return true, // Hint is "yes", just no exact timestamp
    };

    let now_micros = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_micros();

    let idle_duration_secs = (now_micros.saturating_sub(idle_since_micros) / 1_000_000) as u64;

    match timeout_secs {
        Some(threshold) => idle_duration_secs >= threshold,
        None => true, // Fallback to system's generic "yes"
    }
}

/// Generic report printer
fn print_report(db: &db::Db, title: &str, days_lookback: i64, theme_opt: Option<FocusdTheme>) -> anyhow::Result<()> {
    let theme = theme_opt.unwrap_or_else(FocusdTheme::default_dark);
    let raw_data = db.get_usage_since(days_lookback)?;
    let total_seconds: i64 = raw_data.iter().map(|(_, s)| s).sum();
    
    let t_h = total_seconds / 3600;
    let t_m = (total_seconds % 3600) / 60;
    
    let (acc_r, acc_g, acc_b) = hex_to_rgb(&theme.accent);
    println!("\n{} — {}h {}m", title.bold().truecolor(acc_r, acc_g, acc_b), t_h, t_m);

    // Comparison and Activity Status logic
    let today = chrono::Local::now().date_naive();
    if days_lookback == 0 {
        let is_hyprland = is_hyprland();
        let idle = is_session_idle(None);
        
        let active_name = if idle {
            "Idle".to_string()
        } else {
            let win = if is_hyprland {
                hyprland::get_focused_window()
            } else {
                x11::X11Backend::new().ok().and_then(|b| b.get_focused_window())
            };
            win.map(|(a, _)| a).unwrap_or_else(|| "Unknown".to_string())
        };

        let (act_r, act_g, act_b) = if idle { hex_to_rgb(&theme.destructive) } else { hex_to_rgb(&theme.success) };
        println!("  {:<10}  : {}", "Active now".dimmed(), active_name.truecolor(act_r, act_g, act_b));

        // Today comparison
        let yesterday = today - ChronoDuration::days(1);
        let y_total = db.get_day_total(yesterday).unwrap_or(0);
        print_comparison(total_seconds, y_total, "yesterday", &theme);

        // Sparkline for Today report
        print_sparkline(db, today, &theme)?;
    } else if days_lookback == 7 {
        // Week comparison
        let prev_start = today - ChronoDuration::days(15);
        let prev_end = today - ChronoDuration::days(8);
        let prev_data = db.get_app_usage_range(prev_start, prev_end)?;
        let prev_total: i64 = prev_data.iter().map(|(_, s)| s).sum();
        print_comparison(total_seconds, prev_total, "last week", &theme);
    }

    println!();
    display_usage_table(raw_data, "", Some(theme));    Ok(())
}

fn print_comparison(current: i64, previous: i64, label: &str, theme: &FocusdTheme) {
    let diff = current - previous;
    let abs_diff = diff.abs();
    let h = abs_diff / 3600;
    let m = (abs_diff % 3600) / 60;
    let time_str = format!("{}h {}m", h, m);

    let (s_r, s_g, s_b) = hex_to_rgb(&theme.success);
    let (d_r, d_g, d_b) = hex_to_rgb(&theme.destructive);
    let (w_r, w_g, w_b) = hex_to_rgb(&theme.warning);

    if diff > 0 {
        println!("  {} {} vs {}", "↑".truecolor(s_r, s_g, s_b), time_str.truecolor(s_r, s_g, s_b), label);
    } else if diff < 0 {
        println!("  {} {} vs {}", "↓".truecolor(d_r, d_g, d_b), time_str.truecolor(d_r, d_g, d_b), label);
    } else {
        println!("  {} {} vs {}", "•".truecolor(w_r, w_g, w_b), time_str.truecolor(w_r, w_g, w_b), label);
    }
}

fn print_sparkline(db: &db::Db, end_date: NaiveDate, theme: &FocusdTheme) -> anyhow::Result<()> {
    const SPARKS: [char; 8] = [' ', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
    let start_date = end_date - ChronoDuration::days(6);
    let totals_map = db.get_daily_totals(start_date, end_date)?;
    
    let mut day_totals = Vec::new();
    for i in 0..7 {
        let date = end_date - ChronoDuration::days(6 - i);
        let val = totals_map.get(&date.to_string()).cloned().unwrap_or(0);
        day_totals.push(val);
    }

    let max = *day_totals.iter().max().unwrap_or(&1).max(&1);
    let spark_str: String = day_totals.iter().map(|&t| {
        let idx = (t * 7 / max) as usize;
        SPARKS[idx]
    }).collect();

    let (r, g, b) = hex_to_rgb(&theme.primary);
    println!("  Last 7d  {}", spark_str.truecolor(r, g, b));
    Ok(())
}

fn make_bar(value: f64, max_val: f64, width: usize, theme: &FocusdTheme) -> String {
    let filled_exact = (value / max_val) * width as f64;
    let full_blocks = filled_exact.floor() as usize;
    let remainder = filled_exact - full_blocks as f64;
    let partial_index = (remainder * 8.0).round() as usize;

    let (f_r, f_g, f_b) = hex_to_rgb(&theme.primary);
    let (e_r, e_g, e_b) = hex_to_rgb(&theme.muted);

    let mut bar = String::new();
    let mut visible_width = 0;

    if full_blocks > 0 {
        bar.push_str(&"█".repeat(full_blocks).truecolor(f_r, f_g, f_b).to_string());
        visible_width += full_blocks;
    }
    
    if visible_width < width {
        let partial_chars = [' ', '▏', '▎', '▍', '▌', '▋', '▊', '▉', '█'];
        let partial_char = partial_chars[partial_index];
        if partial_index == 8 {
            bar.push_str(&"█".truecolor(f_r, f_g, f_b).to_string());
            visible_width += 1;
        } else if partial_index > 0 {
            bar.push_str(&partial_char.to_string().truecolor(f_r, f_g, f_b).to_string());
            visible_width += 1;
        }
    }
    
    if visible_width < width {
        let remaining = width - visible_width;
        bar.push_str(&"░".repeat(remaining).truecolor(e_r, e_g, e_b).to_string());
    }
    
    bar
}

fn format_duration(seconds: i64) -> String {
    let h = seconds / 3600;
    let m = (seconds % 3600) / 60;
    if h > 0 {
        format!("{}h {}m", h, m)
    } else {
        format!("{}m", m)
    }
}

/// Generic diagnostic utility
fn run_doctor() -> anyhow::Result<()> {
    println!("\n{} — System Diagnosis\n", "focusd".bold());

    // 1. Database
    let mut db_path = dirs::data_local_dir().expect("Data dir missing");
    db_path.push("focusd/focusd.db");
    if db_path.exists() {
        println!("  {} Database exists and is readable ({})", "✓".green(), db_path.display());
    } else {
        println!("  {} Database not found at {}", "✗".red(), db_path.display());
    }

    // 2. Config
    let config_path = config::Config::get_path();
    if config_path.exists() {
        println!("  {} Config file exists ({})", "✓".green(), config_path.display());
        match std::fs::read_to_string(&config_path) {
            Ok(content) => {
                if toml::from_str::<toml::Value>(&content).is_ok() {
                    println!("  {} Config parses without error", "✓".green());
                } else {
                    println!("  {} Config has invalid syntax", "✗".red());
                }
            }
            Err(_) => println!("  {} Config file is not readable", "✗".red()),
        }
    } else {
        println!("  {} Config file not found", "✗".red());
    }

    // 3. Environment/Backend
    let is_hyprland = is_hyprland();
    if is_hyprland {
        println!("  {} Hyprland detected (env var set)", "✓".green());
    } else {
        match x11::X11Backend::new() {
            Ok(_) => println!("  {} X11 detected and accessible", "✓".green()),
            Err(e) => println!("  {} Neither Hyprland nor X11 accessible: {}", "✗".red(), e),
        }
    }

    // 4. Dependencies
    let mut loginctl = Command::new("loginctl");
    match loginctl.arg("--version").output() {
        Ok(o) if o.status.success() => println!("  {} loginctl is available", "✓".green()),
        _ => println!("  {} loginctl not found or failed", "✗".red()),
    }

    if is_hyprland {
        let mut hyprctl = Command::new("hyprctl");
        match hyprctl.arg("version").output() {
            Ok(o) if o.status.success() => println!("  {} hyprctl is available", "✓".green()),
            _ => println!("  {} hyprctl not found or failed", "✗".red()),
        }
    }

    println!();
    Ok(())
}

fn get_cli_theme(config: &config::Config, override_theme: Option<&str>) -> Option<FocusdTheme> {
    let mut config_clone = config.clone();
    if let Some(t) = override_theme {
        if t == "none" {
            colored::control::set_override(false);
            return None;
        }
        config_clone.theme = t.to_string();
    }
    Some(resolve_theme(&config_clone))
}

fn hex_to_rgb(hex: &str) -> (u8, u8, u8) {
    let h = hex.trim_start_matches('#');
    let r = u8::from_str_radix(&h[0..2], 16).unwrap_or(255);
    let g = u8::from_str_radix(&h[2..4], 16).unwrap_or(255);
    let b = u8::from_str_radix(&h[4..6], 16).unwrap_or(255);
    (r, g, b)
}

/// Live-refreshing terminal dashboard.
fn run_watch(db: &db::Db, refresh_secs: u64) -> anyhow::Result<()> {
    let mut stdout = std::io::stdout();

    // Enter alternate screen + raw mode
    terminal::enable_raw_mode()?;
    execute!(stdout, terminal::EnterAlternateScreen, cursor::Hide)?;

    // Ensure terminal is always restored, even on panic
    let result = watch_loop(db, refresh_secs);

    // Cleanup
    let _ = terminal::disable_raw_mode();
    let mut stdout = std::io::stdout();
    let _ = execute!(stdout, terminal::LeaveAlternateScreen, cursor::Show);

    result
}

fn watch_loop(db: &db::Db, refresh_secs: u64) -> anyhow::Result<()> {
    let is_hyprland = is_hyprland();
    let x11_backend = if !is_hyprland { x11::X11Backend::new().ok() } else { None };
    let env_name = if is_hyprland { "Hyprland" } else { "X11" };

    let mut stdout = std::io::stdout();
    let poll_interval = Duration::from_millis(200);
    let mut elapsed = Duration::ZERO;

    loop {
        // Check for keypress
        if event::poll(poll_interval)? {
            if let Event::Key(key) = event::read()? {
                let quit = key.code == KeyCode::Char('q')
                    || key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL);
                if quit {
                    return Ok(());
                }
            }
        }

        elapsed += poll_interval;
        if elapsed < Duration::from_secs(refresh_secs) {
            continue;
        }
        elapsed = Duration::ZERO;

        // ── Gather data ──────────────────────────────────────────────────────
        let window_opt = if is_hyprland {
            hyprland::get_focused_window()
        } else {
            x11_backend.as_ref().and_then(|b| b.get_focused_window())
        };

        let idle = is_session_idle(None);

        let mut today_data = db.get_usage_since(0).unwrap_or_default();
        today_data.truncate(8);
        let total_today: i64 = today_data.iter().map(|(_, s)| s).sum();
        let max_val = today_data.iter().map(|(_, s)| *s).max().unwrap_or(1);

        let (current_streak, longest_streak) = db.get_streak_info().unwrap_or((0, 0));

        // ── Render ───────────────────────────────────────────────────────────
        execute!(
            stdout,
            cursor::MoveTo(0, 0),
            terminal::Clear(ClearType::All)
        )?;

        // Header
        println!(
            "{}  {}",
            "focusd watch".bold().cyan(),
            "[q to quit]".dimmed()
        );
        println!();

        // Environment + active window
        println!("  {:<12} {}", "Environment :".dimmed(), env_name.yellow());
        match &window_opt {
            Some((app, title)) => {
                let label = format!("{} — {}", app, title);
                println!("  {:<12} {}", "Active now  :".dimmed(), label.truncate_pad(60).green());
            }
            None => println!("  {:<12} {}", "Active now  :".dimmed(), "(none)".dimmed()),
        }
        println!(
            "  {:<12} {}",
            "Idle        :".dimmed(),
            if idle { "Yes".red() } else { "No".green() }
        );
        println!();

        // Today section
        let total_h = total_today / 3600;
        let total_m = (total_today % 3600) / 60;
        println!(
            "  {}",
            format!("── Today ─── {}h {}m total", total_h, total_m)
                .bold()
        );
        println!();

        if today_data.is_empty() {
            println!("  {}", "No data yet today.".dimmed());
        } else {
            for (display_name, seconds) in &today_data {
                if display_name.trim().is_empty() { continue; }

                let h = seconds / 3600;
                let m = (seconds % 3600) / 60;
                let pct = if total_today > 0 { seconds * 100 / total_today } else { 0 };

                let bar_width: usize = 20;
                let filled_len = (*seconds as f64 / max_val as f64 * bar_width as f64) as usize;
                let empty_len = bar_width.saturating_sub(filled_len);

                let bar_filled: String = std::iter::repeat('█').take(filled_len).collect();
                let bar_empty: String  = std::iter::repeat('░').take(empty_len).collect();

                println!(
                    "  {:<15} {}{}  {}h {:02}m  {}%",
                    display_name.truncate_pad(15),
                    bar_filled.cyan(),
                    bar_empty.dimmed(),
                    h, m,
                    pct
                );
            }
        }
        println!();

        // Streak section
        println!("  {}", "── Streak ──────────────────────────────".bold());
        println!();
        println!(
            "  Current: {}   Longest: {}",
            format!("{} days", current_streak).green(),
            format!("{} days", longest_streak).yellow()
        );
        println!();
    }
}

/// Helper to render the standard focusd table style
fn display_usage_table(data: Vec<(String, i64)>, title: &str, theme_opt: Option<FocusdTheme>) {
    let theme = theme_opt.unwrap_or_else(FocusdTheme::default_dark);
    if !title.is_empty() {
        let total_seconds: i64 = data.iter().map(|(_, s)| s).sum();
        let t_h = total_seconds / 3600;
        let t_m = (total_seconds % 3600) / 60;
        let (r, g, b) = hex_to_rgb(&theme.accent);
        println!("\n{} — {}h {}m\n", title.bold().truecolor(r, g, b), t_h, t_m);
    }

    if data.is_empty() {
        println!("No data found.");
        return;
    }

    let total_seconds: i64 = data.iter().map(|(_, s)| s).sum::<i64>().max(1);
    let max_val = data.iter().map(|(_, s)| *s).max().unwrap_or(1);

    let (mf_r, mf_g, mf_b) = hex_to_rgb(&theme.muted_foreground);

    for (display_name, seconds) in data {
        if display_name.trim().is_empty() { continue; }

        let time_str = format_duration(seconds);
        let bar = make_bar(seconds as f64, max_val as f64, 24, &theme);
        let pct = (seconds * 100 / total_seconds) as u64;

        println!(
            "{:<16} {}  {:>8}  {:>3}%", 
            display_name.truncate_pad(16), 
            bar, 
            time_str.truecolor(mf_r, mf_g, mf_b),
            pct.to_string().truecolor(mf_r, mf_g, mf_b)
        );
    }
    println!("");
}

trait StringExt {
    fn truncate_pad(&self, len: usize) -> String;
}

impl StringExt for String {
    fn truncate_pad(&self, len: usize) -> String {
        // Updated formatting to be stricter
        if self.len() > len {
            let mut s = self.clone();
            s.truncate(len - 1);
            format!("{}…", s)
        } else {
            format!("{:<width$}", self, width = len)
        }
    }
}
