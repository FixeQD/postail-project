use rusqlite::{params, Connection};

use crate::db::Mailbox;
use crate::error::DBError;

pub fn fetch_mailboxes(conn: &Connection, account_id: &str) -> Result<Vec<Mailbox>, DBError> {
    let mut stmt = conn.prepare(
        "SELECT name, role, uid_validity, highest_modseq, last_synced_uid FROM mailboxes WHERE account_id = ?",
    )?;
    let mailboxes_iter = stmt.query_map([account_id], |row| {
        Ok(Mailbox {
            name: row.get::<_, String>(0)?,
            display_name: row.get::<_, String>(0)?, // Default to name
            role: row
                .get::<_, Option<String>>(1)?
                .unwrap_or_else(|| "other".to_string()),
            uid_validity: row.get(2)?,
            highest_modseq: row.get(3)?,
            last_synced_uid: row.get(4)?,
        })
    })?;
    let mailboxes: Result<Vec<Mailbox>, _> = mailboxes_iter.collect();
    mailboxes.map_err(DBError::Sqlite)
}

pub fn upsert_mailbox(
    conn: &Connection,
    account_id: &str,
    mailbox: &Mailbox,
) -> Result<(), DBError> {
    conn.execute(
        "INSERT INTO mailboxes (account_id, name, role, uid_validity, highest_modseq, last_synced_uid)
         VALUES (?, ?, ?, ?, ?, ?)
         ON CONFLICT(account_id, name) DO UPDATE SET
            role = CASE WHEN role_customized = 1 THEN role ELSE excluded.role END,
            uid_validity = excluded.uid_validity,
            highest_modseq = excluded.highest_modseq,
            last_synced_uid = excluded.last_synced_uid",
        params![
            account_id,
            mailbox.name,
            mailbox.role,
            mailbox.uid_validity,
            mailbox.highest_modseq,
            mailbox.last_synced_uid,
        ],
    )?;
    Ok(())
}

pub fn get_mailbox_by_role(
    conn: &Connection,
    account_id: &str,
    role: &str,
) -> Result<Option<String>, DBError> {
    let result: Result<String, rusqlite::Error> = conn.query_row(
        "SELECT name FROM mailboxes WHERE account_id = ? AND role = ? LIMIT 1",
        params![account_id, role],
        |row| row.get(0),
    );
    match result {
        Ok(name) => Ok(Some(name)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.into()),
    }
}
