use crate::db::filters::{self, FilterRule};
use crate::globals::get_db_pool;
use tauri::command;
use uuid::Uuid;

#[command]
pub async fn get_filter_rules(account_id: String) -> Result<Vec<FilterRule>, String> {
    let pool = get_db_pool().await.map_err(|e| e.to_string())?;
    let conn = pool.get().map_err(|e| e.to_string())?;
    filters::get_rules(&conn, &account_id).map_err(|e| e.to_string())
}

#[command]
pub async fn save_filter_rule(mut rule: FilterRule) -> Result<(), String> {
    let pool = get_db_pool().await.map_err(|e| e.to_string())?;
    let conn = pool.get().map_err(|e| e.to_string())?;

    if rule.id.is_empty() {
        rule.id = Uuid::new_v4().to_string();

        // For new rules, set position to be the LAST
        let max_pos: i32 = conn
            .query_row(
                "SELECT MAX(position) FROM filter_rules WHERE account_id = ?",
                [&rule.account_id],
                |row| row.get::<_, Option<i32>>(0),
            )
            .map_err(|e| e.to_string())?
            .unwrap_or(-1);

        rule.position = max_pos + 1;
    }

    filters::save_rule(&conn, &rule).map_err(|e| e.to_string())
}

#[command]
pub async fn delete_filter_rule(rule_id: String, account_id: String) -> Result<(), String> {
    let pool = get_db_pool().await.map_err(|e| e.to_string())?;
    let conn = pool.get().map_err(|e| e.to_string())?;
    filters::delete_rule(&conn, &rule_id, &account_id).map_err(|e| e.to_string())
}

#[command]
pub async fn reorder_filter_rules(
    account_id: String,
    ordered_ids: Vec<String>,
) -> Result<(), String> {
    let pool = get_db_pool().await.map_err(|e| e.to_string())?;
    let conn = pool.get().map_err(|e| e.to_string())?;
    filters::reorder_rules(&conn, &account_id, &ordered_ids).map_err(|e| e.to_string())
}

#[command]
pub async fn apply_filters_to_mailbox(account_id: String, mailbox: String) -> Result<u32, String> {
    let pool = get_db_pool().await.map_err(|e| e.to_string())?;
    let mut conn = pool.get().map_err(|e| e.to_string())?;
    let count = filters::apply_rules_to_mailbox(&mut *conn, &account_id, &mailbox)
        .map_err(|e| e.to_string())?;

    let aid = account_id.clone();
    tauri::async_runtime::spawn(async move {
        if let Err(e) = crate::cmd::mail::actions::process_flag_queue(&aid).await {
            tracing::error!(target: "postail", "[Filters] Failed to process flag queue after manual apply: {}", e);
        }
    });

    Ok(count)
}

#[command]
pub async fn suggest_rules_for_sender(
    account_id: String,
    from_addr: String,
) -> Result<Vec<FilterRule>, String> {
    if from_addr.trim().is_empty() {
        return Ok(vec![]);
    }
    let pool = get_db_pool().await.map_err(|e| e.to_string())?;
    let conn = pool.get().map_err(|e| e.to_string())?;
    crate::db::suggestions::suggest_rules_for_sender(&conn, &account_id, &from_addr)
        .map_err(|e| e.to_string())
}
