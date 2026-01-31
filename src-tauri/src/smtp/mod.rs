use std::sync::{Arc, Mutex};

use rusqlite::Connection;
use tauri::AppHandle;

use crate::security::SecurityManager;

pub mod mime_builder;
pub mod outbox;
pub mod sender;
pub mod worker;

pub struct SmtpManager {
    conn: Arc<Mutex<Option<Connection>>>,
    security: Arc<Mutex<SecurityManager>>,
    app_handle: Arc<Mutex<Option<AppHandle>>>,
}

impl SmtpManager {
    pub fn new(
        conn: Arc<Mutex<Option<Connection>>>,
        security: Arc<Mutex<SecurityManager>>,
    ) -> Self {
        Self {
            conn,
            security,
            app_handle: Arc::new(Mutex::new(None)),
        }
    }

    pub fn set_app_handle(&self, handle: AppHandle) {
        let mut guard = self.app_handle.lock().unwrap();
        *guard = Some(handle);
    }
}

impl Clone for SmtpManager {
    fn clone(&self) -> Self {
        Self {
            conn: Arc::clone(&self.conn),
            security: Arc::clone(&self.security),
            app_handle: Arc::clone(&self.app_handle),
        }
    }
}
