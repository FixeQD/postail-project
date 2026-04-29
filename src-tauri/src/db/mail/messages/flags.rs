use rusqlite::{Connection, params};
use crate::error::DBError;

pub fn update_message_flags(
    conn: &Connection,
    account_id: &str,
    mailbox: &str,
    uids: &[u32],
    add_flags: Option<&[&str]>,
    remove_flags: Option<&[&str]>,
) -> Result<usize, DBError> {
    if uids.is_empty() {
        return Ok(0);
    }

    for uid in uids {
        let current_flags: Option<String> = conn.query_row(
            "SELECT flags_json FROM messages WHERE account_id = ? AND mailbox = ? AND uid = ?",
            params![account_id, mailbox, uid],
            |row| row.get(0),
        )?;

        let mut flags: Vec<String> = current_flags
            .as_deref()
            .and_then(|s| serde_json::from_str(s).ok())
            .unwrap_or_default();

        if let Some(add) = add_flags {
            for flag in add {
                if !flags.contains(&flag.to_string()) {
                    flags.push(flag.to_string());
                }
            }
        }

        if let Some(remove) = remove_flags {
            for flag in remove {
                flags.retain(|f| f != flag);
            }
        }

        conn.execute(
            "UPDATE messages SET flags_json = ? WHERE account_id = ? AND mailbox = ? AND uid = ?",
            params![
                serde_json::to_string(&flags).unwrap_or_default(),
                account_id,
                mailbox,
                uid
            ],
        )?;
    }

    Ok(uids.len())
}

pub fn mark_read(
    conn: &Connection,
    account_id: &str,
    mailbox: &str,
    uids: &[u32],
    read: bool,
) -> Result<usize, DBError> {
    if read {
        update_message_flags(conn, account_id, mailbox, uids, Some(&["\\Seen"]), None)
    } else {
        update_message_flags(conn, account_id, mailbox, uids, None, Some(&["\\Seen"]))
    }
}

pub fn move_to_trash(
    conn: &Connection,
    account_id: &str,
    mailbox: &str,
    uids: &[u32],
) -> Result<usize, DBError> {
    let trash = crate::db::mail::mailbox::get_mailbox_by_role(conn, account_id, "trash")?
        .ok_or_else(|| {
            tracing::error!(target: "postail", "Trash mailbox not found for account {}", account_id);
            rusqlite::Error::QueryReturnedNoRows
        })?;

    for uid in uids {
        crate::db::mail::flag_queue::enqueue_move_operation(
            conn, account_id, mailbox, &trash, *uid,
        )?;
    }

    let placeholders = uids.iter().map(|_| "?").collect::<Vec<_>>().join(",");

    let mut params_update: Vec<rusqlite::types::Value> = vec![
        trash.to_string().into(),
        account_id.to_string().into(),
        mailbox.to_string().into(),
    ];
    for &uid in uids {
        params_update.push(uid.into());
    }
    conn.execute(
        &format!("UPDATE OR IGNORE messages SET mailbox = ?, uid = -uid WHERE account_id = ? AND mailbox = ? AND uid IN ({})", placeholders),
        rusqlite::params_from_iter(params_update)
    )?;

    let query = format!(
        "DELETE FROM messages WHERE account_id = ? AND mailbox = ? AND uid IN ({})",
        placeholders
    );

    let mut params: Vec<rusqlite::types::Value> =
        vec![account_id.to_string().into(), mailbox.to_string().into()];
    for uid in uids {
        params.push((*uid).into());
    }

    let deleted = conn.execute(&query, rusqlite::params_from_iter(params))?;
    Ok(deleted)
}

/// Toggle the starred flag for a single message and return the new state.
pub fn toggle_starred(
    conn: &Connection,
    account_id: &str,
    mailbox: &str,
    uid: u32,
) -> Result<bool, DBError> {
    conn.execute(
        "UPDATE messages SET starred = CASE WHEN starred = 1 THEN 0 ELSE 1 END
         WHERE account_id = ? AND mailbox = ? AND uid = ?",
        params![account_id, mailbox, uid],
    )?;

    let new_state: i64 = conn.query_row(
        "SELECT starred FROM messages WHERE account_id = ? AND mailbox = ? AND uid = ?",
        params![account_id, mailbox, uid],
        |row| row.get(0),
    )?;

    Ok(new_state != 0)
}

/// Explicitly set the starred flag
pub fn set_starred(
    conn: &Connection,
    account_id: &str,
    mailbox: &str,
    uid: u32,
    starred: bool,
) -> Result<(), DBError> {
    conn.execute(
        "UPDATE messages SET starred = ? WHERE account_id = ? AND mailbox = ? AND uid = ?",
        params![if starred { 1i64 } else { 0i64 }, account_id, mailbox, uid],
    )?;
    Ok(())
}
