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

/// Persists a Draft into the database, inserting or replacing the row with the same id.
///
/// Serializes the draft's `to`, `cc`, `bcc`, and `attachments` fields to JSON, sets `updated_at` to
/// the current UNIX epoch seconds, and uses the draft's `created_at` value as provided. JSON
/// serialization failures are returned as `DBError::Json`; database errors are propagated.
///
/// # Examples
///
/// ```
/// use rusqlite::Connection;
/// // assume Draft and DraftAttachment are in scope and serde derives are available
/// let conn = Connection::open_in_memory().unwrap();
/// // table creation omitted for brevity; assume `drafts` table exists with the expected schema
/// let draft = Draft {
///     id: "d1".into(),
///     account_id: "a1".into(),
///     subject: Some("Hello".into()),
///     body: Some("Body".into()),
///     to: vec!["to@example.com".into()],
///     cc: Vec::new(),
///     bcc: Vec::new(),
///     attachments: Vec::new(),
///     created_at: 1,
///     updated_at: 1,
/// };
/// let res = save_draft(&conn, &draft);
/// assert!(res.is_ok());
/// ```
pub fn save_draft(conn: &Connection, draft: &Draft) -> Result<(), DBError> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    let to_json = serde_json::to_string(&draft.to).map_err(DBError::Json)?;
    let cc_json = serde_json::to_string(&draft.cc).map_err(DBError::Json)?;
    let bcc_json = serde_json::to_string(&draft.bcc).map_err(DBError::Json)?;
    let attachments_json = serde_json::to_string(&draft.attachments).map_err(DBError::Json)?;

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

/// Fetches a draft by its id from the database.
///
/// Missing or invalid JSON stored in the recipient (to/cc/bcc) or attachments columns is
/// treated as an empty collection when constructing the returned Draft.
///
/// # Returns
///
/// `Ok(Some(Draft))` if a draft with the given id exists, `Ok(None)` if no row matches,
/// or `Err(DBError)` if a database error occurs.
///
/// # Examples
///
/// ```
/// # use my_crate::db::load_draft;
/// # let conn = /* obtain rusqlite::Connection */ unimplemented!();
/// let draft = load_draft(&conn, "draft-id-123");
/// match draft {
///     Ok(Some(d)) => assert_eq!(d.id, "draft-id-123"),
///     Ok(None) => println!("no draft found"),
///     Err(e) => panic!("database error: {:?}", e),
/// }
/// ```
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
            to: to_json
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or_default(),
            cc: cc_json
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or_default(),
            bcc: bcc_json
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or_default(),
            attachments: attachments_json
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or_default(),
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

/// Retrieves all drafts belonging to the specified account, ordered by `updated_at` descending.
///
/// JSON columns (`to_json`, `cc_json`, `bcc_json`, `attachments_json`) are parsed into their
/// corresponding fields; if a JSON column is missing or fails to parse, the field falls back to
/// an empty vector.
///
/// # Returns
///
/// A `Vec<Draft>` containing all drafts for `account_id`, newest first.
///
/// # Examples
///
/// ```
/// let drafts = list_drafts(&conn, "account_1").unwrap();
/// assert!(drafts.iter().all(|d| d.account_id == "account_1"));
/// ```
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
            to: to_json
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or_default(),
            cc: cc_json
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or_default(),
            bcc: bcc_json
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or_default(),
            attachments: attachments_json
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or_default(),
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

/// Delete a draft by its ID from the database.
///
/// Removes the row in the `drafts` table whose `id` matches the provided `id`.
///
/// # Parameters
///
/// - `id`: The draft's unique identifier.
///
/// # Examples
///
/// ```
/// use rusqlite::Connection;
/// # use crate::db::drafts::{Draft, delete_draft};
/// let conn = Connection::open_in_memory().unwrap();
/// conn.execute_batch(r#"
///     CREATE TABLE drafts (id TEXT PRIMARY KEY, account_id TEXT, subject TEXT, body TEXT, to_json TEXT, cc_json TEXT, bcc_json TEXT, attachments_json TEXT, created_at INTEGER, updated_at INTEGER);
///     INSERT INTO drafts (id, account_id, created_at, updated_at) VALUES ('d1', 'a1', 0, 0);
/// "#).unwrap();
///
/// delete_draft(&conn, "d1").unwrap();
/// let count: i64 = conn.query_row("SELECT COUNT(1) FROM drafts WHERE id = 'd1'", [], |r| r.get(0)).unwrap();
/// assert_eq!(count, 0);
/// ```
pub fn delete_draft(conn: &Connection, id: &str) -> Result<(), DBError> {
    conn.execute("DELETE FROM drafts WHERE id = ?", [id])?;
    Ok(())
}