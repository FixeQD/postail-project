use std::str::FromStr;
use std::sync::{Arc, Mutex};

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

    /// Parses a string into an `EncryptionType`.
    ///
    /// On success returns the corresponding `EncryptionType` variant; on failure returns an `Err` with a message describing the unknown input.
    /// Recognized (case-insensitive) inputs:
    /// - `"tls"` or `"ssl"` -> `Tls`
    /// - `"starttls"` or `"start_tls"` -> `StartTls`
    /// - `"plain"`, `"none"`, or `""` -> `Plain`
    ///
    /// # Examples
    ///
    /// ```
    /// use std::str::FromStr;
    ///
    /// assert_eq!(EncryptionType::from_str("TLS").unwrap(), EncryptionType::Tls);
    /// assert_eq!(EncryptionType::from_str("start_tls").unwrap(), EncryptionType::StartTls);
    /// assert_eq!(EncryptionType::from_str("").unwrap(), EncryptionType::Plain);
    /// assert!(EncryptionType::from_str("invalid").is_err());
    /// ```
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
    /// Create a new SmtpManager using the provided shared connection and security manager.
    ///
    /// The returned manager shares the given `conn` and `security` handles and initializes
    /// its internal `app_handle` to `None`.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::{Arc, Mutex};
    /// use rusqlite::Connection;
    /// use crate::security::SecurityManager;
    /// use crate::smtp::SmtpManager;
    ///
    /// // Prepare shared handles (database connection omitted here; using None placeholder)
    /// let conn = Arc::new(Mutex::new(None::<Connection>));
    /// let security = Arc::new(Mutex::new(SecurityManager::default()));
    ///
    /// let manager = SmtpManager::new(conn, security);
    /// ```
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

    /// Stores the provided `AppHandle` inside the manager for later use by components that need access to the application handle.
    ///
    /// This replaces any previously stored handle.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// // Assume `manager` is an existing `SmtpManager` and `handle` is an `AppHandle`.
    /// manager.set_app_handle(handle);
    /// ```
    pub fn set_app_handle(&self, handle: AppHandle) {
        let mut guard = self.app_handle.lock().unwrap();
        *guard = Some(handle);
    }
}

impl Clone for SmtpManager {
    /// Creates a new `SmtpManager` that shares the same underlying connection, security manager, and app handle.
    ///
    /// # Returns
    /// A new `SmtpManager` instance that shares ownership of the internal `Arc`-wrapped fields with `self`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::sync::{Arc, Mutex};
    /// # use rusqlite::Connection;
    /// # use crate::security::SecurityManager;
    /// # use crate::smtp::SmtpManager;
    /// // Setup (hidden) — construct required arguments for SmtpManager::new
    /// # let conn = Arc::new(Mutex::new(None::<Connection>));
    /// # let security = Arc::new(Mutex::new(SecurityManager::default()));
    /// let manager = SmtpManager::new(conn, security);
    /// let cloned = manager.clone();
    /// // `cloned` shares the same internal state as `manager`.
    /// ```
    fn clone(&self) -> Self {
        Self {
            conn: Arc::clone(&self.conn),
            security: Arc::clone(&self.security),
            app_handle: Arc::clone(&self.app_handle),
        }
    }
}