use focusd_core::db::Db;
use chrono::{Local, Duration, Datelike};

#[derive(serde::Serialize)]
struct DashboardData {
    total_seconds: i64,
    apps: Vec<(String, i64)>, // Name, Seconds
    chart: Vec<(String, i64)>, // Date Label (Mon/Tue), Seconds
}

#[tauri::command]
fn get_data(view: String) -> Result<DashboardData, String> {
    let db = Db::init().map_err(|e| e.to_string())?;
    let today = Local::now().date_naive();

    // 1. Determine Range
    let (start, end) = if view == "week" {
        let w = today.weekday();
        let mon = today - Duration::days(w.num_days_from_monday() as i64);
        (mon, today)
    } else {
        (today, today)
    };

    // 2. Fetch Apps List (Summed over range)
    let apps = db.get_app_usage_range(start, end).map_err(|e| e.to_string())?;
    
    let total = apps.iter().map(|(_, s)| s).sum();

    // 3. Fetch Chart Data (Daily Totals for Week view)
    let mut chart = Vec::new();
    if view == "week" {
        let map = db.get_daily_totals(start, end).unwrap_or_default();
        let mut d = start;
        while d <= end {
            let val = *map.get(&d.to_string()).unwrap_or(&0);
            chart.push((d.format("%a").to_string(), val)); // "Mon", "Tue"
            d += Duration::days(1);
        }
    }

    Ok(DashboardData { total_seconds: total, apps, chart })
}

#[tauri::command]
fn get_theme() -> Result<std::collections::HashMap<String, String>, String> {
    let config = focusd_core::config::Config::load();
    let theme = focusd_core::theme::resolve_theme(&config);
    Ok(theme.to_hsl_map())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![get_data, get_theme])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}