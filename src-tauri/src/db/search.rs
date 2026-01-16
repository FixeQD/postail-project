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
    let mut result = String::with_capacity(query.len() * 2);
    let mut in_phrase = false;
    let mut chars = query.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '"' if !in_phrase => {
                in_phrase = true;
                result.push(c);
            }
            '"' if in_phrase => {
                in_phrase = false;
                result.push(c);
            }
            '-' | '*' | '(' | ')' | ':' | '^' if !in_phrase => {
                result.push('\\');
                result.push(c);
            }
            '"' => result.push_str("\"\""),
            '\\' if !in_phrase => {
                if let Some(&next) = chars.peek() {
                    if matches!(next, '-' | '*' | '(' | ')' | ':' | '^' | '"') {
                        result.push(next);
                        chars.next();
                    } else {
                        result.push(c);
                    }
                } else {
                    result.push(c);
                }
            }
            _ => result.push(c),
        }
    }

    result
}

pub fn parse_fts_query(query: &str) -> String {
    let escaped = escape_fts_query(query);

    if escaped.contains(' ') && !escaped.starts_with('"') && !escaped.ends_with('"') {
        format!("\"{}\"", escaped)
    } else {
        escaped
    }
}

pub fn parse_advanced_fts_query(query: &str) -> String {
    let normalized = query.trim();

    if normalized.is_empty() {
        return "\"\"".to_string();
    }

    if normalized.starts_with('"') && normalized.ends_with('"') {
        return escape_fts_query(normalized);
    }

    let has_boolean = normalized.contains(" AND ")
        || normalized.contains(" OR ")
        || normalized.contains(" NOT ");

    if has_boolean {
        let processed = normalized
            .replace(" AND ", " AND ")
            .replace(" OR ", " OR ")
            .replace(" NOT ", " NOT ");

        let tokens: Vec<&str> = processed.split_whitespace().collect();
        let mut result = String::new();
        let mut expect_operand = true;

        for token in tokens {
            match token {
                "AND" | "OR" | "NOT" => {
                    result.push_str(&format!(" {} ", token.to_uppercase()));
                    expect_operand = true;
                }
                _ => {
                    if !expect_operand {
                        result.push(' ');
                    }
                    if token.starts_with('-') {
                        result.push_str(token);
                    } else if token.contains(':') {
                        result.push_str(token);
                    } else {
                        result.push_str(&format!("\"{}\"", escape_fts_query(token)));
                    }
                    expect_operand = false;
                }
            }
        }

        result
    } else {
        parse_fts_query(normalized)
    }
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

    let mut query_params = vec![parse_fts_query(query)];
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

pub fn search_messages_advanced(
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

    let mut query_params = vec![parse_advanced_fts_query(query)];
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
