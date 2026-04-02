//! Suggests moving a high-volume sender out of the inbox when ≥8 messages exist in inbox and none are starred (so the sender isn't considered important).
//! The rule is created with an empty move-to target - user must pick the folder before enabling.

use super::{count_messages_in_role, extract_email, rule_already_exists};
use crate::db::filters::{
    ActionType, ConditionField, ConditionOperator, FilterRule, MatchMode, RuleAction, RuleCondition,
};
use crate::error::DBError;
use rusqlite::{Connection, params};

const INBOX_THRESHOLD: i64 = 8;

pub fn suggest(
    conn: &Connection,
    account_id: &str,
    from_addr: &str,
) -> Result<Option<FilterRule>, DBError> {
    let email = extract_email(from_addr);
    let pattern = format!("%{}%", email);

    let inbox_count = count_messages_in_role(conn, account_id, &pattern, "inbox");
    if inbox_count < INBOX_THRESHOLD {
        return Ok(None);
    }

    // Don't suggest if the user has starred any message from this sender.
    let starred: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM messages WHERE account_id = ? AND from_addr LIKE ? AND starred = 1",
            params![account_id, pattern],
            |row| row.get(0),
        )
        .unwrap_or(0);

    if starred > 0 {
        return Ok(None);
    }

    if rule_already_exists(conn, account_id, email)? {
        return Ok(None);
    }

    Ok(Some(FilterRule {
        id: String::new(),
        account_id: account_id.to_string(),
        // Blank target — the user must choose a destination folder before enabling.
        name: format!("Move bulk mail from {} out of inbox", email),
        match_mode: MatchMode::All,
        conditions: vec![RuleCondition {
            field: ConditionField::From,
            operator: ConditionOperator::Contains,
            value: email.to_string(),
        }],
        actions: vec![RuleAction {
            action_type: ActionType::MoveTo,
            value: Some(String::new()),
        }],
        position: 0,
        enabled: false,
    }))
}
