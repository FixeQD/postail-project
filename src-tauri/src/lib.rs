pub mod build_info;
pub mod cmd;
pub mod db;
pub mod email_view;
pub mod error;
pub mod globals;
pub mod imap;
pub mod maintenance;
pub mod network;
pub mod oauth;
pub mod protocol;
pub mod security;
pub mod smtp;
pub mod utils;
pub mod webview_policy;

use std::sync::atomic::Ordering;

use crate::globals::SMTP_MANAGER;
use crate::imap::pool::init_pool;
use crate::imap::sync_status::set_sync_status_app_handle;
use crate::network::cache::{RESOURCE_CACHE, ResourceCache};
use tauri::Manager;

/// TPM helper mode: Initialize TPM with elevated privileges (Linux only)
#[cfg(all(target_os = "linux", feature = "tpm"))]
pub fn tpm_helper_init() -> Result<(), String> {
    crate::security::tpm_helper_init()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt()
        .with_target(false)
        .with_level(true)
        .without_time()
        .with_line_number(false)
        .with_file(false)
        .with_env_filter(tracing_subscriber::EnvFilter::new("info,tss_esapi=warn"))
        .init();

    tauri::Builder::default()
        .manage(email_view::EmailViewState::default())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .setup(|app| {
            let handle = app.handle().clone();

            // Initialize resource cache
            {
                let cache_dir = app
                    .path()
                    .app_data_dir()
                    .expect("failed to resolve app data dir")
                    .join("resource_cache");
                RESOURCE_CACHE
                    .set(ResourceCache::new(cache_dir))
                    .unwrap_or_else(|_| tracing::warn!("resource cache already initialized"));
            }

            // System tray
            {
                use tauri::menu::{MenuBuilder, MenuItemBuilder};
                use tauri::tray::TrayIconBuilder;

                let show = MenuItemBuilder::with_id("show", "Open Postail").build(app)?;
                let quit = MenuItemBuilder::with_id("quit", "Quit").build(app)?;
                let menu = MenuBuilder::new(app).items(&[&show, &quit]).build()?;

                let tray_handle: tauri::AppHandle = handle.clone();
                TrayIconBuilder::new()
                    .icon(app.default_window_icon().unwrap().clone())
                    .menu(&menu)
                    .tooltip("Postail")
                    .on_menu_event(move |_tray, event| match event.id.as_ref() {
                        "show" => {
                            if let Some(w) = tray_handle.get_webview_window("main") {
                                let _ = w.show();
                                let _ = w.set_focus();
                            }
                        }
                        "quit" => tray_handle.exit(0),
                        _ => {}
                    })
                    .build(app)?;
            }

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

            // Start null proxy — accepts connections, always responds 502.
            #[cfg(target_os = "windows")]
            let proxy_port: u16 = 18731;
            #[cfg(not(target_os = "windows"))]
            let proxy_port: u16 = portpicker::pick_unused_port().unwrap_or(18731);

            tauri::async_runtime::spawn(async move {
                crate::network::null_proxy::start_on_port(proxy_port).await;
            });

            if let Some(main_window) = app.get_webview_window("main") {
                webview_policy::install_network_block(&main_window, proxy_port);
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            cmd::security::get_app_initialization_status,
            cmd::utils::greet,
            cmd::account::start_oauth_flow,
            cmd::account::complete_oauth_flow,
            cmd::account::complete_reauth_flow,
            cmd::account::add_account,
            cmd::account::add_custom_account,
            cmd::account::update_account_name,
            cmd::account::update_custom_account,
            cmd::account::list_accounts,
            cmd::account::remove_account,
            cmd::account::get_available_providers,
            cmd::security::check_security_options,
            #[cfg(all(target_os = "linux", feature = "tpm"))]
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
            cmd::security::change_security_method,
            cmd::security::reset_security_setup,
            cmd::security::generate_recovery_phrase,
            cmd::security::unlock_with_recovery_phrase,
            cmd::security::verify_recovery_words,
            cmd::mail::listing::fetch_mailboxes,
            cmd::mail::listing::update_mailbox_role,
            cmd::mail::folders::create_folder,
            cmd::mail::folders::create_subfolder,
            cmd::mail::folders::rename_folder,
            cmd::mail::folders::delete_folder,
            cmd::mail::folders::set_folder_hidden,
            cmd::mail::folders::move_messages,
            cmd::mail::folders::archive_messages,
            cmd::mail::folders::subscribe_folder,
            cmd::mail::folders::unsubscribe_folder,
            cmd::mail::listing::fetch_headers,
            cmd::mail::listing::fetch_message_full,
            cmd::mail::listing::save_attachment,
            cmd::mail::listing::add_message_tag,
            cmd::mail::listing::remove_message_tag,
            cmd::mail::listing::get_account_tags,
            cmd::mail::listing::get_tag_colors,
            cmd::mail::listing::set_tag_color,
            cmd::mail::listing::rename_tag,
            cmd::mail::listing::delete_tag,
            cmd::filters::get_filter_rules,
            cmd::filters::save_filter_rule,
            cmd::filters::delete_filter_rule,
            cmd::filters::reorder_filter_rules,
            cmd::filters::apply_filters_to_mailbox,
            cmd::filters::suggest_rules_for_sender,
            cmd::mail::sync::start_sync,
            cmd::mail::sync::get_inbox_baseline_uids,
            cmd::mail::sync::stop_sync,
            cmd::mail::sync::get_sync_status,
            cmd::mail::sync::sync_mailbox_list,
            cmd::mail::sync::sync_single_mailbox,
            cmd::mail::sync::watch_mailbox,
            cmd::mail::sync::unwatch_mailbox,
            cmd::mail::sync::unwatch_all_mailboxes,
            cmd::maintenance::clear_cache,
            cmd::maintenance::backfill_snippets,
            cmd::settings::get_autostart_enabled,
            cmd::settings::set_autostart_enabled,
            cmd::maintenance::dev_reset_data,
            cmd::mail::sync::record_mailbox_activity,
            cmd::mail::actions::search_messages,
            cmd::mail::actions::search_messages_advanced,
            cmd::mail::actions::imap_search_messages,
            cmd::mail::actions::mark_read,
            cmd::mail::listing::fetch_raw_eml_text,
            cmd::mail::listing::fetch_thread,
            cmd::mail::actions::delete_messages,
            cmd::mail::actions::toggle_starred,
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
            cmd::contacts::list_contacts,
            cmd::contacts::search_contacts_full,
            cmd::contacts::get_contact_messages,
            cmd::contacts::update_contact,
            cmd::attachments::add_attachment,
            cmd::attachments::add_attachment_bytes,
            cmd::attachments::add_inline_attachment,
            cmd::attachments::add_inline_attachment_path,
            cmd::attachments::remove_attachment,
            cmd::smtp::build_email_from_draft,
            cmd::smtp::send_read_receipt,
            cmd::utils::process_email_content,
            cmd::utils::show_notification,
            cmd::utils::show_main_window,
            cmd::utils::auto_fix_email_html,
            cmd::utils::set_email_view_content,
            cmd::settings::get_all_settings,
            cmd::settings::get_setting,
            cmd::settings::set_setting,
            cmd::settings::migrate_data_path,
            cmd::settings::set_initial_data_dir,
            cmd::settings::get_default_data_dir,
            cmd::settings::get_theme_config,
            cmd::settings::set_theme_config,
            cmd::settings::get_build_info,
            cmd::network::clear_resource_cache,
            cmd::network::get_resource_cache_stats,
            cmd::search::get_saved_searches,
            cmd::search::create_saved_search,
            cmd::search::delete_saved_search,
            cmd::signatures::list_signatures,
            cmd::signatures::save_signature,
            cmd::signatures::delete_signature,
            cmd::signatures::get_default_signature,
            cmd::templates::list_templates,
            cmd::templates::save_template,
            cmd::templates::delete_template
        ])
        .register_uri_scheme_protocol("postail", protocol::handler)
        .build(tauri::generate_context!())
        .expect("error while running tauri application")
        .run(|app_handle, event| {
            if let tauri::RunEvent::WindowEvent {
                label,
                event: tauri::WindowEvent::CloseRequested { api, .. },
                ..
            } = event
            {
                if label == "main" && globals::MINIMIZE_TO_TRAY.load(Ordering::SeqCst) {
                    api.prevent_close();
                    if let Some(w) = app_handle.get_webview_window("main") {
                        let _ = w.hide();
                    }
                }
            }
        });
}
