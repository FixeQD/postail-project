use crate::security::storage::SecretStore;
use crate::security::tpm::protocol::{
    async_io::{receive_message_async, send_message_async},
    TpmRequest, TpmResponse,
};
use crate::security::MasterKey;
use nix::sys::socket::{getsockopt, sockopt};
use std::fs;
use std::os::fd::{AsFd, FromRawFd, RawFd};
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::Mutex;

#[cfg(all(target_os = "linux", feature = "tpm"))]
fn get_executable_path() -> std::io::Result<PathBuf> {
    if let Ok(appimage) = std::env::var("APPIMAGE") {
        return Ok(PathBuf::from(appimage));
    }
    std::env::current_exe()
}

/// TPM helper mode: Initialize TPM with elevated privileges (Linux only)
#[cfg(all(target_os = "linux", feature = "tpm"))]
pub fn tpm_helper_init() -> Result<(), String> {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| e.to_string())?;

    rt.block_on(async {
        let (uid, gid) = get_helper_identity()?;
        let socket_path = setup_socket_dir(uid, gid)?;

        let listener = UnixListener::bind(&socket_path)
            .map_err(|e| format!("Failed to bind socket at {:?}: {}", socket_path, e))?;

        // Set socket permissions so only the user can connect
        let _ = fs::set_permissions(&socket_path, fs::Permissions::from_mode(0o600));
        let _ = nix::unistd::chown(&socket_path, Some(uid), Some(gid));

        tracing::info!(
            "TPM Proxy Helper (UID: {}) listening on {:?}",
            uid.as_raw(),
            socket_path
        );

        if let Ok(parent_pid) = get_parent_pid() {
            start_watchdog(parent_pid)?;
        }

        let storage_path = Arc::new(Mutex::new(
            crate::security::tpm::store::common::default_storage_path(),
        ));

        while let Ok((mut stream, _)) = listener.accept().await {
            let target_uid = uid.as_raw();
            let path = Arc::clone(&storage_path);
            tokio::spawn(async move {
                if let Err(e) = handle_client(&mut stream, target_uid, path).await {
                    tracing::error!("Client error: {}", e);
                }
            });
        }
        Ok::<(), String>(())
    })
}

#[cfg(not(all(target_os = "linux", feature = "tpm")))]
pub fn tpm_helper_init() -> Result<(), String> {
    Err("TPM helper mode only available on Linux with TPM feature".to_string())
}

#[cfg(all(target_os = "linux", feature = "tpm"))]
fn get_helper_identity() -> Result<(nix::unistd::Uid, nix::unistd::Gid), String> {
    let pkexec_uid_str = std::env::var("PKEXEC_UID")
        .map_err(|_| "PKEXEC_UID env var not set (Helper must be run via pkexec)".to_string())?;
    let uid_raw: u32 = pkexec_uid_str
        .parse()
        .map_err(|_| "Invalid PKEXEC_UID".to_string())?;
    let uid = nix::unistd::Uid::from_raw(uid_raw);

    let user = nix::unistd::User::from_uid(uid)
        .map_err(|e| format!("Failed to find user with UID {}: {}", uid_raw, e))?
        .ok_or_else(|| format!("User with UID {} not found", uid_raw))?;

    Ok((uid, user.gid))
}

#[cfg(all(target_os = "linux", feature = "tpm"))]
fn setup_socket_dir(uid: nix::unistd::Uid, gid: nix::unistd::Gid) -> Result<PathBuf, String> {
    let socket_dir = PathBuf::from(format!("/run/user/{}", uid));

    if !socket_dir.exists() {
        fs::create_dir_all(&socket_dir)
            .map_err(|e| format!("Failed to create socket dir: {}", e))?;
        let _ = nix::unistd::chown(&socket_dir, Some(uid), Some(gid));
    }

    let socket_path = socket_dir.join("postail-tpm.sock");
    let _ = fs::remove_file(&socket_path);
    Ok(socket_path)
}

