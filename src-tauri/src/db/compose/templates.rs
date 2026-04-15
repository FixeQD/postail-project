use crate::error::DBError;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Template {
    pub id: String,
    pub account_id: String,
    pub name: String,
    pub subject: String,
    pub html_body: String,
    pub created_at: i64,
    pub updated_at: i64,
}

pub fn list_templates(conn: &Connection, account_id: &str) -> Result<Vec<Template>, DBError> {
    let mut stmt = conn.prepare(
        "SELECT id, account_id, name, subject, html_body, created_at, updated_at
         FROM templates WHERE account_id = ? ORDER BY name ASC",
    )?;
    let rows = stmt.query_map([account_id], |row| {
        Ok(Template {
            id: row.get(0)?,
            account_id: row.get(1)?,
            name: row.get(2)?,
            subject: row.get(3)?,
            html_body: row.get(4)?,
            created_at: row.get(5)?,
            updated_at: row.get(6)?,
        })
    })?;

    let mut templates = Vec::new();
    for tmpl in rows {
        templates.push(tmpl?);
    }
    Ok(templates)
}

pub fn get_template(
    conn: &Connection,
    id: &str,
    account_id: &str,
) -> Result<Option<Template>, DBError> {
    let mut stmt = conn.prepare(
        "SELECT id, account_id, name, subject, html_body, created_at, updated_at
         FROM templates WHERE id = ? AND account_id = ?",
    )?;
    let mut rows = stmt.query_map([id, account_id], |row| {
        Ok(Template {
            id: row.get(0)?,
            account_id: row.get(1)?,
            name: row.get(2)?,
            subject: row.get(3)?,
            html_body: row.get(4)?,
            created_at: row.get(5)?,
            updated_at: row.get(6)?,
        })
    })?;

    if let Some(tmpl) = rows.next() {
        Ok(Some(tmpl?))
    } else {
        Ok(None)
    }
}

pub fn save_template(conn: &Connection, tmpl: &Template) -> Result<(), DBError> {
    let now = chrono::Utc::now().timestamp();

    conn.execute(
        "INSERT OR REPLACE INTO templates (id, account_id, name, subject, html_body, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, ?)",
        (
            &tmpl.id,
            &tmpl.account_id,
            &tmpl.name,
            &tmpl.subject,
            &tmpl.html_body,
            tmpl.created_at,
            now,
        ),
    )?;

    Ok(())
}

pub fn delete_template(conn: &Connection, id: &str, account_id: &str) -> Result<(), DBError> {
    conn.execute(
        "DELETE FROM templates WHERE id = ? AND account_id = ?",
        [id, account_id],
    )?;
    Ok(())
}
