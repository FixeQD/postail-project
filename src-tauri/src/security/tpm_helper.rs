use crate::security::stores::tpm::get_tpm_store;
use crate::security::tpm_protocol::{
    async_io::{receive_message_async, send_message_async},
    TpmRequest, TpmResponse,
};
use crate::security::MasterKey;
use nix::sys::socket::{getsockopt, sockopt};
use std::fs;
use std::os::fd::{AsFd, FromRawFd, RawFd};
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use tokio::net::{UnixListener, UnixStream};

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

        while let Ok((mut stream, _)) = listener.accept().await {
            let target_uid = uid.as_raw();
            tokio::spawn(async move {
                if let Err(e) = handle_client(&mut stream, target_uid).await {
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

#[cfg(all(target_os = "linux", feature = "tpm"))]
async fn handle_client(stream: &mut UnixStream, target_uid: u32) -> Result<(), String> {
    let fd = stream.as_fd();
    let creds = getsockopt(&fd, sockopt::PeerCredentials).map_err(|e| e.to_string())?;

    if creds.uid() != target_uid && creds.uid() != 0 {
        return Err("Unauthorized: UID mismatch".to_string());
    }

    let exe_path = std::env::current_exe().map_err(|e| e.to_string())?;
    let peer_exe =
        std::fs::read_link(format!("/proc/{}/exe", creds.pid())).map_err(|e| e.to_string())?;

    if exe_path != peer_exe {
        return Err("Unauthorized: Binary mismatch".to_string());
    }

    let store = get_tpm_store().ok_or_else(|| "TPM store not available".to_string())?;

    loop {
        let req: TpmRequest = match receive_message_async(stream).await {
            Ok(r) => r,
            Err(_) => break,
        };

        let res = match req {
            TpmRequest::Ping => {
                if store.is_available() {
                    TpmResponse::Ok { key: None }
                } else {
                    TpmResponse::Err("TPM hardware not accessible by helper".to_string())
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
        };

        send_message_async(stream, &res).await?;
    }
    Ok(())
}
