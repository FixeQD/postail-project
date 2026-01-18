use crate::security::SecurityManager;
use rusqlite::Connection;
use std::sync::{Arc, Mutex};

pub mod connection;
pub mod mailbox;
pub mod sync;
pub mod sync_status;

pub struct ImapManager {
    conn: Arc<Mutex<Option<Connection>>>,
    security: Arc<Mutex<SecurityManager>>,
}

impl ImapManager {
    pub fn new(
        conn: Arc<Mutex<Option<Connection>>>,
        security: Arc<Mutex<SecurityManager>>,
    ) -> Self {
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
