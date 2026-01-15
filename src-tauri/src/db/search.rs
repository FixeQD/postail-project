use crate::error::DBError;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResult {
    pub message_id: i64,
    pub account_id: String,
    pub mailbox: String,
    pub uid: u32,
    pub subject: Option<String>,
    pub from_addr: Option<String>,
    pub snippet: Option<String>,
    pub rank: f64,
}

pub fn escape_fts_query(query: &str) -> String {
    query
        .replace('"', "\"\"")
        .chars()
        .map(|c| match c {
            '-' | '*' | '(' | ')' => format!(r"\{c}"),
            _ => c.to_string(),
        })
        .collect()
}

pub fn search_messages(
    conn: &Connection,
    account_id: Option<&str>,
    mailbox: Option<&str>,
    query: &str,
    limit: u32,
) -> Result<Vec<SearchResult>, DBError> {
    let (where_clause, params): (String, Vec<String>) = match (account_id, mailbox) {
        (Some(a), Some(m)) => (
            "AND m.account_id = ? AND m.mailbox = ?".to_string(),
            vec![a.to_string(), m.to_string()],
        ),
        (Some(a), None) => ("AND m.account_id = ?".to_string(), vec![a.to_string()]),
        (None, None) => ("".to_string(), vec![]),
        _ => return Err(DBError::Sqlite(rusqlite::Error::InvalidQuery)),
    };

    let sql = format!(
        "SELECT m.id, m.account_id, m.mailbox, m.uid, m.subject, m.from_addr,
                m.snippet, messages_fts.rank
         FROM messages_fts
         JOIN messages m ON messages_fts.rowid = m.id
         WHERE messages_fts MATCH ? {}
         ORDER BY rank DESC LIMIT ?",
        where_clause
    );

    let mut query_params = vec![escape_fts_query(query)];
    query_params.extend(params);
    query_params.push(limit.to_string());

    let mut stmt = conn.prepare(&sql)?;
    let results_iter = stmt.query_map(rusqlite::params_from_iter(query_params), |row| {
        Ok(SearchResult {
            message_id: row.get(0)?,
            account_id: row.get(1)?,
            mailbox: row.get(2)?,
            uid: row.get::<_, i64>(3)? as u32,
            subject: row.get(4)?,
            from_addr: row.get(5)?,
            snippet: row.get(6)?,
            rank: row.get(7)?,
        })
    })?;

    let results: Result<Vec<SearchResult>, _> = results_iter.collect();
    results.map_err(DBError::Sqlite)
}
