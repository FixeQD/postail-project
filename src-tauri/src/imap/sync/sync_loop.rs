use std::sync::{Arc, Mutex};

use async_imap::Session;
use async_native_tls::TlsStream;
use async_std::net::TcpStream;
use tokio::time::Duration;

impl crate::imap::ImapManager {
    pub fn start_sync(&self, account_id: &str) -> Result<(), String> {
        Ok(())
    }
}
