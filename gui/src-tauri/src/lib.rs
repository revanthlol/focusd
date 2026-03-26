use focusd_core::db::Db;
use chrono::{Local, Duration, Datelike, NaiveDate};

#[derive(serde::Serialize)]
struct DashboardData {
    total_seconds: i64,
    daily_average: i64,
    comparison_seconds: i64,      // positive = more than prev period, negative = less
    comparison_label: String,     // "vs yesterday" / "vs last week" / "vs last month"
    apps: Vec<(String, i64)>,
    daily_chart: Vec<(String, i64)>,
    hourly_chart: Vec<(u32, i64)>,  // only populated for "today" view
    heatmap: Vec<(String, i64)>,    // only populated for "month" view
    current_app: Option<String>,
    is_idle: bool,
}

#[tauri::command]
fn get_dashboard(view: String, anchor: Option<String>) -> Result<DashboardData, String> {
    let db = Db::init().map_err(|e| e.to_string())?;
    
    let anchor_date = match anchor {
        Some(s) => NaiveDate::parse_from_str(&s, "%Y-%m-%d").unwrap_or_else(|_| Local::now().date_naive()),
        None => Local::now().date_naive(),
    };
    let today = Local::now().date_naive();

    let mut data = DashboardData {
        total_seconds: 0,
        daily_average: 0,
        comparison_seconds: 0,
        comparison_label: String::new(),
        apps: Vec::new(),
        daily_chart: Vec::new(),
        hourly_chart: Vec::new(),
        heatmap: Vec::new(),
        current_app: None,
        is_idle: false,
    };

    match view.as_str() {
        "today" => {
            let start = anchor_date;
            let end = anchor_date;
            data.apps = db.get_app_usage_range(start, end).map_err(|e| e.to_string())?;
            data.total_seconds = data.apps.iter().map(|(_, s)| *s).sum();
            data.daily_average = data.total_seconds;
            
            let yesterday = anchor_date.pred_opt().unwrap();
            let y_total = db.get_day_total(yesterday).map_err(|e| e.to_string())?;
            data.comparison_seconds = data.total_seconds - y_total;
            data.comparison_label = "vs yesterday".to_string();
            
            data.hourly_chart = db.get_hourly_totals(anchor_date).map_err(|e| e.to_string())?;
        }
        "week" => {
            let w = anchor_date.weekday();
            let start = anchor_date - Duration::days(w.num_days_from_monday() as i64);
            let mut end = start + Duration::days(6);
            if end > today { end = today; }
            
            data.apps = db.get_app_usage_range(start, end).map_err(|e| e.to_string())?;
            data.total_seconds = data.apps.iter().map(|(_, s)| *s).sum();
            
            let days = (end - start).num_days() + 1;
            data.daily_average = if days > 0 { data.total_seconds / days } else { 0 };
            
            let prev_start = start - Duration::days(7);
            let prev_end = end - Duration::days(7);
            let prev_apps = db.get_app_usage_range(prev_start, prev_end).map_err(|e| e.to_string())?;
            let prev_total: i64 = prev_apps.iter().map(|(_, s)| *s).sum();
            data.comparison_seconds = data.total_seconds - prev_total;
            data.comparison_label = "vs last week".to_string();
            
            let totals_map = db.get_daily_totals(start, start + Duration::days(6)).map_err(|e| e.to_string())?;
            for i in 0..7 {
                let d = start + Duration::days(i);
                let label = d.format("%a").to_string();
                let val = *totals_map.get(&d.to_string()).unwrap_or(&0);
                data.daily_chart.push((label, val));
            }
        }
        "month" => {
            let start = anchor_date.with_day(1).unwrap();
            let mut end = if anchor_date.month() == 12 {
                NaiveDate::from_ymd_opt(anchor_date.year() + 1, 1, 1).unwrap().pred_opt().unwrap()
            } else {
                NaiveDate::from_ymd_opt(anchor_date.year(), anchor_date.month() + 1, 1).unwrap().pred_opt().unwrap()
            };
            let month_end = end;
            if end > today { end = today; }
            
            data.apps = db.get_app_usage_range(start, end).map_err(|e| e.to_string())?;
            data.total_seconds = data.apps.iter().map(|(_, s)| *s).sum();
            
            let days = (end - start).num_days() + 1;
            data.daily_average = if days > 0 { data.total_seconds / days } else { 0 };
            
            let prev_anchor = start.pred_opt().unwrap();
            let prev_start = prev_anchor.with_day(1).unwrap();
            let prev_end = prev_anchor;
            let prev_apps = db.get_app_usage_range(prev_start, prev_end).map_err(|e| e.to_string())?;
            let prev_total: i64 = prev_apps.iter().map(|(_, s)| *s).sum();
            data.comparison_seconds = data.total_seconds - prev_total;
            data.comparison_label = "vs last month".to_string();
            
            let totals_map = db.get_daily_totals(start, month_end).map_err(|e| e.to_string())?;
            let mut d = start;
            while d <= month_end {
                let label = d.day().to_string();
                let val = *totals_map.get(&d.to_string()).unwrap_or(&0);
                data.daily_chart.push((label, val));
                d = d.succ_opt().unwrap();
            }
            
            data.heatmap = db.get_monthly_heatmap(anchor_date.year(), anchor_date.month()).map_err(|e| e.to_string())?;
        }
        _ => return Err("Invalid view".to_string()),
    }

    Ok(data)
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
        .invoke_handler(tauri::generate_handler![get_dashboard, get_theme])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}