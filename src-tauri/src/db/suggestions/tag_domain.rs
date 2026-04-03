//! Suggests tagging all mail from a non-generic domain when ≥5 messages already exist.
//! Skipped for common providers (Gmail, Outlook, etc.) since tagging by domain there would just tag everyone.

use super::{extract_domain, extract_email, is_generic_provider};
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
    let Some(domain) = extract_domain(email) else {
        return Ok(None);
    };

    if is_generic_provider(domain) {
        return Ok(None);
    }

    let pattern = format!("%@{}%", domain);
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM messages WHERE account_id = ? AND from_addr LIKE ?",
            params![account_id, pattern],
            |row| row.get(0),
        )
        .unwrap_or(0);

    if count < THRESHOLD {
        return Ok(None);
    }

    // Check no existing rule already targets this domain.
    let rules = crate::db::filters::get_rules(conn, account_id)?;
    let domain_lower = domain.to_lowercase();
    let already = rules.iter().any(|r| {
        r.conditions.iter().any(|c| {
            matches!(c.field, ConditionField::From)
                && c.value.to_lowercase().contains(&domain_lower)
        })
    });
    if already {
        return Ok(None);
    }

    // Tag = first label of domain, e.g. "github" from "github.com".
    let tag = domain.split('.').next().unwrap_or(domain).to_lowercase();

    Ok(Some(FilterRule {
        id: String::new(),
        account_id: account_id.to_string(),
        name: format!("Tag mail from @{}", domain),
        match_mode: MatchMode::All,
        conditions: vec![RuleCondition {
            field: ConditionField::From,
            operator: ConditionOperator::Contains,
            value: format!("@{}", domain),
        }],
        actions: vec![RuleAction {
            action_type: ActionType::AddTag,
            value: Some(tag),
        }],
        position: 0,
        enabled: false,
    }))
}
