use crate::error::DBError;
use chrono::Utc;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Signature {
    pub id: String,
    pub account_id: String,
    pub name: String,
    pub html_content: String,
    pub is_default: bool,
    pub created_at: i64,
    pub updated_at: i64,
}

pub fn list_signatures(conn: &Connection, account_id: &str) -> Result<Vec<Signature>, DBError> {
    let mut stmt = conn.prepare(
        "SELECT id, account_id, name, html_content, is_default, created_at, updated_at
         FROM signatures WHERE account_id = ? ORDER BY created_at ASC",
    )?;
    let rows = stmt.query_map([account_id], |row| {
        Ok(Signature {
            id: row.get(0)?,
            account_id: row.get(1)?,
            name: row.get(2)?,
            html_content: row.get(3)?,
            is_default: row.get::<_, i32>(4)? != 0,
            created_at: row.get(5)?,
            updated_at: row.get(6)?,
        })
    })?;

    let mut signatures = Vec::new();
    for sig in rows {
        signatures.push(sig?);
    }
    Ok(signatures)
}

pub fn get_signature(
    conn: &Connection,
    id: &str,
    account_id: &str,
) -> Result<Option<Signature>, DBError> {
    let mut stmt = conn.prepare(
        "SELECT id, account_id, name, html_content, is_default, created_at, updated_at
         FROM signatures WHERE id = ? AND account_id = ?",
    )?;
    let mut rows = stmt.query_map([id, account_id], |row| {
        Ok(Signature {
            id: row.get(0)?,
            account_id: row.get(1)?,
            name: row.get(2)?,
            html_content: row.get(3)?,
            is_default: row.get::<_, i32>(4)? != 0,
            created_at: row.get(5)?,
            updated_at: row.get(6)?,
        })
    })?;

    if let Some(sig) = rows.next() {
        Ok(Some(sig?))
    } else {
        Ok(None)
    }
}

pub fn save_signature(conn: &Connection, sig: &Signature) -> Result<(), DBError> {
    let now = Utc::now().timestamp();

    if sig.is_default {
        conn.execute(
            "UPDATE signatures SET is_default = 0 WHERE account_id = ?",
            [&sig.account_id],
        )?;
    }

    conn.execute(
        "INSERT OR REPLACE INTO signatures (id, account_id, name, html_content, is_default, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, ?)",
        (
            &sig.id,
            &sig.account_id,
            &sig.name,
            &sig.html_content,
            sig.is_default as i32,
            sig.created_at,
            now,
        ),
    )?;

    Ok(())
}

pub fn delete_signature(conn: &Connection, id: &str, account_id: &str) -> Result<(), DBError> {
    conn.execute(
        "DELETE FROM signatures WHERE id = ? AND account_id = ?",
        [id, account_id],
    )?;
    Ok(())
}
