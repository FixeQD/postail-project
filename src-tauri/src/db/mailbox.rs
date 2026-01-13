use rusqlite::{params, Connection, Result as SqlResult};
use crate::error::DBError;

pub fn fetch_mailboxes(conn: &Connection, account_id: &str) -> Result<Vec<Mailbox>, DBError> {
    let mut stmt = conn.prepare(
        "SELECT name, uid_validity, highest_modseq, last_synced_uid FROM mailboxes WHERE account_id = ?",
    )?;
    let mailboxes_iter = stmt.query_map([account_id], |row| {
        Ok(Mailbox {
            name: row.get(0)?,
            uid_validity: row.get(1)?,
            highest_modseq: row.get(2)?,
            last_synced_uid: row.get(3)?,
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
        "INSERT OR REPLACE INTO mailboxes (account_id, name, uid_validity, highest_modseq, last_synced_uid)
         VALUES (?, ?, ?, ?, ?)",
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


pub fn upsert_mailbox(
    conn: &Connection,
    account_id: &str,
    mailbox: &Mailbox,
) -> Result<(), DBError> {
    conn.execute(
        "INSERT OR REPLACE INTO mailboxes (account_id, name, uid_validity, highest_modseq, last_synced_uid)
         VALUES (?, ?, ?, ?, ?)",
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


pub fn upsert_mailbox(
    conn: &Connection,
    account_id: &str,
    mailbox: &Mailbox,
) -> Result<(), DBError> {
    conn.execute(
        "INSERT OR REPLACE INTO mailboxes (account_id, name, uid_validity, highest_modseq, last_synced_uid)
         VALUES (?, ?, ?, ?, ?)",
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
