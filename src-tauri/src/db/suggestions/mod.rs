mod archive_sender;
mod delete_sender;
mod mark_read_sender;
mod tag_domain;

use crate::db::filters::{ConditionField, FilterRule};
use crate::error::DBError;
use rusqlite::{Connection, params};

pub fn suggest_rules_for_sender(
    conn: &Connection,
    account_id: &str,
    from_addr: &str,
) -> Result<Vec<FilterRule>, DBError> {
    let mut suggestions = Vec::new();

    macro_rules! try_suggest {
        ($module:ident) => {
            if let Some(rule) = $module::suggest(conn, account_id, from_addr)? {
                suggestions.push(rule);
            }
        };
    }

    try_suggest!(delete_sender);
    try_suggest!(mark_read_sender);
    try_suggest!(tag_domain);
    try_suggest!(archive_sender);

    Ok(suggestions)
}

// ── Shared helpers ────────────────────────────────────────────────────────────

/// Extract the raw email address from "Display Name <email>" or plain "email".
pub(crate) fn extract_email(from_addr: &str) -> &str {
    if let Some(start) = from_addr.rfind('<') {
        if let Some(end) = from_addr[start..].find('>') {
            return from_addr[start + 1..start + end].trim();
        }
    }
    from_addr.trim()
}

/// Extract domain part from "user@domain.com" → "domain.com".
pub(crate) fn extract_domain(email: &str) -> Option<&str> {
    email.split('@').nth(1).map(str::trim)
}

const GENERIC_PROVIDERS: &[&str] = &[
    "gmail.com",
    "googlemail.com",
    "yahoo.com",
    "yahoo.co.uk",
    "hotmail.com",
    "hotmail.co.uk",
    "outlook.com",
    "live.com",
    "icloud.com",
    "me.com",
    "mac.com",
    "protonmail.com",
    "proton.me",
    "tutanota.com",
    "aol.com",
    "gmx.com",
    "gmx.net",
];

pub(crate) fn is_generic_provider(domain: &str) -> bool {
    let lower = domain.to_lowercase();
    GENERIC_PROVIDERS.contains(&lower.as_str())
}

/// Returns true if any existing rule already has a From condition that matches `value`.
pub(crate) fn rule_already_exists(
    conn: &Connection,
    account_id: &str,
    value: &str,
) -> Result<bool, DBError> {
    let rules = crate::db::filters::get_rules(conn, account_id)?;
    let value_lower = value.to_lowercase();
    Ok(rules.iter().any(|r| {
        r.conditions.iter().any(|c| {
            matches!(c.field, ConditionField::From) && c.value.to_lowercase() == value_lower
        })
    }))
}

/// Count messages from a sender (LIKE pattern) in mailboxes with a given role.
pub(crate) fn count_messages_in_role(
    conn: &Connection,
    account_id: &str,
    from_pattern: &str,
    role: &str,
) -> i64 {
    conn.query_row(
        "SELECT COUNT(*) FROM messages m
         JOIN mailboxes mb ON mb.account_id = m.account_id AND mb.name = m.mailbox
         WHERE m.account_id = ? AND m.from_addr LIKE ? AND mb.role = ?",
        params![account_id, from_pattern, role],
        |row| row.get(0),
    )
    .unwrap_or(0)
}

/// Count all messages from a sender (LIKE pattern).
pub(crate) fn count_messages_total(
    conn: &Connection,
    account_id: &str,
    from_pattern: &str,
) -> i64 {
    conn.query_row(
        "SELECT COUNT(*) FROM messages WHERE account_id = ? AND from_addr LIKE ?",
        params![account_id, from_pattern],
        |row| row.get(0),
    )
    .unwrap_or(0)
}
