pub mod outbox;
pub mod sender;

use crate::db::{enqueue_message, list_outbox, OutboxItem};
use crate::security::SecurityManager;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};
use rusqlite::Connection;
use std::fs;
use std::sync::{Arc, Mutex};
use tokio::time::Duration;
use uuid::Uuid;

pub struct SmtpManager {
    conn: Arc<Mutex<Connection>>,
    security: Arc<Mutex<SecurityManager>>,
}

impl SmtpManager {
    pub fn new(conn: Arc<Mutex<Connection>>, security: Arc<Mutex<SecurityManager>>) -> Self {
        Self { conn, security }
    }
}

impl Clone for SmtpManager {
    fn clone(&self) -> Self {
        Self {
            conn: Arc::clone(&self.conn),
            security: Arc::clone(&self.security),
        }
    }
}
