use crate::error::DBError;
use crate::globals::get_db_pool;
use rusqlite::params;
use std::collections::HashMap;

pub async fn get_all_settings() -> Result<HashMap<String, String>, DBError> {
    let pool = get_db_pool().await?;
    let conn = pool.get()?;
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

pub async fn get_setting(key: &str) -> Result<Option<String>, DBError> {
    let pool = get_db_pool().await?;
    let conn = pool.get()?;
    let mut stmt = conn.prepare("SELECT value FROM settings WHERE key = ?")?;
    let mut rows = stmt.query(params![key])?;

    if let Some(row) = rows.next()? {
        Ok(Some(row.get(0)?))
    } else {
        Ok(None)
    }
}

pub async fn set_setting(key: &str, value: &str) -> Result<(), DBError> {
    let pool = get_db_pool().await?;
    let conn = pool.get()?;
    conn.execute(
        "INSERT OR REPLACE INTO settings (key, value) VALUES (?, ?)",
        params![key, value],
    )?;
    Ok(())
}
