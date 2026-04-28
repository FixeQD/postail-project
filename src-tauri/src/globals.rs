use crate::db::DbPool;
use crate::imap::ImapManager;
use crate::security::{CryptoHandle, SecurityManager};
use crate::smtp::SmtpManager;
use std::sync::Arc;
use std::sync::LazyLock;
use std::sync::atomic::AtomicBool;
use tokio::sync::Mutex;
use tokio::sync::RwLock;

pub static MINIMIZE_TO_TRAY: AtomicBool = AtomicBool::new(false);

pub static OAUTH_PORT: LazyLock<std::sync::Mutex<u16>> = LazyLock::new(|| std::sync::Mutex::new(0));
pub static DB_CONN: LazyLock<Arc<Mutex<Option<DbPool>>>> =
    LazyLock::new(|| Arc::new(Mutex::new(None)));
pub static SECURITY: LazyLock<Arc<Mutex<SecurityManager>>> = LazyLock::new(|| {
    Arc::new(Mutex::new(
        SecurityManager::new().expect("Failed to initialize security"),
    ))
});

pub static CRYPTO_ACTOR: LazyLock<Arc<RwLock<Option<CryptoHandle>>>> =
    LazyLock::new(|| Arc::new(RwLock::new(None)));
pub static IMAP_MANAGER: LazyLock<Arc<Mutex<ImapManager>>> = LazyLock::new(|| {
    Arc::new(Mutex::new(ImapManager::new(
        Arc::clone(&DB_CONN),
        Arc::clone(&SECURITY),
    )))
});
pub static SMTP_MANAGER: LazyLock<Arc<Mutex<SmtpManager>>> = LazyLock::new(|| {
    Arc::new(Mutex::new(SmtpManager::new(
        Arc::clone(&DB_CONN),
        Arc::clone(&SECURITY),
    )))
});

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
