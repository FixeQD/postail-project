use rusqlite::params;
use crate::db::init_db;
use crate::error::DBError;
use std::collections::HashMap;

pub fn get_all_settings() -> Result<HashMap<String, String>, DBError> {
    let conn = init_db()?;
    let mut stmt = conn.prepare("SELECT key, value FROM settings")?;
    let rows = stmt.query_map([], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
    })?;

    let mut settings = HashMap::new();
    for row in rows {
        let (key, value) = row?;
        settings.insert(key, value);
    }
    Ok(settings)
}

pub fn get_setting(key: &str) -> Result<Option<String>, DBError> {
    let conn = init_db()?;
    let mut stmt = conn.prepare("SELECT value FROM settings WHERE key = ?")?;
    let mut rows = stmt.query(params![key])?;

    if let Some(row) = rows.next()? {
        Ok(Some(row.get(0)?))
    } else {
        Ok(None)
    }
}

pub fn set_setting(key: &str, value: &str) -> Result<(), DBError> {
    let conn = init_db()?;
    conn.execute(
        "INSERT OR REPLACE INTO settings (key, value) VALUES (?, ?)",
        params![key, value],
    )?;
    Ok(())
}
