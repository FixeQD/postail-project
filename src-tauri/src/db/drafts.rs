use crate::db::sql_helpers::*;
use crate::error::DBError;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Serialize, Deserialize)]
pub struct Draft {
    pub id: String,
    pub account_id: String,
    pub subject: Option<String>,
    pub body: Option<String>,
    pub to: Vec<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

pub fn save_draft(conn: &Connection, draft: &Draft) -> Result<(), DBError> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    let to_json = serde_json::to_string(&draft.to).map_err(|e| DBError::Json(e))?;

    conn.execute(
        "INSERT OR REPLACE INTO drafts (id, account_id, subject, body, to_json, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, ?)",
        (
            &draft.id,
            &draft.account_id,
            &draft.subject,
            &draft.body,
            &to_json,
            draft.created_at,
            now,
        ),
    )?;

    Ok(())
}

pub fn load_draft(conn: &Connection, id: &str) -> Result<Option<Draft>, DBError> {
    let mut stmt = conn.prepare("SELECT id, account_id, subject, body, to_json, created_at, updated_at FROM drafts WHERE id = ?")?;
    let mut rows = stmt.query_map([id], |row| {
        let to_json: String = row.get(4)?;
        let to: Vec<String> = serde_json::from_str(&to_json).unwrap_or_default();
        Ok(Draft {
            id: row.get(0)?,
            account_id: row.get(1)?,
            subject: row.get(2)?,
            body: row.get(3)?,
            to,
            created_at: row.get(5)?,
            updated_at: row.get(6)?,
        })
    })?;

    if let Some(draft) = rows.next() {
        Ok(Some(draft?))
    } else {
        Ok(None)
    }
}

pub fn list_drafts(conn: &Connection, account_id: &str) -> Result<Vec<Draft>, DBError> {
    let mut stmt = conn.prepare("SELECT id, account_id, subject, body, to_json, created_at, updated_at FROM drafts WHERE account_id = ? ORDER BY updated_at DESC")?;
    let rows = stmt.query_map([account_id], |row| {
        let to_json: String = row.get(4)?;
        let to: Vec<String> = serde_json::from_str(&to_json).unwrap_or_default();
        Ok(Draft {
            id: row.get(0)?,
            account_id: row.get(1)?,
            subject: row.get(2)?,
            body: row.get(3)?,
            to,
            created_at: row.get(5)?,
            updated_at: row.get(6)?,
        })
    })?;

    let mut drafts = Vec::new();
    for draft in rows {
        drafts.push(draft?);
    }
    Ok(drafts)
}

pub fn delete_draft(conn: &Connection, id: &str) -> Result<(), DBError> {
    conn.execute("DELETE FROM drafts WHERE id = ?", [id])?;
    Ok(())
}
