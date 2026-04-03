use crate::error::DBError;
use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConditionField {
    From,
    To,
    Subject,
    Body,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConditionOperator {
    Contains,
    NotContains,
    Equals,
    NotEquals,
    StartsWith,
    EndsWith,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleCondition {
    pub field: ConditionField,
    pub operator: ConditionOperator,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionType {
    MoveTo,   // value = folder name
    AddTag,   // value = tag name
    Star,     // no value
    MarkRead, // no value
    Delete,   // no value
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleAction {
    pub action_type: ActionType,
    pub value: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MatchMode {
    All, // AND
    Any, // OR
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterRule {
    pub id: String,
    pub account_id: String,
    pub name: String,
    pub match_mode: MatchMode,
    pub conditions: Vec<RuleCondition>,
    pub actions: Vec<RuleAction>,
    pub position: i32,
    pub enabled: bool,
}

pub fn get_rules(conn: &Connection, account_id: &str) -> Result<Vec<FilterRule>, DBError> {
    let mut stmt = conn.prepare(
        "SELECT id, account_id, name, match_mode, conditions_json, actions_json, position, enabled
         FROM filter_rules WHERE account_id = ? ORDER BY position ASC",
    )?;
    let rules_iter = stmt.query_map(params![account_id], |row| {
        let conditions_json: String = row.get(4)?;
        let actions_json: String = row.get(5)?;
        let match_mode_str: String = row.get(3)?;

        Ok(FilterRule {
            id: row.get(0)?,
            account_id: row.get(1)?,
            name: row.get(2)?,
            match_mode: match match_mode_str.as_str() {
                "any" => MatchMode::Any,
                _ => MatchMode::All,
            },
            conditions: serde_json::from_str(&conditions_json).unwrap_or_default(),
            actions: serde_json::from_str(&actions_json).unwrap_or_default(),
            position: row.get(6)?,
            enabled: row.get::<_, i32>(7)? != 0,
        })
    })?;

    let mut rules = Vec::new();
    for rule in rules_iter {
        rules.push(rule?);
    }
    Ok(rules)
}

pub fn save_rule(conn: &Connection, rule: &FilterRule) -> Result<(), DBError> {
    conn.execute(
        "INSERT OR REPLACE INTO filter_rules (id, account_id, name, match_mode, conditions_json, actions_json, position, enabled)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        params![
            rule.id,
            rule.account_id,
            rule.name,
            match rule.match_mode {
                MatchMode::Any => "any",
                MatchMode::All => "all",
            },
            serde_json::to_string(&rule.conditions).unwrap_or_default(),
            serde_json::to_string(&rule.actions).unwrap_or_default(),
            rule.position,
            if rule.enabled { 1 } else { 0 },
        ],
    )?;
    Ok(())
}

pub fn delete_rule(conn: &Connection, rule_id: &str, account_id: &str) -> Result<(), DBError> {
    conn.execute(
        "DELETE FROM filter_rules WHERE id = ? AND account_id = ?",
        params![rule_id, account_id],
    )?;
    Ok(())
}

pub fn reorder_rules(
    conn: &Connection,
    account_id: &str,
    ordered_ids: &[String],
) -> Result<(), DBError> {
    for (idx, id) in ordered_ids.iter().enumerate() {
        conn.execute(
            "UPDATE filter_rules SET position = ? WHERE id = ? AND account_id = ?",
            params![idx as i32, id, account_id],
        )?;
    }
    Ok(())
}

struct MessageMatchData {
    pub from_addr: String,
    pub to_list: Vec<String>,
    pub subject: String,
    pub body_plain: String,
}

pub fn apply_rules_to_message(
    conn: &mut Connection,
    account_id: &str,
    mailbox: &str,
    uid: u32,
) -> Result<(), DBError> {
    apply_rules_to_messages(conn, account_id, mailbox, &[uid])
}

pub fn apply_rules_to_messages(
    conn: &mut Connection,
    account_id: &str,
    mailbox: &str,
    uids: &[u32],
) -> Result<(), DBError> {
    if uids.is_empty() {
        return Ok(());
    }

    let rules = get_rules(conn, account_id)?;
    let enabled_rules: Vec<FilterRule> = rules.into_iter().filter(|r| r.enabled).collect();

    if enabled_rules.is_empty() {
        return Ok(());
    }

    for &uid in uids {
        let msg_data = conn
            .query_row(
                "SELECT from_addr, to_json, subject, mb.body_plain
             FROM messages m
             LEFT JOIN message_bodies mb ON mb.message_id = m.id
             WHERE m.account_id = ? AND m.mailbox = ? AND m.uid = ?",
                params![account_id, mailbox, uid],
                |row| {
                    let to_json: Option<String> = row.get(1)?;
                    let to_list: Vec<String> = to_json
                        .and_then(|s| serde_json::from_str(&s).ok())
                        .unwrap_or_default();

                    Ok(MessageMatchData {
                        from_addr: row.get::<_, Option<String>>(0)?.unwrap_or_default(),
                        to_list,
                        subject: row.get::<_, Option<String>>(2)?.unwrap_or_default(),
                        body_plain: row.get::<_, Option<String>>(3)?.unwrap_or_default(),
                    })
                },
            )
            .optional()?;

        let Some(msg) = msg_data else {
            continue;
        };

        for rule in &enabled_rules {
            let is_match = match rule.match_mode {
                MatchMode::All => rule.conditions.iter().all(|c| matches_condition(&msg, c)),
                MatchMode::Any => rule.conditions.iter().any(|c| matches_condition(&msg, c)),
            };

            if is_match {
                tracing::info!(target: "postail", "[Filters] Rule \"{}\" matched message UID={} in {}", rule.name, uid, mailbox);
                let (moves, non_moves): (Vec<_>, Vec<_>) = rule
                    .actions
                    .clone()
                    .into_iter()
                    .partition(|a| matches!(a.action_type, ActionType::MoveTo));
                for action in non_moves {
                    execute_action(conn, account_id, mailbox, uid, &action)?;
                }
                for action in moves {
                    execute_action(conn, account_id, mailbox, uid, &action)?;
                }
                // First-match semantics: stop after first matching rule
                break;
            }
        }
    }

    Ok(())
}

fn matches_condition(msg: &MessageMatchData, cond: &RuleCondition) -> bool {
    let needle = cond.value.to_lowercase();
    let operator = &cond.operator;

    let check_single = |haystack: &str| -> bool {
        let haystack_lower = haystack.to_lowercase();
        match operator {
            ConditionOperator::Contains => haystack_lower.contains(&needle),
            ConditionOperator::NotContains => !haystack_lower.contains(&needle),
            ConditionOperator::Equals => haystack_lower == needle,
            ConditionOperator::NotEquals => haystack_lower != needle,
            ConditionOperator::StartsWith => haystack_lower.starts_with(&needle),
            ConditionOperator::EndsWith => haystack_lower.ends_with(&needle),
        }
    };

    match cond.field {
        ConditionField::From => check_single(&msg.from_addr),
        ConditionField::Subject => check_single(&msg.subject),
        ConditionField::Body => check_single(&msg.body_plain),
        ConditionField::To => {
            if msg.to_list.is_empty() {
                return match operator {
                    ConditionOperator::NotContains | ConditionOperator::NotEquals => true,
                    _ => false,
                };
            }
            match operator {
                ConditionOperator::NotContains | ConditionOperator::NotEquals => {
                    msg.to_list.iter().all(|h| check_single(h))
                }
                _ => msg.to_list.iter().any(|h| check_single(h)),
            }
        }
    }
}

fn execute_action(
    conn: &Connection,
    account_id: &str,
    mailbox: &str,
    uid: u32,
    action: &RuleAction,
) -> Result<(), DBError> {
    match action.action_type {
        ActionType::MarkRead => {
            crate::db::mail::messages::mark_read(conn, account_id, mailbox, &[uid], true)?;
            crate::db::mail::flag_queue::enqueue_flag_change(
                conn,
                account_id,
                mailbox,
                uid,
                "add",
                &[String::from("\\Seen")],
            )?;
        }
        ActionType::Star => {
            crate::db::mail::messages::set_starred(conn, account_id, mailbox, uid, true)?;
            crate::db::mail::flag_queue::enqueue_flag_change(
                conn,
                account_id,
                mailbox,
                uid,
                "add",
                &[String::from("\\Flagged")],
            )?;
        }
        ActionType::AddTag => {
            if let Some(tag) = &action.value {
                let normalized_tag = tag.trim().replace(' ', "_");
                if !normalized_tag.is_empty() {
                    crate::db::mail::messages::add_tag(
                        conn,
                        account_id,
                        mailbox,
                        uid,
                        &normalized_tag,
                    )?;
                    crate::db::mail::flag_queue::enqueue_flag_change(
                        conn,
                        account_id,
                        mailbox,
                        uid,
                        "add",
                        &[normalized_tag],
                    )?;
                }
            }
        }
        ActionType::Delete => {
            crate::db::mail::messages::move_to_trash(conn, account_id, mailbox, &[uid])?;
        }
        ActionType::MoveTo => {
            if let Some(target) = &action.value {
                if target.trim().is_empty() {
                    return Ok(());
                }
                conn.execute(
                    "UPDATE messages SET mailbox = ? WHERE account_id = ? AND mailbox = ? AND uid = ?",
                    params![target, account_id, mailbox, uid],
                )?;

                crate::db::mail::flag_queue::enqueue_move_operation(
                    conn, account_id, mailbox, target, uid,
                )?;
            }
        }
    }
    Ok(())
}

pub fn apply_rules_to_mailbox(
    conn: &mut Connection,
    account_id: &str,
    mailbox: &str,
) -> Result<u32, DBError> {
    let uids: Vec<u32> = {
        let mut stmt =
            conn.prepare("SELECT uid FROM messages WHERE account_id = ? AND mailbox = ?")?;
        let uids_iter = stmt.query_map(params![account_id, mailbox], |row| row.get::<_, u32>(0))?;
        let mut collected = Vec::new();
        for uid_res in uids_iter {
            collected.push(uid_res?);
        }
        collected
    };

    if let Err(e) = apply_rules_to_messages(conn, account_id, mailbox, &uids) {
        tracing::error!(target: "postail", "[Filters] Error applying rules to mailbox {}: {}", mailbox, e);
        return Err(e);
    }

    Ok(uids.len() as u32)
}
