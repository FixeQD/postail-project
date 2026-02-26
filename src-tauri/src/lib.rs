pub mod build_info;
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

/// TPM helper mode: Initialize TPM with elevated privileges (Linux only)
#[cfg(all(target_os = "linux", feature = "tpm"))]
pub fn tpm_helper_init() -> Result<(), String> {
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    use std::path::PathBuf;
    use tokio::net::{UnixListener, UnixStream};

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| e.to_string())?;

    rt.block_on(async {
        // Use PKEXEC_UID set by pkexec is more secure than USER env var
        let pkexec_uid_str = std::env::var("PKEXEC_UID")
            .map_err(|_| "PKEXEC_UID env var not set (Helper must be run via pkexec)".to_string())?;
        let uid_raw: u32 = pkexec_uid_str.parse().map_err(|_| "Invalid PKEXEC_UID".to_string())?;
        let uid = nix::unistd::Uid::from_raw(uid_raw);

        let user = nix::unistd::User::from_uid(uid)
            .map_err(|e| format!("Failed to find user with UID {}: {}", uid_raw, e))?
            .ok_or_else(|| format!("User with UID {} not found", uid_raw))?;
        let gid = user.gid;

        let socket_dir = PathBuf::from(format!("/run/user/{}", uid));
        
        // Ensure the directory exists
        if !socket_dir.exists() {
             fs::create_dir_all(&socket_dir).map_err(|e| format!("Failed to create socket dir: {}", e))?;
             let _ = nix::unistd::chown(&socket_dir, Some(uid), Some(gid));
        }
        
        let socket_path = socket_dir.join("postail-tpm.sock");

        // Clean up old socket
        let _ = fs::remove_file(&socket_path);

        let listener = UnixListener::bind(&socket_path)
            .map_err(|e| format!("Failed to bind socket at {:?}: {}", socket_path, e))?;

        // Set socket permissions so only the user can connect
        let _ = fs::set_permissions(&socket_path, fs::Permissions::from_mode(0o600));
        let _ = nix::unistd::chown(&socket_path, Some(uid), Some(gid));

        eprintln!("TPM Proxy Helper (UID: {}) listening on {:?}", uid_raw, socket_path);

        while let Ok((mut stream, _)) = listener.accept().await {
            tokio::spawn(async move {
                if let Err(e) = handle_client(&mut stream).await {
                    eprintln!("Client error: {}", e);
                }
            });
        }
        Ok::<(), String>(())
    })?;

    async fn handle_client(stream: &mut UnixStream) -> Result<(), String> {
        use crate::security::stores::tpm::get_tpm_store;
        use crate::security::tpm_protocol::{
            async_io::{receive_message_async, send_message_async},
            TpmRequest, TpmResponse,
        };
        use crate::security::MasterKey;
        use nix::sys::socket::{getsockopt, sockopt};
        use std::os::fd::AsFd;

        // 1. Verify peer (UID and binary path)
        let fd = stream.as_fd();
        let creds = getsockopt(&fd, sockopt::PeerCredentials)
            .map_err(|e| e.to_string())?;
        
        // Use PKEXEC_UID for verification
        let pkexec_uid_str = std::env::var("PKEXEC_UID").map_err(|_| "PKEXEC_UID not set".to_string())?;
        let target_uid: u32 = pkexec_uid_str.parse().map_err(|_| "Invalid PKEXEC_UID".to_string())?;
        
        // Verify UID matches the user who started us, or root
        if creds.uid() != target_uid && creds.uid() != 0 {
             return Err("Unauthorized: UID mismatch".to_string());
        }

        // Verify executable path (only Postail can connect)
        let exe_path = std::env::current_exe().map_err(|e| e.to_string())?;
        let peer_exe = std::fs::read_link(format!("/proc/{}/exe", creds.pid()))
            .map_err(|e| e.to_string())?;
        
        if exe_path != peer_exe {
            return Err("Unauthorized: Binary mismatch".to_string());
        }

        // 2. Process requests
        loop {
            let req: TpmRequest = match receive_message_async(stream).await {
                Ok(r) => r,
                Err(_) => break, // Connection closed
            };

            let store = get_tpm_store().ok_or_else(|| "TPM store not available".to_string())?;

            let res = match req {
                TpmRequest::Store { key } => {
                    match MasterKey::from_bytes(&key) {
                        Ok(mk) => match store.store(&mk) {
                            Ok(_) => TpmResponse::Ok { key: None },
                            Err(e) => TpmResponse::Err(e.to_string()),
                        },
                        Err(e) => TpmResponse::Err(e.to_string()),
                    }
                }
                TpmRequest::Retrieve => {
                    match store.retrieve() {
                        Ok(mk) => TpmResponse::Ok { key: Some(mk.as_bytes().to_vec()) },
                        Err(e) => TpmResponse::Err(e.to_string()),
                    }
                }
                TpmRequest::Delete => {
                    match store.delete() {
                        Ok(_) => TpmResponse::Ok { key: None },
                        Err(e) => TpmResponse::Err(e.to_string()),
                    }
                }
            };

            send_message_async(stream, &res).await?;
        }

        Ok(())
    }

    Ok(())
}

#[cfg(not(all(target_os = "linux", feature = "tpm")))]
pub fn tpm_helper_init() -> Result<(), String> {
    Err("TPM helper mode only available on Linux with TPM feature".to_string())
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
            cmd::mail::listing::save_attachment,
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
            cmd::settings::set_theme_config,
            cmd::settings::get_build_info
        ])
        .register_uri_scheme_protocol("postail", protocol::handler)
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
