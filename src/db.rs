use rusqlite::{params, Connection, Result};
use chrono::{Duration, Local}; 
use std::fs;

pub struct Db {
    conn: Connection,
}

#[derive(serde::Serialize)]
pub struct ExportEntry {
    pub date: String,
    pub app: String,
    pub seconds: i64,
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
        Ok(db)
    }

    fn create_tables(&self) -> Result<()> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS apps (
                id INTEGER PRIMARY KEY,
                app_id TEXT UNIQUE NOT NULL,
                display_name TEXT
            )", []
        )?;

        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS usage_daily (
                id INTEGER PRIMARY KEY,
                app_ref_id INTEGER NOT NULL,
                date TEXT NOT NULL,
                seconds_focused INTEGER DEFAULT 0,
                FOREIGN KEY(app_ref_id) REFERENCES apps(id),
                UNIQUE(app_ref_id, date)
            )", []
        )?;
        Ok(())
    }

    pub fn log_usage(&self, wm_class: &str, _window_title: &str, seconds: u64) -> anyhow::Result<()> {
        let today = Local::now().date_naive().to_string();

        self.conn.execute(
            "INSERT OR IGNORE INTO apps (app_id, display_name) VALUES (?1, ?2)",
            params![wm_class, wm_class], 
        )?;

        let app_ref_id: i64 = self.conn.query_row(
            "SELECT id FROM apps WHERE app_id = ?1",
            params![wm_class],
            |row| row.get(0),
        )?;

        self.conn.execute(
            "INSERT INTO usage_daily (app_ref_id, date, seconds_focused) 
             VALUES (?1, ?2, ?3)
             ON CONFLICT(app_ref_id, date) DO UPDATE SET seconds_focused = seconds_focused + ?3",
            params![app_ref_id, today, seconds],
        )?;

        Ok(())
    }

    // === NEW: Generic Query for Today or Date Range ===
    pub fn get_usage_since(&self, days_ago: i64) -> anyhow::Result<Vec<(String, i64)>> {
        let cutoff_date = (Local::now() - Duration::days(days_ago)).date_naive().to_string();
        
        let mut stmt = self.conn.prepare(
            "SELECT a.display_name, SUM(u.seconds_focused) as total
             FROM usage_daily u
             JOIN apps a ON u.app_ref_id = a.id
             WHERE u.date >= ?1
             GROUP BY a.display_name
             ORDER BY total DESC"
        )?;

        let rows = stmt.query_map(params![cutoff_date], |row| {
            Ok((row.get(0)?, row.get(1)?))
        })?;

        let mut result = Vec::new();
        for r in rows {
            result.push(r?);
        }
        Ok(result)
    }

    // === NEW: Export Data ===
    pub fn export_json(&self) -> anyhow::Result<Vec<ExportEntry>> {
        let mut stmt = self.conn.prepare(
            "SELECT u.date, a.display_name, u.seconds_focused
             FROM usage_daily u
             JOIN apps a ON u.app_ref_id = a.id
             ORDER BY u.date DESC, u.seconds_focused DESC"
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
}