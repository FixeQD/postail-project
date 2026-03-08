use crate::db::OutboxItem;
use crate::globals::{DB_CONN, SMTP_MANAGER};
use std::sync::Arc;
use tauri::command;

#[command]
pub async fn send_read_receipt(
    account_id: String,
    to_address: String,
    original_message_id: Option<String>,
    original_subject: Option<String>,
) -> Result<(), String> {
    let from_email = {
        let conn_guard = DB_CONN.lock().await;
        let conn = conn_guard.as_ref().ok_or("Database not initialized")?;
        let mut stmt = conn
            .prepare("SELECT email FROM accounts WHERE id = ?")
            .map_err(|e| e.to_string())?;
        let email: String = stmt
            .query_row([&account_id], |row| row.get(0))
            .map_err(|e| e.to_string())?;
        email
    };

    let subject = original_subject.as_deref().unwrap_or("(no subject)");
    let eml = crate::smtp::mdn::build_mdn(
        &from_email,
        &to_address,
        subject,
        original_message_id.as_deref(),
    );

    tracing::info!(
        target: "postail",
        "[MDN] Sending read receipt to={} for message_id={:?}",
        to_address,
        original_message_id
    );

    let smtp = SMTP_MANAGER.lock().await;
    smtp.send_email(&account_id, &eml).await.map_err(|e| {
        tracing::error!(target: "postail", "[MDN] Failed to send read receipt: {}", e);
        e
    })
}

#[command]
pub async fn enqueue_message(account_id: String, raw_eml: Vec<u8>) -> Result<String, String> {
    let smtp = SMTP_MANAGER.lock().await;
    smtp.enqueue_message(&account_id, &raw_eml).await
}

#[command]
pub async fn list_outbox(account_id: String) -> Result<Vec<OutboxItem>, String> {
    let smtp = SMTP_MANAGER.lock().await;
    smtp.list_outbox(&account_id).await
}

#[command]
pub async fn retry_sending(outbox_id: String) -> Result<(), String> {
    let smtp = SMTP_MANAGER.lock().await;
    smtp.retry_sending(&outbox_id).await
}

#[command]
pub async fn cancel_sending(outbox_id: String) -> Result<(), String> {
    let smtp = SMTP_MANAGER.lock().await;
    smtp.cancel_sending(&outbox_id).await
}

#[derive(serde::Serialize)]
pub struct BuildEmailResult {
    pub eml_bytes: Vec<u8>,
    pub html_with_cids: String,
}

#[command]
pub async fn build_email_from_draft(
    draft_id: String,
    request_read_receipt: Option<bool>,
) -> Result<BuildEmailResult, String> {
    tracing::info!(target: "postail", "[build_email_from_draft] Starting for draft_id={} request_read_receipt={:?}", draft_id, request_read_receipt);
    let db_conn = Arc::clone(&DB_CONN);

    tokio::task::spawn_blocking(move || {
        let conn_guard = db_conn.blocking_lock();
        let conn = conn_guard.as_ref().ok_or("Database not initialized")?;

        let draft = crate::db::load_draft(conn, &draft_id)
            .map_err(|e| e.to_string())?
            .ok_or("Draft not found")?;

        let from_email: String = {
            let mut stmt = conn
                .prepare("SELECT email FROM accounts WHERE id = ?")
                .map_err(|e| e.to_string())?;
            stmt.query_row([&draft.account_id], |row| row.get(0))
                .map_err(|e| e.to_string())?
        };

        let html_body = draft.body.unwrap_or_default();
        let html_with_cids =
            crate::smtp::mime_builder::replace_asset_urls_with_cids(&html_body, &draft.attachments);

        let to: Vec<&str> = draft.to.iter().map(|s| s.as_str()).collect();
        let cc: Vec<&str> = draft.cc.iter().map(|s| s.as_str()).collect();
        let bcc: Vec<&str> = draft.bcc.iter().map(|s| s.as_str()).collect();
        let subject = draft.subject.unwrap_or_default();

        tracing::info!(target: "postail", "[build_email_from_draft] Building email with {} to, {} cc, {} bcc recipients, subject='{}'",
            to.len(), cc.len(), bcc.len(), subject);

        let disposition_notification_to = if request_read_receipt.unwrap_or(false) {
            Some(from_email.clone())
        } else {
            None
        };

        let eml_bytes = crate::smtp::mime_builder::build_multipart_email(
            &from_email,
            to,
            cc,
            bcc,
            &subject,
            &html_with_cids,
            &draft.attachments,
            disposition_notification_to.as_deref(),
        )
        .map_err(|e| e.to_string())?;

        Ok(BuildEmailResult {
            eml_bytes,
            html_with_cids,
        })
    })
    .await
    .map_err(|e| e.to_string())?
}
