//! Suggests auto mark-as-read when ≥5 messages from sender all have \\Seen - the user never leaves these unread, so pre-reading them on arrival makes sense.

use super::{count_messages_total, extract_email, rule_already_exists};
use crate::db::filters::{
    ActionType, ConditionField, ConditionOperator, FilterRule, MatchMode, RuleAction, RuleCondition,
};
use crate::error::DBError;
use rusqlite::{Connection, params};

const THRESHOLD: i64 = 5;

pub fn suggest(
    conn: &Connection,
    account_id: &str,
    from_addr: &str,
) -> Result<Option<FilterRule>, DBError> {
    let email = extract_email(from_addr);
    let pattern = format!("%{}%", email);

    let total = count_messages_total(conn, account_id, &pattern);
    if total < THRESHOLD {
        return Ok(None);
    }

    // If any message is unread, the user does actually read them selectively — no point.
    let unread: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM messages
             WHERE account_id = ? AND from_addr LIKE ?
               AND (flags_json IS NULL OR flags_json NOT LIKE '%\\\\Seen%')",
            params![account_id, pattern],
            |row| row.get(0),
        )
        .unwrap_or(0);

    if unread > 0 {
        return Ok(None);
    }

    if rule_already_exists(conn, account_id, email)? {
        return Ok(None);
    }

    Ok(Some(FilterRule {
        id: String::new(),
        account_id: account_id.to_string(),
        name: format!("Mark as read on arrival from {}", email),
        match_mode: MatchMode::All,
        conditions: vec![RuleCondition {
            field: ConditionField::From,
            operator: ConditionOperator::Contains,
            value: email.to_string(),
        }],
        actions: vec![RuleAction {
            action_type: ActionType::MarkRead,
            value: None,
        }],
        position: 0,
        enabled: false,
    }))
}
