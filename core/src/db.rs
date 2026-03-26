use rusqlite::{params, Connection, Result};
use chrono::{Duration, Local, NaiveDate, Timelike};
use std::fs;
use std::collections::HashMap;
use crate::config::Config;

pub struct Db {
    conn: Connection,
}

#[derive(serde::Serialize)]
pub struct ExportEntry {
    pub date: String,
    pub app: String,
    pub seconds: i64,
}

#[derive(serde::Serialize)]
pub struct GlobalStats {
    pub total_days_tracked: i64,
    pub total_seconds_all_time: i64,
    pub total_distinct_apps: i64,
    pub first_tracked_date: Option<String>,
}

impl Db {
    pub fn init() -> anyhow::Result<Self> {
        let mut db_path = dirs::data_local_dir().expect("Could not find data dir");
        db_path.push("focusd");
        if !db_path.exists() {
            let _ = fs::create_dir_all(&db_path);
        }
        db_path.push("focusd.db");

        let conn = Connection::open(db_path)?;
        conn.execute_batch("PRAGMA journal_mode = WAL; PRAGMA synchronous = NORMAL;")?;

        let db = Db { conn };
        db.create_tables()?;
        db.run_migrations()?;
        Ok(db)
    }

    fn create_tables(&self) -> Result<()> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS apps (
                id INTEGER PRIMARY KEY,
                app_id TEXT UNIQUE NOT NULL,
                display_name TEXT
            )",
            [],
        )?;

        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS usage_daily (
                id INTEGER PRIMARY KEY,
                app_ref_id INTEGER NOT NULL,
                date TEXT NOT NULL,
                seconds_focused INTEGER DEFAULT 0,
                FOREIGN KEY(app_ref_id) REFERENCES apps(id),
                UNIQUE(app_ref_id, date)
            )",
            [],
        )?;

        // Also created here for fresh installs
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS usage_hourly (
                id INTEGER PRIMARY KEY,
                app_ref_id INTEGER NOT NULL,
                date TEXT NOT NULL,
                hour INTEGER NOT NULL,
                seconds_focused INTEGER DEFAULT 0,
                FOREIGN KEY(app_ref_id) REFERENCES apps(id),
                UNIQUE(app_ref_id, date, hour)
            )",
            [],
        )?;

        Ok(())
    }

    /// Run schema migrations keyed by `PRAGMA user_version`.
    /// Each migration runs inside a transaction and bumps the version on success.
    fn run_migrations(&self) -> anyhow::Result<()> {
        let current_version: i64 =
            self.conn.query_row("PRAGMA user_version", [], |row| row.get(0))?;

        eprintln!("[focusd] DB schema version: {}", current_version);

        if current_version < 1 {
            eprintln!("[focusd] Running migration 1: create usage_hourly …");
            self.conn.execute_batch(
                "BEGIN;
                 CREATE TABLE IF NOT EXISTS usage_hourly (
                     id INTEGER PRIMARY KEY,
                     app_ref_id INTEGER NOT NULL,
                     date TEXT NOT NULL,
                     hour INTEGER NOT NULL,
                     seconds_focused INTEGER DEFAULT 0,
                     FOREIGN KEY(app_ref_id) REFERENCES apps(id),
                     UNIQUE(app_ref_id, date, hour)
                 );
                 PRAGMA user_version = 1;
                 COMMIT;",
            )?;
            eprintln!("[focusd] Migration 1 complete.");
        }

        // Future migrations follow the same pattern:
        // if current_version < 2 { … PRAGMA user_version = 2; … }

        Ok(())
    }

    pub fn log_usage(&self, wm_class: &str, _window_title: &str, seconds: u64) -> anyhow::Result<()> {
        // Normalize: trim whitespace; skip empty entries
        let wm_class = wm_class.trim();
        if wm_class.is_empty() {
            return Ok(());
        }

        let now = Local::now();
        let today = now.date_naive().to_string();
        let hour = now.hour() as i64;

        // Wrap both upserts in a single transaction
        self.conn.execute_batch("BEGIN")?;

        let result = (|| -> anyhow::Result<()> {
            // Ensure app row exists
            self.conn.execute(
                "INSERT OR IGNORE INTO apps (app_id, display_name) VALUES (?1, ?2)",
                params![wm_class, wm_class],
            )?;

            let app_ref_id: i64 = self.conn.query_row(
                "SELECT id FROM apps WHERE app_id = ?1",
                params![wm_class],
                |row| row.get(0),
            )?;

            // Upsert daily total
            self.conn.execute(
                "INSERT INTO usage_daily (app_ref_id, date, seconds_focused)
                 VALUES (?1, ?2, ?3)
                 ON CONFLICT(app_ref_id, date) DO UPDATE SET seconds_focused = seconds_focused + ?3",
                params![app_ref_id, today, seconds],
            )?;

            // Upsert hourly total
            self.conn.execute(
                "INSERT INTO usage_hourly (app_ref_id, date, hour, seconds_focused)
                 VALUES (?1, ?2, ?3, ?4)
                 ON CONFLICT(app_ref_id, date, hour) DO UPDATE SET seconds_focused = seconds_focused + ?4",
                params![app_ref_id, today, hour, seconds],
            )?;

            Ok(())
        })();

        match result {
            Ok(()) => {
                self.conn.execute_batch("COMMIT")?;
                Ok(())
            }
            Err(e) => {
                let _ = self.conn.execute_batch("ROLLBACK");
                Err(e)
            }
        }
    }

    pub fn export_json(&self) -> anyhow::Result<Vec<ExportEntry>> {
        let mut stmt = self.conn.prepare(
            "SELECT u.date, a.display_name, u.seconds_focused
             FROM usage_daily u
             JOIN apps a ON u.app_ref_id = a.id
             ORDER BY u.date DESC, u.seconds_focused DESC",
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(ExportEntry {
                date: row.get(0)?,
                app: row.get(1)?,
                seconds: row.get(2)?,
            })
        })?;

        let mut result = Vec::new();
        for r in rows {
            result.push(r?);
        }
        Ok(result)
    }

    // === QUERY METHODS ===

    /// Total screen time per day for a date range (for charts).
    /// Returns: HashMap<"2024-03-01", total_seconds>
    pub fn get_daily_totals(&self, start: NaiveDate, end: NaiveDate) -> anyhow::Result<HashMap<String, i64>> {
        let mut stmt = self.conn.prepare(
            "SELECT u.date, SUM(u.seconds_focused)
             FROM usage_daily u
             WHERE u.date BETWEEN ?1 AND ?2
             GROUP BY u.date",
        )?;

        let rows = stmt.query_map(params![start.to_string(), end.to_string()], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
        })?;

        let mut map = HashMap::new();
        for r in rows {
            let (date_str, seconds) = r?;
            map.insert(date_str, seconds);
        }
        Ok(map)
    }

    /// Total time per app for a date range (for list view).
    pub fn get_app_usage_range(&self, start: NaiveDate, end: NaiveDate) -> anyhow::Result<Vec<(String, i64)>> {
        let mut stmt = self.conn.prepare(
            "SELECT a.app_id, SUM(u.seconds_focused) as total
             FROM usage_daily u
             JOIN apps a ON u.app_ref_id = a.id
             WHERE u.date BETWEEN ?1 AND ?2
             GROUP BY a.app_id
             ORDER BY total DESC",
        )?;

        let rows = stmt.query_map(params![start.to_string(), end.to_string()], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
        })?;

        let config = Config::load();
        let mut merged: HashMap<String, i64> = HashMap::new();

        for r in rows {
            let (app_id, seconds) = r?;
            let display_name = config.resolve_alias(&app_id).unwrap_or(app_id);
            *merged.entry(display_name).or_insert(0) += seconds;
        }

        let mut result: Vec<(String, i64)> = merged.into_iter().collect();
        result.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        Ok(result)
    }

    /// Legacy support for CLI — wraps get_app_usage_range.
    pub fn get_usage_since(&self, days_ago: i64) -> anyhow::Result<Vec<(String, i64)>> {
        let end = Local::now().date_naive();
        let start = end - chrono::Duration::days(days_ago);
        self.get_app_usage_range(start, end)
    }

    /// 1. Get total seconds for a single day across all apps.
    pub fn get_day_total(&self, date: NaiveDate) -> anyhow::Result<i64> {
        let total: Option<i64> = self.conn.query_row(
            "SELECT SUM(seconds_focused) FROM usage_daily WHERE date = ?1",
            params![date.to_string()],
            |row| row.get(0),
        )?;
        Ok(total.unwrap_or(0))
    }

    /// 2. Get hourly totals (hour, seconds) for a given day.
    pub fn get_hourly_totals(&self, date: NaiveDate) -> anyhow::Result<Vec<(u32, i64)>> {
        let mut stmt = self.conn.prepare(
            "SELECT hour, SUM(seconds_focused) 
             FROM usage_hourly 
             WHERE date = ?1 
             GROUP BY hour 
             ORDER BY hour",
        )?;

        let rows = stmt.query_map(params![date.to_string()], |row| {
            let h: i64 = row.get(0)?;
            Ok((h as u32, row.get(1)?))
        })?;

        let mut result = Vec::new();
        for r in rows {
            result.push(r?);
        }
        Ok(result)
    }

    /// 3. Get detailed app breakdown per hour for a given day.
    pub fn get_hourly_app_breakdown(&self, date: NaiveDate) -> anyhow::Result<Vec<(u32, String, i64)>> {
        let mut stmt = self.conn.prepare(
            "SELECT u.hour, a.display_name, u.seconds_focused
             FROM usage_hourly u
             JOIN apps a ON u.app_ref_id = a.id
             WHERE u.date = ?1
             ORDER BY u.hour ASC, u.seconds_focused DESC",
        )?;

        let rows = stmt.query_map(params![date.to_string()], |row| {
            let h: i64 = row.get(0)?;
            Ok((h as u32, row.get(1)?, row.get(2)?))
        })?;

        let mut result = Vec::new();
        for r in rows {
            result.push(r?);
        }
        Ok(result)
    }

    /// 4. Get total seconds for every day in a specific month.
    pub fn get_monthly_heatmap(&self, year: i32, month: u32) -> anyhow::Result<Vec<(String, i64)>> {
        let start_date = NaiveDate::from_ymd_opt(year, month, 1).ok_or_else(|| anyhow::anyhow!("Invalid date"))?;
        
        let end_date = if month == 12 {
            NaiveDate::from_ymd_opt(year + 1, 1, 1)
        } else {
            NaiveDate::from_ymd_opt(year, month + 1, 1)
        }.unwrap().pred_opt().unwrap();

        let mut stmt = self.conn.prepare(
            "SELECT date, SUM(seconds_focused)
             FROM usage_daily
             WHERE date BETWEEN ?1 AND ?2
             GROUP BY date
             ORDER BY date ASC",
        )?;

        let rows = stmt.query_map(params![start_date.to_string(), end_date.to_string()], |row| {
            Ok((row.get(0)?, row.get(1)?))
        })?;

        let mut result = Vec::new();
        for r in rows {
            result.push(r?);
        }
        Ok(result)
    }

    /// 5. Calculate current and longest consecutive day streaks.
    pub fn get_streak_info(&self) -> anyhow::Result<(i64, i64)> {
        let mut stmt = self.conn.prepare("SELECT DISTINCT date FROM usage_daily ORDER BY date ASC")?;
        let dates: Vec<NaiveDate> = stmt.query_map([], |row| {
            let s: String = row.get(0)?;
            Ok(NaiveDate::parse_from_str(&s, "%Y-%m-%d").unwrap())
        })?
        .collect::<Result<Vec<_>, _>>()?;

        if dates.is_empty() {
            return Ok((0, 0));
        }

        let mut longest = 0;
        let mut current_run = 1;
        
        for i in 1..dates.len() {
            if dates[i] == dates[i-1].succ_opt().unwrap() {
                current_run += 1;
            } else {
                if current_run > longest {
                    longest = current_run;
                }
                current_run = 1;
            }
        }
        if current_run > longest {
            longest = current_run;
        }

        // Current streak logic
        let today = Local::now().date_naive();
        let mut current_streak = 0;
        
        if let Some(last_date) = dates.last() {
            if *last_date == today {
                // Count backwards from end of the consecutive block containing today
                let mut run = 0;
                for date in dates.iter().rev() {
                    let expected = today - Duration::days(run as i64);
                    if *date == expected {
                        run += 1;
                    } else {
                        break;
                    }
                }
                current_streak = run;
            } else if *last_date == today.pred_opt().unwrap() {
                // Streak is still "current" if it includes yesterday
                let mut run = 0;
                let yesterday = today.pred_opt().unwrap();
                for date in dates.iter().rev() {
                    let expected = yesterday - Duration::days(run as i64);
                    if *date == expected {
                        run += 1;
                    } else {
                        break;
                    }
                }
                current_streak = run;
            }
        }

        Ok((current_streak as i64, longest as i64))
    }

    /// 6. Get overall statistics for the dashboard sidebar.
    pub fn get_global_stats(&self) -> anyhow::Result<GlobalStats> {
        let total_days: i64 = self.conn.query_row(
            "SELECT COUNT(DISTINCT date) FROM usage_daily",
            [],
            |row| row.get(0),
        )?;

        let total_seconds: Option<i64> = self.conn.query_row(
            "SELECT SUM(seconds_focused) FROM usage_daily",
            [],
            |row| row.get(0),
        )?;

        let total_apps: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM apps",
            [],
            |row| row.get(0),
        )?;

        let first_date: Option<String> = self.conn.query_row(
            "SELECT MIN(date) FROM usage_daily",
            [],
            |row| row.get(0),
        ).unwrap_or(None);

        Ok(GlobalStats {
            total_days_tracked: total_days,
            total_seconds_all_time: total_seconds.unwrap_or(0),
            total_distinct_apps: total_apps,
            first_tracked_date: first_date,
        })
    }
}
