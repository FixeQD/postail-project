use crate::imap::ImapManager;
use crate::security::SecurityManager;
use crate::smtp::SmtpManager;
use lazy_static::lazy_static;
use rusqlite::Connection;
use std::sync::{Arc, Mutex, Once};

lazy_static! {
    pub static ref OAUTH_PORT: Mutex<u16> = Mutex::new(0);
    pub static ref DB_CONN: Arc<Mutex<Option<Connection>>> = Arc::new(Mutex::new(None));
    pub static ref SECURITY: Arc<Mutex<SecurityManager>> = Arc::new(Mutex::new(
        SecurityManager::new().expect("Failed to initialize security")
    ));
    pub static ref IMAP_MANAGER: Arc<tokio::sync::Mutex<ImapManager>> =
        Arc::new(tokio::sync::Mutex::new(ImapManager::new(
            Arc::clone(&DB_CONN),
            Arc::clone(&SECURITY),
        )));
    pub static ref SMTP_MANAGER: Arc<Mutex<SmtpManager>> = Arc::new(Mutex::new(SmtpManager::new(
        Arc::clone(&DB_CONN),
        Arc::clone(&SECURITY),
    )));
}

static INIT: Once = Once::new();

pub fn set_oauth_port(port: u16) {
    INIT.call_once(|| {
        *OAUTH_PORT.lock().unwrap() = port;
    });
}

pub fn get_oauth_port() -> u16 {
    *OAUTH_PORT.lock().unwrap()
}
