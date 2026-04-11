use crate::db::MailHeader;
use crate::error::AppError;
use crate::imap::ImapManager;
use rusqlite::params_from_iter;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImapSearchCriteria {
    pub from: Option<String>,
    pub to: Option<String>,
    pub subject: Option<String>,
    pub body: Option<String>,
    pub since: Option<String>,  // YYYY-MM-DD
    pub before: Option<String>, // YYYY-MM-DD
    pub has_attachment: Option<bool>,
}

const IMAP_MONTHS: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

/// Converts YYYY-MM-DD to DD-Mon-YYYY for IMAP date criteria
/// Returns None if the input is malformed
fn to_imap_date(iso: &str) -> Option<String> {
    let parts: Vec<&str> = iso.split('-').collect();
    if parts.len() != 3 {
        return None;
    }
    let year = parts[0];
    let month: usize = parts[1].parse::<usize>().ok()?;
    if !(1..=12).contains(&month) {
        return None;
    }
    let day: u32 = parts[2].parse().ok()?;
    if !(1..=31).contains(&day) {
        return None;
    }
    let month_name = IMAP_MONTHS.get(month - 1)?;
    Some(format!("{:02}-{}-{}", day, month_name, year))
}

impl ImapSearchCriteria {
    /// Builds an IMAP SEARCH criteria string from the struct fields
    /// Returns None if no criteria are set
    pub fn build_criteria(&self) -> Option<String> {
        let mut parts: Vec<String> = Vec::new();

        if let Some(from) = &self.from {
            if !from.trim().is_empty() {
                parts.push(format!("FROM \"{}\"", from.replace('"', "\\\"")));
            }
        }

        if let Some(to) = &self.to {
            if !to.trim().is_empty() {
                parts.push(format!("TO \"{}\"", to.replace('"', "\\\"")));
            }
        }

        if let Some(subject) = &self.subject {
            if !subject.trim().is_empty() {
                parts.push(format!("SUBJECT \"{}\"", subject.replace('"', "\\\"")));
            }
        }

        if let Some(body) = &self.body {
            if !body.trim().is_empty() {
                parts.push(format!("BODY \"{}\"", body.replace('"', "\\\"")));
            }
        }

        if let Some(since) = &self.since {
            if let Some(imap_date) = to_imap_date(since) {
                parts.push(format!("SINCE {}", imap_date));
            }
        }

        if let Some(before) = &self.before {
            if let Some(imap_date) = to_imap_date(before) {
                parts.push(format!("BEFORE {}", imap_date));
            }
        }

        if self.has_attachment == Some(true) {
            parts.push("HEADER Content-Type multipart".to_string());
        }

        if parts.is_empty() {
            return None;
        }

        Some(parts.join(" "))
    }
}

fn build_uid_in_clause(count: usize) -> String {
    (0..count).map(|_| "?").collect::<Vec<_>>().join(", ")
}

fn get_headers_by_uids(
    conn: &rusqlite::Connection,
    account_id: &str,
    mailbox: &str,
    uids: &[u32],
) -> Result<Vec<MailHeader>, crate::error::DBError> {
    if uids.is_empty() {
        return Ok(vec![]);
    }

    let mut all_headers = Vec::new();

    for chunk in uids.chunks(500) {
        let placeholders = build_uid_in_clause(chunk.len());
        let sql = format!(
            "SELECT uid, message_id, internal_date, subject, from_addr, to_json, cc_json, flags_json, snippet,
                    has_attachments, starred, mailbox,
                    (SELECT json_group_array(tag) FROM message_tags mt WHERE mt.message_id = m.id) as tags_json
             FROM messages m
             WHERE account_id = ? AND mailbox = ? AND uid IN ({})
             ORDER BY uid DESC",
            placeholders
        );

        let mut params: Vec<rusqlite::types::Value> = vec![
            rusqlite::types::Value::Text(account_id.to_string()),
            rusqlite::types::Value::Text(mailbox.to_string()),
        ];
        params.extend(
            chunk
                .iter()
                .map(|&u| rusqlite::types::Value::Integer(u as i64)),
        );

        let mut stmt = conn.prepare(&sql)?;
        let headers_iter = stmt.query_map(params_from_iter(params.iter()), |row| {
            let to_json: Option<String> = row.get(5)?;
            let to: Vec<String> = to_json
                .map(|s| serde_json::from_str(&s).unwrap_or_default())
                .unwrap_or_default();
            let cc_json: Option<String> = row.get(6)?;
            let cc: Vec<String> = cc_json
                .map(|s| serde_json::from_str(&s).unwrap_or_default())
                .unwrap_or_default();
            let flags_json: Option<String> = row.get(7)?;
            let flags: Vec<String> = flags_json
                .map(|s| serde_json::from_str(&s).unwrap_or_default())
                .unwrap_or_default();
            let tags_json: Option<String> = row.get(12)?;
            let tags: Vec<String> = tags_json
                .map(|s| serde_json::from_str(&s).unwrap_or_default())
                .unwrap_or_default();

            let ts: i64 = row.get(2)?;
            let internal_date = crate::db::mail::messages::safe_timestamp_from_utc(ts)
                .ok_or_else(|| rusqlite::Error::InvalidColumnName("internal_date".into()))?;

            Ok(MailHeader {
                uid: row.get::<_, u32>(0)?,
                mailbox: row.get(11)?,
                message_id: row.get(1)?,
                internal_date,
                subject: row.get(3)?,
                from: vec![row.get::<_, Option<String>>(4)?.unwrap_or_default()],
                to,
                cc,
                flags,
                snippet: row.get(8)?,
                has_attachments: row.get::<_, i64>(9)? != 0,
                starred: row.get::<_, i64>(10)? != 0,
                tags,
            })
        })?;

        for header in headers_iter {
            all_headers.push(header.map_err(crate::error::DBError::Sqlite)?);
        }
    }

    Ok(all_headers)
}

impl ImapManager {
    /// Runs IMAP UID SEARCH on the server, then hydrates matching MailHeaders from local DB
    /// If a UID is not yet in the local DB (not synced), it is silently skipped
    pub async fn uid_search_messages(
        &self,
        account_id: &str,
        mailbox: &str,
        criteria: &ImapSearchCriteria,
    ) -> Result<Vec<MailHeader>, AppError> {
        let criteria_str = match criteria.build_criteria() {
            Some(c) => c,
            None => return Ok(vec![]),
        };

        tracing::debug!(
            target: "postail",
            "[IMAP] UID SEARCH {}@{}: {}",
            mailbox, account_id, criteria_str
        );

        let uids: Vec<u32> = {
            let mut session = self.connect_imap(account_id).await?;
            session.select(mailbox).await.map_err(AppError::from)?;

            let uid_set = session
                .uid_search(&criteria_str)
                .await
                .map_err(AppError::from)?;

            session.logout().await.map_err(AppError::from)?;

            let mut uids: Vec<u32> = uid_set.into_iter().collect();
            uids.sort_unstable_by(|a, b| b.cmp(a)); // newest first
            uids.truncate(1000);
            uids
        };

        if uids.is_empty() {
            return Ok(vec![]);
        }

        tracing::debug!(
            target: "postail",
            "[IMAP] UID SEARCH returned {} UIDs for {}@{}",
            uids.len(), mailbox, account_id
        );

        let pool = crate::globals::get_db_pool().await?;
        let conn = pool
            .get()
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        get_headers_by_uids(&conn, account_id, mailbox, &uids)
            .map_err(|e| AppError::DatabaseError(e.to_string()))
    }
}
