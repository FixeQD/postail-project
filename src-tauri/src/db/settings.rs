use crate::error::DBError;
use crate::globals::DB_CONN;
use rusqlite::params;
use std::collections::HashMap;

pub async fn get_all_settings() -> Result<HashMap<String, String>, DBError> {
    let conn_guard = DB_CONN.lock().await;
    let conn = conn_guard.as_ref().ok_or(DBError::Security(
        crate::error::SecurityError::KeyDerivation("Database not initialized".to_string()),
    ))?;
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
    let conn_guard = DB_CONN.lock().await;
    let conn = conn_guard.as_ref().ok_or(DBError::Security(
        crate::error::SecurityError::KeyDerivation("Database not initialized".to_string()),
    ))?;
    let mut stmt = conn.prepare("SELECT value FROM settings WHERE key = ?")?;
    let mut rows = stmt.query(params![key])?;

    if let Some(row) = rows.next()? {
        Ok(Some(row.get(0)?))
    } else {
        Ok(None)
    }
}

pub async fn set_setting(key: &str, value: &str) -> Result<(), DBError> {
    let conn_guard = DB_CONN.lock().await;
    let conn = conn_guard.as_ref().ok_or(DBError::Security(
        crate::error::SecurityError::KeyDerivation("Database not initialized".to_string()),
    ))?;
    conn.execute(
        "INSERT OR REPLACE INTO settings (key, value) VALUES (?, ?)",
        params![key, value],
    )?;
    Ok(())
}
