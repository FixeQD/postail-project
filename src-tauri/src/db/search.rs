use crate::error::DBError;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
    pub has_attachments: bool,
    pub date: i64,
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

pub fn tokenize_fts_query(query: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    for c in query.chars() {
        match c {
            '"' => {
                in_quotes = !in_quotes;
                current.push(c);
            }
            ' ' | '\t' | '\n' | '\r' if !in_quotes => {
                if !current.is_empty() {
                    tokens.push(current.clone());
                    current.clear();
                }
            }
            _ => current.push(c),
        }
    }
    if !current.is_empty() {
        tokens.push(current);
    }
    tokens
}

pub fn parse_advanced_fts_query(query: &str) -> String {
    let normalized = query.trim();

    if normalized.is_empty() {
        return "\"\"".to_string();
    }

    let is_advanced = normalized.contains(" AND ")
        || normalized.contains(" OR ")
        || normalized.contains(" NOT ")
        || normalized.contains(':');

    if is_advanced {
        let tokens = tokenize_fts_query(normalized);
        let mut result = String::new();
        let mut expect_operand = true;

        for token in tokens {
            match token.as_str() {
                "AND" | "OR" | "NOT" => {
                    result.push_str(&format!(" {} ", token.to_uppercase()));
                    expect_operand = true;
                }
                _ => {
                    if !expect_operand {
                        result.push(' ');
                    }

                    if token.starts_with("subject:")
                        || token.starts_with("from_addr:")
                        || token.starts_with("to_json:")
                        || token.starts_with("snippet:")
                        || token.starts_with("body_plain:")
                    {
                        result.push_str(&token);
                    } else if token.starts_with('-') && token.len() > 1 {
                        let term = &token[1..];
                        if term.starts_with('"') && term.ends_with('"') {
                            result.push_str(&format!("-{}", term));
                        } else {
                            result.push_str(&format!("-\"{}\"", escape_fts_query(term)));
                        }
                    } else {
                        if token.starts_with('"') && token.ends_with('"') && token.len() >= 2 {
                            result.push_str(&token);
                        } else {
                            result.push_str(&format!("\"{}\"", escape_fts_query(&token)));
                        }
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
                m.snippet, messages_fts.rank, m.has_attachments, m.internal_date
         FROM messages_fts
         JOIN messages m ON messages_fts.rowid = m.id
         WHERE messages_fts MATCH ? {}
         ORDER BY m.internal_date DESC LIMIT ?",
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
            has_attachments: row.get(8)?,
            date: row.get(9)?,
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
                m.snippet, messages_fts.rank, m.has_attachments, m.internal_date
         FROM messages_fts
         JOIN messages m ON messages_fts.rowid = m.id
         WHERE messages_fts MATCH ? {}
         ORDER BY m.internal_date DESC LIMIT ?",
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
            has_attachments: row.get(8)?,
            date: row.get(9)?,
        })
    })?;

    let results: Result<Vec<SearchResult>, _> = results_iter.collect();
    results.map_err(DBError::Sqlite)
}

fn run_body_search(
    conn: &Connection,
    account_id: Option<&str>,
    mailbox: Option<&str>,
    body_query: &str,
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
                m.snippet, message_bodies_fts.rank, m.has_attachments, m.internal_date
         FROM message_bodies_fts
         JOIN messages m ON message_bodies_fts.rowid = m.id
         WHERE message_bodies_fts MATCH ? {}
         ORDER BY m.internal_date DESC LIMIT ?",
        where_clause
    );

    let mut query_params = vec![parse_fts_query(body_query)];
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
            has_attachments: row.get(8)?,
            date: row.get(9)?,
        })
    })?;

    let results: Result<Vec<SearchResult>, _> = results_iter.collect();
    results.map_err(DBError::Sqlite)
}

/// Search across header fields and/or body. Merges and deduplicates by message_id, keeping the best rank (more negative) when a message matches in both
pub fn search_messages_with_body(
    conn: &Connection,
    account_id: Option<&str>,
    mailbox: Option<&str>,
    header_query: &str,
    body_query: &str,
    limit: u32,
) -> Result<Vec<SearchResult>, DBError> {
    let fetch_limit = limit * 2;
    let mut seen: HashMap<i64, SearchResult> = HashMap::new();

    if !header_query.trim().is_empty() {
        let header_results =
            search_messages_advanced(conn, account_id, mailbox, header_query, fetch_limit)?;
        for r in header_results {
            seen.insert(r.message_id, r);
        }
    }

    if !body_query.trim().is_empty() {
        let body_results = run_body_search(conn, account_id, mailbox, body_query, fetch_limit)?;
        for r in body_results {
            seen.entry(r.message_id)
                .and_modify(|existing| {
                    // more negative = better
                    if r.rank < existing.rank {
                        existing.rank = r.rank;
                    }
                })
                .or_insert(r);
        }
    }

    let mut results: Vec<SearchResult> = seen.into_values().collect();
    // sort DESC by date
    results.sort_by(|a, b| b.date.cmp(&a.date));
    results.truncate(limit as usize);

    Ok(results)
}
