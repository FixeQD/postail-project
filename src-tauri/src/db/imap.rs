use crate::error::DBError;
use rusqlite::{params, Connection};

pub fn check_uidvalidity(
    conn: &Connection,
    account_id: &str,
    mailbox: &str,
    remote_validity: u32,
) -> Result<bool, DBError> {
    let local: Option<u32> = conn
        .query_row(
            "SELECT uid_validity FROM mailboxes WHERE account_id = ? AND name = ?",
            params![account_id, mailbox],
            |row| row.get(0),
        )
        .ok();

    match local {
        Some(v) if v != remote_validity => {
            conn.execute(
                "DELETE FROM messages WHERE account_id = ? AND mailbox = ?",
                params![account_id, mailbox],
            )?;
            Ok(true)
        }
        _ => Ok(false),
    }
}

pub fn get_mailbox_metadata(
    conn: &Connection,
    account_id: &str,
    mailbox: &str,
) -> Result<(u32, Option<u64>), DBError> {
    conn.query_row(
        "SELECT uid_validity, highest_modseq FROM mailboxes
         WHERE account_id = ? AND name = ?",
        params![account_id, mailbox],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )
    .map_err(DBError::Sqlite)
}

pub fn update_highest_modseq(
    conn: &Connection,
    account_id: &str,
    mailbox: &str,
    modseq: u64,
) -> Result<(), DBError> {
    conn.execute(
        "UPDATE mailboxes SET highest_modseq = ?
         WHERE account_id = ? AND name = ?",
        params![modseq, account_id, mailbox],
    )?;
    Ok(())
}

pub fn update_message_flags(
    conn: &Connection,
    account_id: &str,
    mailbox: &str,
    uid: u32,
    flags: &[String],
) -> Result<(), DBError> {
    let current: String = conn.query_row(
        "SELECT flags_json FROM messages WHERE account_id = ? AND mailbox = ? AND uid = ?",
        params![account_id, mailbox, uid],
        |row| row.get(0),
    )?;

    let current_flags: Vec<String> = serde_json::from_str(&current)?;
    if current_flags != flags {
        conn.execute(
            "UPDATE messages SET flags_json = ? WHERE account_id = ? AND mailbox = ? AND uid = ?",
            params![serde_json::to_string(flags)?, account_id, mailbox, uid],
        )?;
    }
    Ok(())
}
