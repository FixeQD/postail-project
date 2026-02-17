pub mod cmd;
pub mod db;
pub mod error;
pub mod globals;
pub mod imap;
pub mod maintenance;
pub mod oauth;
pub mod protocol;
pub mod security;
pub mod smtp;
pub mod utils;

use crate::globals::SMTP_MANAGER;
use crate::imap::pool::init_pool;
use crate::imap::sync_status::set_sync_status_app_handle;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt()
        .with_target(false)
        .with_level(true)
        .without_time()
        .with_line_number(false)
        .with_file(false)
        .init();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let handle = app.handle().clone();

            // Initialize managers in async context
            let setup_handle = handle.clone();
            tauri::async_runtime::spawn(async move {
                let smtp = SMTP_MANAGER.lock().await;
                smtp.set_app_handle(setup_handle.clone()).await;
                set_sync_status_app_handle(setup_handle);
            });

            // Start auto-lock timer
            let timer_handle = handle.clone();
            tauri::async_runtime::spawn(async move {
                crate::security::start_lock_timer(timer_handle).await;
            });

            // Initialize IMAP connection pool
            tauri::async_runtime::spawn(async move {
                init_pool().await;
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            cmd::security::get_app_initialization_status,
            cmd::utils::greet,
            cmd::account::start_oauth_flow,
            cmd::account::complete_oauth_flow,
            cmd::account::add_account,
            cmd::account::add_custom_account,
            cmd::account::list_accounts,
            cmd::account::remove_account,
            cmd::account::get_available_providers,
            cmd::security::check_security_options,
            cmd::security::check_tpm_availability,
            cmd::security::initialize_security,
            cmd::security::record_lock_activity,
            cmd::security::is_app_locked,
            cmd::security::unlock_app,
            cmd::security::set_auto_lock_timeout,
            cmd::security::get_auto_lock_timeout,
            cmd::security::set_auto_lock_pin,
            cmd::security::use_encryption_password_for_lock,
            cmd::security::is_lock_using_encryption_password,
            cmd::security::is_lock_configured,
            cmd::security::get_security_method,
            cmd::security::generate_recovery_phrase,
            cmd::security::unlock_with_recovery_phrase,
            cmd::security::verify_recovery_words,
            cmd::mail::listing::fetch_mailboxes,
            cmd::mail::listing::fetch_headers,
            cmd::mail::listing::fetch_message_full,
            cmd::mail::sync::start_sync,
            cmd::mail::sync::stop_sync,
            cmd::mail::sync::get_sync_status,
            cmd::mail::sync::sync_mailbox_list,
            cmd::mail::sync::sync_single_mailbox,
            cmd::mail::sync::watch_mailbox,
            cmd::mail::sync::unwatch_mailbox,
            cmd::mail::sync::unwatch_all_mailboxes,
            cmd::mail::sync::record_mailbox_activity,
            cmd::mail::actions::search_messages,
            cmd::mail::actions::mark_read,
            cmd::mail::actions::delete_messages,
            cmd::smtp::enqueue_message,
            cmd::smtp::list_outbox,
            cmd::smtp::retry_sending,
            cmd::smtp::cancel_sending,
            cmd::maintenance::export_backup,
            cmd::maintenance::import_backup,
            cmd::maintenance::run_maintenance,
            cmd::drafts::save_draft,
            cmd::drafts::list_drafts,
            cmd::drafts::delete_draft,
            cmd::maintenance::search_contacts,
            cmd::attachments::add_attachment,
            cmd::attachments::add_attachment_bytes,
            cmd::attachments::add_inline_attachment,
            cmd::attachments::remove_attachment,
            cmd::smtp::build_email_from_draft,
            cmd::utils::process_email_content,
            cmd::utils::auto_fix_email_html,
            cmd::settings::get_all_settings,
            cmd::settings::get_setting,
            cmd::settings::set_setting,
            cmd::settings::migrate_data_path,
            cmd::settings::get_default_data_dir,
            cmd::settings::get_theme_config,
            cmd::settings::set_theme_config
        ])
        .register_uri_scheme_protocol("postail", protocol::handler)
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
