pub mod connection;
pub mod sync;

use crate::db::{upsert_mailbox, upsert_message, MailHeader, Mailbox};
use crate::security::SecurityManager;
use async_imap::Session;
use async_native_tls::TlsStream;
use async_std::net::TcpStream;
use rusqlite::Connection;
use std::sync::{Arc, Mutex};
use tokio::time::Duration;

pub struct ImapManager {
    conn: Arc<Mutex<Connection>>,
    security: Arc<Mutex<SecurityManager>>,
}

impl ImapManager {
    pub fn new(conn: Arc<Mutex<Connection>>, security: Arc<Mutex<SecurityManager>>) -> Self {
        Self { conn, security }
    }
}

impl Clone for ImapManager {
    fn clone(&self) -> Self {
        Self {
            conn: Arc::clone(&self.conn),
            security: Arc::clone(&self.security),
        }
    }
}
