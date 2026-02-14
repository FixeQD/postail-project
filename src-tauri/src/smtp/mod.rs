use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::Mutex;

use rusqlite::Connection;
use tauri::AppHandle;

use crate::security::SecurityManager;

pub mod mime_builder;
pub mod outbox;
pub mod sender;
pub mod worker;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EncryptionType {
    Tls,
    StartTls,
    Plain,
}

impl FromStr for EncryptionType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "tls" | "ssl" => Ok(Self::Tls),
            "starttls" | "start_tls" => Ok(Self::StartTls),
            "plain" | "none" | "" => Ok(Self::Plain),
            _ => Err(format!("Unknown encryption type: {}", s)),
        }
    }
}

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

    pub async fn set_app_handle(&self, handle: AppHandle) {
        let mut guard = self.app_handle.lock().await;
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
