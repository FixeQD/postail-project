use crate::db::DbPool;
use crate::imap::ImapManager;
use crate::security::SecurityManager;
use crate::smtp::SmtpManager;
use lazy_static::lazy_static;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use tokio::sync::Mutex;

pub static MINIMIZE_TO_TRAY: AtomicBool = AtomicBool::new(false);

lazy_static! {
    pub static ref OAUTH_PORT: std::sync::Mutex<u16> = std::sync::Mutex::new(0);
    pub static ref DB_CONN: Arc<Mutex<Option<DbPool>>> = Arc::new(Mutex::new(None));
    pub static ref SECURITY: Arc<Mutex<SecurityManager>> = Arc::new(Mutex::new(
        SecurityManager::new().expect("Failed to initialize security")
    ));
    pub static ref IMAP_MANAGER: Arc<Mutex<ImapManager>> = Arc::new(Mutex::new(ImapManager::new(
        Arc::clone(&DB_CONN),
        Arc::clone(&SECURITY),
    )));
    pub static ref SMTP_MANAGER: Arc<Mutex<SmtpManager>> = Arc::new(Mutex::new(SmtpManager::new(
        Arc::clone(&DB_CONN),
        Arc::clone(&SECURITY),
    )));
}

pub fn set_oauth_port(port: u16) {
    *OAUTH_PORT.lock().unwrap() = port;
}

pub fn get_oauth_port() -> u16 {
    *OAUTH_PORT.lock().unwrap()
}

pub async fn get_db_pool() -> Result<DbPool, crate::error::DBError> {
    let guard = DB_CONN.lock().await;
    guard
        .clone()
        .ok_or_else(|| crate::error::DBError::Pool("DB pool not initialized".to_string()))
}
