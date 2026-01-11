pub mod db;
pub mod error;
pub mod oauth;
pub mod security;

use lazy_static::lazy_static;
use rusqlite::Connection;
use std::sync::{Arc, Mutex};
use tauri::Emitter;

use crate::db::{
    add_account as db_add_account, init_db, list_accounts as db_list_accounts,
    remove_account as db_remove_account, AccountInput, AccountMeta,
};
use crate::error::{DBError, OAuthError};
use crate::security::SecurityManager;

lazy_static! {
    static ref DB_CONN: Arc<Mutex<Connection>> = Arc::new(Mutex::new(init_db().unwrap()));
    static ref SECURITY: SecurityManager = SecurityManager::new().unwrap();
}

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn add_account(input: AccountInput) -> Result<AccountMeta, String> {
    let conn = DB_CONN.lock().unwrap();
    db_add_account(&*conn, input, &SECURITY).map_err(|e| e.to_string())
}

#[tauri::command]
fn list_accounts() -> Result<Vec<AccountMeta>, String> {
    let conn = DB_CONN.lock().unwrap();
    db_list_accounts(&*conn).map_err(|e| e.to_string())
}

#[tauri::command]
fn remove_account(id: String) -> Result<(), String> {
    let conn = DB_CONN.lock().unwrap();
    db_remove_account(&*conn, &id).map_err(|e| e.to_string())
}

#[tauri::command]
async fn start_oauth_flow(provider: String) -> Result<String, String> {
    let provider = match provider.as_str() {
        "gmail" => oauth::Provider::Gmail,
        "outlook" => oauth::Provider::Outlook,
        _ => return Err("Unknown provider".to_string()),
    };
    match oauth::start_oauth_flow(provider) {
        Ok(url) => Ok(url),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
async fn complete_oauth_flow(code: String, state: String) -> Result<oauth::OAuthTokens, String> {
    match oauth::complete_oauth_flow(code, state).await {
        Ok(tokens) => Ok(tokens),
        Err(e) => Err(e.to_string()),
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_os::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            start_oauth_flow,
            complete_oauth_flow,
            add_account,
            list_accounts,
            remove_account
        ])
        .register_uri_scheme_protocol("postail", |app, request| {
            if request.uri().path() == "/oauth/callback" {
                let uri = request.uri().to_string();
                if let Ok(url) = url::Url::parse(&uri) {
                    let query_pairs: std::collections::HashMap<_, _> =
                        url.query_pairs().into_owned().collect();
                    if let (Some(code), Some(state)) =
                        (query_pairs.get("code"), query_pairs.get("state"))
                    {
                        let _ = app.app_handle().emit(
                            "oauth_callback",
                            serde_json::json!({ "code": code, "state": state }),
                        );
                    }
                }
            }
            tauri::http::Response::builder()
                .status(200)
                .body(std::borrow::Cow::Owned(vec![]))
                .unwrap()
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
