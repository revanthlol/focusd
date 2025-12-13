mod db;
mod x11;
mod hyprland;
mod config;
mod idle;

use clap::{Parser, Subcommand};
use colored::*;
use std::{thread, time::Duration};
use std::env;

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
    Daemon,
    Today,
    Week,
    Export,
    Listen, 
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let config = config::Config::load();
    let db = db::Db::init()?;

    match cli.command {
        Commands::Daemon => {
            run_daemon(&db, &config)?;
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

                if idle::is_session_idle() {
                    println!("{}", ">> IDLE (OS reported user away) <<".red());
                }

                thread::sleep(Duration::from_secs(config.interval));
            }
        }
        Commands::Today => {
            // Pass Config to the print function now
            print_report(&db, &config, "Today", 0)?;
        }
        Commands::Week => {
            // Pass Config to the print function now
            print_report(&db, &config, "Last 7 Days", 7)?;
        }
        Commands::Export => {
            let data = db.export_json()?;
            let json = serde_json::to_string_pretty(&data)?;
            println!("{}", json);
        }
    }
    Ok(())
}

fn is_hyprland() -> bool {
    env::var("HYPRLAND_INSTANCE_SIGNATURE").is_ok()
}

fn run_daemon(db: &db::Db, config: &config::Config) -> anyhow::Result<()> {
    let is_hyprland = is_hyprland();
    
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

    println!("{}", "focusd daemon started...".green().bold());
    println!("Backend: {}", if is_hyprland { "Hyprland" } else { "X11" });

    loop {
        thread::sleep(Duration::from_secs(config.interval));

        if idle::is_session_idle() { continue; }

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
            }
        }
    }
}

/// Generic report printer
fn print_report(db: &db::Db, config: &config::Config, title: &str, days_lookback: i64) -> anyhow::Result<()> {
    let data = db.get_usage_since(days_lookback)?;
    let total_seconds: i64 = data.iter().map(|(_, s)| s).sum();
    
    let t_h = total_seconds / 3600;
    let t_m = (total_seconds % 3600) / 60;
    
    println!("\n{} — {}h {}m\n", title.bold(), t_h, t_m);

    if data.is_empty() {
        println!("No data found.");
        return Ok(());
    }

    let max_val = data.iter().map(|(_, s)| *s).max().unwrap_or(1);

    for (raw_name, seconds) in data {
        // Fix blank names in report immediately
        if raw_name.trim().is_empty() { continue; }

        // 1. LOOKUP ALIAS: Check if user defined a name in config.toml
        let display_name = config.alias.get(&raw_name).unwrap_or(&raw_name);

        let h = seconds / 3600;
        let m = (seconds % 3600) / 60;
        let s = seconds % 60;
        
        let bar_width: usize = 20; 
        let filled_len = (seconds as f64 / max_val as f64 * bar_width as f64) as usize;
        let empty_len = bar_width.saturating_sub(filled_len);
        
        let bar_filled: String = std::iter::repeat("█").take(filled_len).collect();
        let bar_empty: String = std::iter::repeat("░").take(empty_len).collect();

        println!(
            "{:<15} {}{} {}h {:02}m {:02}s", 
            display_name.truncate_pad(15), 
            bar_filled.cyan(), 
            bar_empty.dimmed(), 
            h, m, s
        );
    }
    println!("");
    Ok(())
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