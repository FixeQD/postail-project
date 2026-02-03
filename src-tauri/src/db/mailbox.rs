use rusqlite::{params, Connection};

use crate::db::Mailbox;
use crate::error::DBError;

pub fn fetch_mailboxes(conn: &Connection, account_id: &str) -> Result<Vec<Mailbox>, DBError> {
    let mut stmt = conn.prepare(
        "SELECT name, uid_validity, highest_modseq, last_synced_uid FROM mailboxes WHERE account_id = ?",
    )?;
    let mailboxes_iter = stmt.query_map([account_id], |row| {
        Ok(Mailbox {
            name: row.get(0)?,
            display_name: row.get(0)?, // Default to name
            role: "other".to_string(), // Default role
            uid_validity: row.get(1)?,
            highest_modseq: row.get(2)?,
            last_synced_uid: row.get(3)?,
        })
    })?;
    let mailboxes: Result<Vec<Mailbox>, _> = mailboxes_iter.collect();
    mailboxes.map_err(DBError::Sqlite)
}

/// Inserts a mailbox for the given account or updates its sync metadata if a mailbox with the same name already exists.
///
/// On conflict of (account_id, name), updates `uid_validity`, `highest_modseq`, and `last_synced_uid` with the provided values.
///
/// # Returns
///
/// `Ok(())` on success, `Err(DBError)` if the database operation fails.
///
/// # Examples
///
/// ```
/// # use rusqlite::Connection;
/// # // `Mailbox` and `upsert_mailbox` are defined in the same crate.
/// let conn = Connection::open_in_memory().unwrap();
/// // Create the table schema used by `upsert_mailbox` for the example.
/// conn.execute_batch("CREATE TABLE mailboxes (account_id TEXT, name TEXT, uid_validity INTEGER, highest_modseq INTEGER, last_synced_uid INTEGER, PRIMARY KEY(account_id, name));").unwrap();
///
/// let mailbox = Mailbox {
///     name: "INBOX".into(),
///     display_name: "INBOX".into(),
///     role: "other".into(),
///     uid_validity: 1,
///     highest_modseq: 0,
///     last_synced_uid: 0,
/// };
///
/// upsert_mailbox(&conn, "account_123", &mailbox).unwrap();
/// ```
pub fn upsert_mailbox(
    conn: &Connection,
    account_id: &str,
    mailbox: &Mailbox,
) -> Result<(), DBError> {
    conn.execute(
        "INSERT INTO mailboxes (account_id, name, uid_validity, highest_modseq, last_synced_uid)
         VALUES (?, ?, ?, ?, ?)
         ON CONFLICT(account_id, name) DO UPDATE SET
            uid_validity = excluded.uid_validity,
            highest_modseq = excluded.highest_modseq,
            last_synced_uid = excluded.last_synced_uid",
        params![
            account_id,
            mailbox.name,
            mailbox.uid_validity,
            mailbox.highest_modseq,
            mailbox.last_synced_uid,
        ],
    )?;
    Ok(())
}