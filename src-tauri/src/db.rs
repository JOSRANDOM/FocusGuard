use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GlobalSchedule {
    pub id: i64,
    /// JSON array de días: [0,1,2] = Lun,Mar,Mié
    pub days: Vec<i32>,
    pub start_time: String,
    pub end_time: String,
    /// IDs de plataformas a bloquear. Vacío = todas las activas.
    pub platforms: Vec<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NewGlobalSchedule {
    pub days: Vec<i32>,
    pub start_time: String,
    pub end_time: String,
    pub platforms: Vec<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Platform {
    pub id: i64,
    pub name: String,
    pub domains: Vec<String>,
    pub enabled: bool,
    pub currently_blocked: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Schedule {
    pub id: i64,
    pub platform_id: i64,
    /// 0=Lun..6=Dom, 7=Todos los días
    pub day_of_week: i32,
    pub start_time: String,
    pub end_time: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NewSchedule {
    pub platform_id: i64,
    pub day_of_week: i32,
    pub start_time: String,
    pub end_time: String,
}

pub fn init_db() -> Result<Connection> {
    let db_path = get_db_path();
    let conn = Connection::open(&db_path)?;

    conn.execute_batch(
        "PRAGMA foreign_keys = ON;

        CREATE TABLE IF NOT EXISTS platforms (
            id      INTEGER PRIMARY KEY AUTOINCREMENT,
            name    TEXT    NOT NULL,
            domains TEXT    NOT NULL,
            enabled INTEGER NOT NULL DEFAULT 0
        );

        CREATE TABLE IF NOT EXISTS schedules (
            id           INTEGER PRIMARY KEY AUTOINCREMENT,
            platform_id  INTEGER NOT NULL,
            day_of_week  INTEGER NOT NULL DEFAULT 7,
            start_time   TEXT    NOT NULL,
            end_time     TEXT    NOT NULL,
            FOREIGN KEY (platform_id) REFERENCES platforms(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS global_schedules (
            id         INTEGER PRIMARY KEY AUTOINCREMENT,
            days       TEXT    NOT NULL,
            start_time TEXT    NOT NULL,
            end_time   TEXT    NOT NULL,
            platforms  TEXT    NOT NULL DEFAULT '[]'
        );",
    )?;

    // Migración: agrega columna platforms si no existe
    let _ = conn.execute(
        "ALTER TABLE global_schedules ADD COLUMN platforms TEXT NOT NULL DEFAULT '[]'",
        [],
    );

    let count: i64 =
        conn.query_row("SELECT COUNT(*) FROM platforms", [], |r| r.get(0))?;

    if count == 0 {
        seed_defaults(&conn)?;
    }

    Ok(conn)
}

fn seed_defaults(conn: &Connection) -> Result<()> {
    let defaults: Vec<(&str, &str)> = vec![
        (
            "TikTok",
            r#"["tiktok.com","www.tiktok.com","vm.tiktok.com","m.tiktok.com"]"#,
        ),
        (
            "Facebook",
            r#"["facebook.com","www.facebook.com","m.facebook.com","web.facebook.com"]"#,
        ),
        (
            "Instagram",
            r#"["instagram.com","www.instagram.com"]"#,
        ),
        (
            "YouTube",
            r#"["youtube.com","www.youtube.com","m.youtube.com","youtu.be"]"#,
        ),
    ];

    for (name, domains) in defaults {
        conn.execute(
            "INSERT INTO platforms (name, domains, enabled) VALUES (?1, ?2, 0)",
            params![name, domains],
        )?;
    }

    Ok(())
}

fn get_db_path() -> std::path::PathBuf {
    #[cfg(target_os = "macos")]
    {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let path = std::path::PathBuf::from(home)
            .join("Library/Application Support/FocusGuard");
        std::fs::create_dir_all(&path).ok();
        path.join("focusguard.db")
    }
    #[cfg(target_os = "windows")]
    {
        let appdata = std::env::var("APPDATA").unwrap_or_else(|_| ".".to_string());
        let path = std::path::PathBuf::from(appdata).join("FocusGuard");
        std::fs::create_dir_all(&path).ok();
        path.join("focusguard.db")
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        std::path::PathBuf::from("focusguard.db")
    }
}

pub fn get_platforms(conn: &Connection) -> Result<Vec<Platform>> {
    let mut stmt =
        conn.prepare("SELECT id, name, domains, enabled FROM platforms ORDER BY id")?;
    let platforms = stmt
        .query_map([], |row| {
            let domains_str: String = row.get(2)?;
            let domains: Vec<String> =
                serde_json::from_str(&domains_str).unwrap_or_default();
            Ok(Platform {
                id: row.get(0)?,
                name: row.get(1)?,
                domains,
                enabled: row.get::<_, i32>(3)? != 0,
                currently_blocked: false,
            })
        })?
        .collect::<Result<Vec<_>>>()?;
    Ok(platforms)
}

pub fn toggle_platform(conn: &Connection, id: i64, enabled: bool) -> Result<()> {
    conn.execute(
        "UPDATE platforms SET enabled = ?1 WHERE id = ?2",
        params![enabled as i32, id],
    )?;
    Ok(())
}

pub fn get_schedules(conn: &Connection, platform_id: i64) -> Result<Vec<Schedule>> {
    let mut stmt = conn.prepare(
        "SELECT id, platform_id, day_of_week, start_time, end_time
         FROM schedules WHERE platform_id = ?1
         ORDER BY day_of_week, start_time",
    )?;
    let schedules = stmt
        .query_map([platform_id], |row| {
            Ok(Schedule {
                id: row.get(0)?,
                platform_id: row.get(1)?,
                day_of_week: row.get(2)?,
                start_time: row.get(3)?,
                end_time: row.get(4)?,
            })
        })?
        .collect::<Result<Vec<_>>>()?;
    Ok(schedules)
}

pub fn add_schedule(conn: &Connection, s: &NewSchedule) -> Result<i64> {
    conn.execute(
        "INSERT INTO schedules (platform_id, day_of_week, start_time, end_time)
         VALUES (?1, ?2, ?3, ?4)",
        params![s.platform_id, s.day_of_week, s.start_time, s.end_time],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn delete_schedule(conn: &Connection, id: i64) -> Result<()> {
    conn.execute("DELETE FROM schedules WHERE id = ?1", params![id])?;
    Ok(())
}

pub fn get_global_schedules(conn: &Connection) -> Result<Vec<GlobalSchedule>> {
    let mut stmt = conn.prepare(
        "SELECT id, days, start_time, end_time, platforms FROM global_schedules ORDER BY start_time",
    )?;
    let schedules = stmt
        .query_map([], |row| {
            let days_str: String = row.get(1)?;
            let platforms_str: String = row.get(4)?;
            Ok(GlobalSchedule {
                id: row.get(0)?,
                days: serde_json::from_str(&days_str).unwrap_or_default(),
                start_time: row.get(2)?,
                end_time: row.get(3)?,
                platforms: serde_json::from_str(&platforms_str).unwrap_or_default(),
            })
        })?
        .collect::<Result<Vec<_>>>()?;
    Ok(schedules)
}

pub fn add_global_schedule(conn: &Connection, s: &NewGlobalSchedule) -> Result<i64> {
    let days_json = serde_json::to_string(&s.days).unwrap_or_default();
    let platforms_json = serde_json::to_string(&s.platforms).unwrap_or_default();
    conn.execute(
        "INSERT INTO global_schedules (days, start_time, end_time, platforms) VALUES (?1, ?2, ?3, ?4)",
        params![days_json, s.start_time, s.end_time, platforms_json],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn delete_global_schedule(conn: &Connection, id: i64) -> Result<()> {
    conn.execute("DELETE FROM global_schedules WHERE id = ?1", params![id])?;
    Ok(())
}
