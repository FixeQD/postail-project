use crate::error::DBError;
use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedSearch {
    pub id: String,
    pub account_id: String,
    pub name: String,
    pub query_json: String,
    pub icon: String,
    pub position: i32,
    pub created_at: i64,
}

pub fn get_saved_searches(
    conn: &Connection,
    account_id: &str,
) -> Result<Vec<SavedSearch>, DBError> {
    let mut stmt = conn.prepare(
        "SELECT id, account_id, name, query_json, icon, position, created_at
         FROM saved_searches
         WHERE account_id = ?
         ORDER BY position ASC, created_at ASC",
    )?;

    let results = stmt
        .query_map([account_id], |row| {
            Ok(SavedSearch {
                id: row.get(0)?,
                account_id: row.get(1)?,
                name: row.get(2)?,
                query_json: row.get(3)?,
                icon: row.get(4)?,
                position: row.get(5)?,
                created_at: row.get(6)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(results)
}

pub fn create_saved_search(
    conn: &Connection,
    id: &str,
    account_id: &str,
    name: &str,
    query_json: &str,
    icon: &str,
    created_at: i64,
) -> Result<SavedSearch, DBError> {
    let position: i32 = conn.query_row(
        "INSERT INTO saved_searches (id, account_id, name, query_json, icon, position, created_at)
         SELECT ?, ?, ?, ?, ?, COALESCE(MAX(position), -1) + 1, ?
         FROM saved_searches WHERE account_id = ?
         RETURNING position",
        params![
            id, account_id, name, query_json, icon, created_at, account_id
        ],
        |row| row.get::<_, i32>(0),
    )?;

    Ok(SavedSearch {
        id: id.to_string(),
        account_id: account_id.to_string(),
        name: name.to_string(),
        query_json: query_json.to_string(),
        icon: icon.to_string(),
        position,
        created_at,
    })
}

pub fn delete_saved_search(conn: &Connection, id: &str, account_id: &str) -> Result<(), DBError> {
    conn.execute(
        "DELETE FROM saved_searches WHERE id = ? AND account_id = ?",
        params![id, account_id],
    )?;
    Ok(())
}
