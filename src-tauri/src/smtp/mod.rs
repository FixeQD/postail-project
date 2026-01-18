use std::sync::{Arc, Mutex};

use rusqlite::Connection;

use crate::security::SecurityManager;

pub mod outbox;
pub mod sender;
pub mod worker;

pub struct SmtpManager {
    conn: Arc<Mutex<Option<Connection>>>,
    security: Arc<Mutex<SecurityManager>>,
}

impl SmtpManager {
    pub fn new(
        conn: Arc<Mutex<Option<Connection>>>,
        security: Arc<Mutex<SecurityManager>>,
    ) -> Self {
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
