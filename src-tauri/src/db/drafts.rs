use crate::error::DBError;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DraftAttachment {
    pub id: String,
    pub filename: String,
    pub content_type: String,
    pub size: u64,
    pub hash: String,
    pub path: String,
    pub cid: Option<String>,
    pub inline: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Draft {
    pub id: String,
    pub account_id: String,
    pub subject: Option<String>,
    pub body: Option<String>,
    pub to: Vec<String>,
    pub cc: Vec<String>,
    pub bcc: Vec<String>,
    pub attachments: Vec<DraftAttachment>,
    pub created_at: i64,
    pub updated_at: i64,
}

pub fn save_draft(conn: &Connection, draft: &Draft) -> Result<(), DBError> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    let to_json = serde_json::to_string(&draft.to).map_err(|e| DBError::Json(e))?;
    let cc_json = serde_json::to_string(&draft.cc).map_err(|e| DBError::Json(e))?;
    let bcc_json = serde_json::to_string(&draft.bcc).map_err(|e| DBError::Json(e))?;
    let attachments_json = serde_json::to_string(&draft.attachments).map_err(|e| DBError::Json(e))?;

    conn.execute(
        "INSERT OR REPLACE INTO drafts (id, account_id, subject, body, to_json, cc_json, bcc_json, attachments_json, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        (
            &draft.id,
            &draft.account_id,
            &draft.subject,
            &draft.body,
            &to_json,
            &cc_json,
            &bcc_json,
            &attachments_json,
            draft.created_at,
            now,
        ),
    )?;

    Ok(())
}

pub fn load_draft(conn: &Connection, id: &str) -> Result<Option<Draft>, DBError> {
    let mut stmt = conn.prepare("SELECT id, account_id, subject, body, to_json, cc_json, bcc_json, attachments_json, created_at, updated_at FROM drafts WHERE id = ?")?;
    let mut rows = stmt.query_map([id], |row| {
        let to_json: Option<String> = row.get(4)?;
        let cc_json: Option<String> = row.get(5)?;
        let bcc_json: Option<String> = row.get(6)?;
        let attachments_json: Option<String> = row.get(7)?;

        Ok(Draft {
            id: row.get(0)?,
            account_id: row.get(1)?,
            subject: row.get(2)?,
            body: row.get(3)?,
            to: to_json.and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_default(),
            cc: cc_json.and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_default(),
            bcc: bcc_json.and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_default(),
            attachments: attachments_json.and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_default(),
            created_at: row.get(8)?,
            updated_at: row.get(9)?,
        })
    })?;

    if let Some(draft) = rows.next() {
        Ok(Some(draft?))
    } else {
        Ok(None)
    }
}

pub fn list_drafts(conn: &Connection, account_id: &str) -> Result<Vec<Draft>, DBError> {
    let mut stmt = conn.prepare("SELECT id, account_id, subject, body, to_json, cc_json, bcc_json, attachments_json, created_at, updated_at FROM drafts WHERE account_id = ? ORDER BY updated_at DESC")?;
    let rows = stmt.query_map([account_id], |row| {
        let to_json: Option<String> = row.get(4)?;
        let cc_json: Option<String> = row.get(5)?;
        let bcc_json: Option<String> = row.get(6)?;
        let attachments_json: Option<String> = row.get(7)?;

        Ok(Draft {
            id: row.get(0)?,
            account_id: row.get(1)?,
            subject: row.get(2)?,
            body: row.get(3)?,
            to: to_json.and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_default(),
            cc: cc_json.and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_default(),
            bcc: bcc_json.and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_default(),
            attachments: attachments_json.and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_default(),
            created_at: row.get(8)?,
            updated_at: row.get(9)?,
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
