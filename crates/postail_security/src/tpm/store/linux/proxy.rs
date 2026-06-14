use crate::tpm::protocol::{
    TpmRequest, TpmResponse, receive_message, send_message,
};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;

pub fn get_socket_path() -> PathBuf {
    let uid = unsafe { nix::libc::getuid() };
    PathBuf::from(format!("/run/user/{}/postail-tpm.sock", uid))
}

pub fn is_socket_alive() -> bool {
    let path = get_socket_path();
    if !path.exists() {
        return false;
    }
    std::os::unix::net::UnixStream::connect(&path).is_ok()
}

pub fn call_proxy(req: TpmRequest) -> Result<Option<Vec<u8>>, String> {
    let path = get_socket_path();
    let mut stream = UnixStream::connect(&path)
        .map_err(|e| format!("Failed to connect to TPM helper: {}", e))?;

    send_message(&mut stream, &req)?;
    let res: TpmResponse = receive_message(&mut stream)?;

    match res {
        TpmResponse::Ok { key } => Ok(key),
        TpmResponse::Err(e) => Err(e),
    }
}
