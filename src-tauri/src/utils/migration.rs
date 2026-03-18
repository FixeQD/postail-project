use crate::globals::{DB_CONN, IMAP_MANAGER, SECURITY, SMTP_MANAGER};
use crate::utils::config::{get_data_dir, set_data_dir_override};
use std::fs;
use std::path::{Path, PathBuf};

#[cfg(all(target_os = "linux", feature = "tpm"))]
fn notify_tpm_helper_new_path(new_path: &str) {
    use crate::security::tpm::protocol::{receive_message, send_message, TpmRequest, TpmResponse};
    use std::os::unix::net::UnixStream;

    let uid = unsafe { nix::libc::getuid() };
    let socket_path = PathBuf::from(format!("/run/user/{}/postail-tpm.sock", uid));

    if !socket_path.exists() {
        return;
    }

    let Ok(mut stream) = UnixStream::connect(&socket_path) else {
        return;
    };

    let req = TpmRequest::UpdateDataDir {
        path: new_path.to_string(),
    };
    if send_message(&mut stream, &req).is_err() {
        return;
    }

    let _: Result<TpmResponse, _> = receive_message(&mut stream);
}

pub async fn perform_migration(new_path: &str) -> Result<(), String> {
    let old_dir = get_data_dir();
    let new_dir = PathBuf::from(new_path);

    if new_dir == old_dir {
        return Err("Target path is the same as current path".to_string());
    }

    tracing::info!(target: "postail", "[Migration] Starting migration from {:?} to {:?}", old_dir, new_dir);

    // 1. Stop all workers
    tracing::info!(target: "postail", "[Migration] Stopping workers...");

    // Stop IMAP syncs
    {
        tracing::info!(target: "postail", "[Migration] Stopping IMAP syncs...");
        let imap = IMAP_MANAGER.lock().await;
        let _ = imap.stop_all_syncs().await;
    }

    // Cleanup sync statuses
    {
        use crate::imap::sync_status::SYNC_STATUS_MANAGER;
        SYNC_STATUS_MANAGER.unregister_all().await;
    }

    // Stop SMTP outbox worker
    {
        tracing::info!(target: "postail", "[Migration] Stopping SMTP worker...");
        let smtp = SMTP_MANAGER.lock().await;
        smtp.stop_outbox_worker();
    }

    // Stop maintenance scheduler
    tracing::info!(target: "postail", "[Migration] Stopping maintenance scheduler...");
    crate::maintenance::stop_maintenance_scheduler();

    // 2. Drop DB connection and lock security (clear master key from memory)
    tracing::info!(target: "postail", "[Migration] Closing database connection...");
    {
        let mut db_guard = DB_CONN.lock().await;
        *db_guard = None;
    }
    {
        let mut security = SECURITY.lock().await;
        security.lock();
    }

    // 3. Move files
    tracing::info!(target: "postail", "[Migration] Moving files...");
    if !new_dir.exists() {
        fs::create_dir_all(&new_dir).map_err(|e| e.to_string())?;
    }

    move_dir_contents(&old_dir, &new_dir)?;

    // 4. Update override
    tracing::info!(target: "postail", "[Migration] Updating data path override...");
    set_data_dir_override(new_path).map_err(|e| e.to_string())?;

    // Notify the running TPM helper (if any) to switch to the new data path
    #[cfg(all(target_os = "linux", feature = "tpm"))]
    notify_tpm_helper_new_path(new_path);

    tracing::info!(target: "postail", "[Migration] Migration complete.");

    Ok(())
}

fn move_dir_contents(src: &Path, dst: &Path) -> Result<(), String> {
    if !src.exists() {
        return Ok(());
    }

    for entry in fs::read_dir(src).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        let file_name = path.file_name().ok_or("Invalid filename")?;
        let dest_path = dst.join(file_name);

        if path.is_dir() {
            fs::create_dir_all(&dest_path).map_err(|e| e.to_string())?;
            move_dir_contents(&path, &dest_path)?;
            fs::remove_dir(&path).map_err(|e| e.to_string())?;
        } else {
            fs::copy(&path, &dest_path).map_err(|e| e.to_string())?;
            fs::remove_file(&path).map_err(|e| e.to_string())?;
        }
    }

    Ok(())
}