#[cfg(all(target_os = "linux", feature = "tpm"))]
fn get_parent_pid() -> Result<u32, String> {
    std::env::var("POSTAIL_PARENT_PID")
        .map_err(|_| "POSTAIL_PARENT_PID not set".to_string())?
        .parse()
        .map_err(|_| "Invalid POSTAIL_PARENT_PID".to_string())
}

#[cfg(all(target_os = "linux", feature = "tpm"))]
fn start_watchdog(parent_pid: u32) -> Result<(), String> {
    let raw_pidfd = unsafe {
        nix::libc::syscall(
            nix::libc::SYS_pidfd_open,
            parent_pid as nix::libc::pid_t,
            0 as nix::libc::c_int,
        )
    };
    if raw_pidfd < 0 {
        return Err(format!(
            "pidfd_open({}) failed: {}",
            parent_pid,
            std::io::Error::last_os_error()
        ));
    }
    let pidfd = unsafe { std::os::fd::OwnedFd::from_raw_fd(raw_pidfd as RawFd) };

    tokio::spawn(async move {
        use tokio::io::unix::AsyncFd;
        if let Ok(async_pidfd) = AsyncFd::new(pidfd) {
            let _ = async_pidfd.readable().await;
            tracing::warn!(
                "[TPM helper] Parent process {} exited — shutting down",
                parent_pid
            );
        }
        std::process::exit(0);
    });
    Ok(())
}

/// Read the APPIMAGE env var from a peer process's /proc/{pid}/environ.
/// Returns None if the file is unreadable or the variable is not set.
#[cfg(all(target_os = "linux", feature = "tpm"))]
fn read_peer_appimage_env(pid: u32) -> Option<String> {
    let environ = std::fs::read(format!("/proc/{}/environ", pid)).ok()?;
    for entry in environ.split(|&b| b == 0) {
        if let Ok(s) = std::str::from_utf8(entry) {
            if let Some(val) = s.strip_prefix("APPIMAGE=") {
                return Some(val.to_string());
            }
        }
    }
    None
}

