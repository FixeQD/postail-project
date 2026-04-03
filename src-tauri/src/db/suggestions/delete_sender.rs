//! Suggests auto-delete when user has already deleted ≥2 messages from this sender.

use crate::db::filters::{
    ActionType, ConditionField, ConditionOperator, FilterRule, MatchMode, RuleAction,
    RuleCondition,
};
use crate::error::DBError;
use rusqlite::Connection;
use super::{count_messages_in_role, extract_email, rule_already_exists};

/// Minimum messages already in trash from this sender to trigger the suggestion.
const TRASH_THRESHOLD: i64 = 2;

pub fn suggest(
    conn: &Connection,
    account_id: &str,
    from_addr: &str,
) -> Result<Option<FilterRule>, DBError> {
    let email = extract_email(from_addr);
    let pattern = format!("%{}%", email);

    let trash_count = count_messages_in_role(conn, account_id, &pattern, "trash");
    if trash_count < TRASH_THRESHOLD {
        return Ok(None);
    }

    if rule_already_exists(conn, account_id, email)? {
        return Ok(None);
    }

    Ok(Some(FilterRule {
        id: String::new(),
        account_id: account_id.to_string(),
        name: format!("Delete messages from {}", email),
        match_mode: MatchMode::All,
        conditions: vec![RuleCondition {
            field: ConditionField::From,
            operator: ConditionOperator::Contains,
            value: email.to_string(),
        }],
        actions: vec![RuleAction {
            action_type: ActionType::Delete,
            value: None,
        }],
        position: 0,
        enabled: false,
    }))
}