#[cfg(all(target_os = "linux", feature = "tpm"))]
async fn handle_client(
    stream: &mut UnixStream,
    target_uid: u32,
    storage_path: Arc<Mutex<PathBuf>>,
) -> Result<(), String> {
    let fd = stream.as_fd();
    let creds = getsockopt(&fd, sockopt::PeerCredentials).map_err(|e| e.to_string())?;

    if creds.uid() != target_uid && creds.uid() != 0 {
        return Err("Unauthorized: UID mismatch".to_string());
    }

    let authorized = if let Ok(own_appimage) = std::env::var("APPIMAGE") {
        read_peer_appimage_env(creds.pid() as u32)
            .map(|peer_appimage| peer_appimage == own_appimage)
            .unwrap_or(false)
    } else {
        let exe_path = get_executable_path().map_err(|e| e.to_string())?;
        let peer_exe =
            std::fs::read_link(format!("/proc/{}/exe", creds.pid())).map_err(|e| e.to_string())?;
        exe_path == peer_exe
    };

    if !authorized {
        return Err("Unauthorized: Binary mismatch".to_string());
    }

    use crate::security::tpm::store::linux::LinuxTpmStore;

    loop {
        let req: TpmRequest = match receive_message_async(stream).await {
            Ok(r) => r,
            Err(_) => break,
        };

        let current_path = storage_path.lock().await.clone();
        let store = match LinuxTpmStore::with_storage_path(current_path) {
            Ok(s) => s,
            Err(e) => {
                send_message_async(stream, &TpmResponse::Err(e.to_string())).await?;
                continue;
            }
        };

        let res = match req {
            TpmRequest::Ping => {
                if store.is_available() {
                    TpmResponse::Ok { key: None }
                } else {
                    TpmResponse::Err("TPM hardware not accessible by helper".to_string())
                }
            }
            TpmRequest::Seal { key } => {
                use crate::security::tpm::store::common::{self, create_primary_key};
                match MasterKey::from_bytes(&key) {
                    Ok(mk) => match store.create_context() {
                        Ok(mut ctx) => {
                            let primary = match create_primary_key(&mut ctx) {
                                Ok(p) => p,
                                Err(e) => {
                                    send_message_async(stream, &TpmResponse::Err(e.to_string())).await?;
                                    continue;
                                }
                            };
                            match common::seal_data(&mut ctx, primary.key_handle, mk.as_bytes()) {
                                Ok(sealed) => {
                                    let _ = ctx.flush_context(primary.key_handle.into());
                                    send_message_async(stream, &TpmResponse::Ok { key: Some(sealed) }).await?;
                                    continue;
                                }
                                Err(e) => {
                                    let _ = ctx.flush_context(primary.key_handle.into());
                                    send_message_async(stream, &TpmResponse::Err(e.to_string())).await?;
                                    continue;
                                }
                            }
                        }
                        Err(e) => {
                            send_message_async(stream, &TpmResponse::Err(e.to_string())).await?;
                            continue;
                        }
                    },
                    Err(e) => TpmResponse::Err(e.to_string()),
                }
            }
            TpmRequest::Unseal { data } => {
                use crate::security::tpm::store::common::{self, create_primary_key};
                match store.create_context() {
                    Ok(mut ctx) => {
                        let primary = match create_primary_key(&mut ctx) {
                            Ok(p) => p,
                            Err(e) => {
                                send_message_async(stream, &TpmResponse::Err(e.to_string())).await?;
                                continue;
                            }
                        };
                        match common::unseal_data(&mut ctx, primary.key_handle, &data) {
                            Ok(unsealed) => {
                                let _ = ctx.flush_context(primary.key_handle.into());
                                send_message_async(stream, &TpmResponse::Ok { key: Some(unsealed) }).await?;
                                continue;
                            }
                            Err(e) => {
                                let _ = ctx.flush_context(primary.key_handle.into());
                                send_message_async(stream, &TpmResponse::Err(e.to_string())).await?;
                                continue;
                            }
                        }
                    }
                    Err(e) => {
                        send_message_async(stream, &TpmResponse::Err(e.to_string())).await?;
                        continue;
                    }
                }
            }
            TpmRequest::Store { key } => match MasterKey::from_bytes(&key) {
                Ok(mk) => match store.store(&mk) {
                    Ok(_) => TpmResponse::Ok { key: None },
                    Err(e) => TpmResponse::Err(e.to_string()),
                },
                Err(e) => TpmResponse::Err(e.to_string()),
            },
            TpmRequest::Retrieve => match store.retrieve() {
                Ok(mk) => TpmResponse::Ok {
                    key: Some(mk.as_bytes().to_vec()),
                },
                Err(e) => TpmResponse::Err(e.to_string()),
            },
            TpmRequest::Delete => match store.delete() {
                Ok(_) => TpmResponse::Ok { key: None },
                Err(e) => TpmResponse::Err(e.to_string()),
            },
            TpmRequest::UpdateDataDir { path } => {
                *storage_path.lock().await = PathBuf::from(&path).join("security");
                TpmResponse::Ok { key: None }
            }
            TpmRequest::StoreFile { path, data } => {
                match fs::write(&path, data) {
                    Ok(_) => TpmResponse::Ok { key: None },
                    Err(e) => TpmResponse::Err(e.to_string()),
                }
            }
            TpmRequest::DeleteFile { path } => {
                match fs::remove_file(&path) {
                    Ok(_) => TpmResponse::Ok { key: None },
                    Err(e) if e.kind() == std::io::ErrorKind::NotFound => TpmResponse::Ok { key: None },
                    Err(e) => TpmResponse::Err(e.to_string()),
                }
            }
        };

        send_message_async(stream, &res).await?;
    }
    Ok(())
}
