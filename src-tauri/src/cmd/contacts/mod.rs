pub mod vcf;
pub mod groups;

pub use vcf::*;
pub use groups::*;

use crate::db::account::contact_groups::ContactGroup;
use crate::db::account::contacts::Contact;
use crate::globals::get_db_pool;
use serde::Serialize;
use tauri::command;

#[command]
pub async fn list_contacts() -> Result<Vec<Contact>, String> {
    let pool = get_db_pool().await.map_err(|e| e.to_string())?;
    let conn = pool.get().map_err(|e| e.to_string())?;
    crate::db::account::contacts::list_contacts(&conn).map_err(|e| e.to_string())
}

#[command]
pub async fn search_contacts_full(
    query: String,
    limit: Option<u32>,
) -> Result<Vec<Contact>, String> {
    let limit = limit.unwrap_or(50);
    let pool = get_db_pool().await.map_err(|e| e.to_string())?;
    let conn = pool.get().map_err(|e| e.to_string())?;
    crate::db::account::contacts::search_contacts(&conn, &query, limit).map_err(|e| e.to_string())
}

#[command]
pub async fn get_contact_messages(
    account_id: i64,
    email: String,
    limit: Option<u32>,
) -> Result<Vec<crate::db::MailHeader>, String> {
    let limit = limit.unwrap_or(50);
    let pool = get_db_pool().await.map_err(|e| e.to_string())?;
    let conn = pool.get().map_err(|e| e.to_string())?;
    crate::db::account::contacts::get_contact_messages(&conn, &account_id.to_string(), &email, limit)
        .map_err(|e| e.to_string())
}

#[command]
pub async fn create_contact(
    name: Option<String>,
    first_name: Option<String>,
    middle_name: Option<String>,
    last_name: Option<String>,
    suffix: Option<String>,
    nickname: Option<String>,
    email: String,
    phone: Option<String>,
    phone_work: Option<String>,
    phone_home: Option<String>,
    phone_fax: Option<String>,
    work_email: Option<String>,
    company: Option<String>,
    job_title: Option<String>,
    department: Option<String>,
    role: Option<String>,
    website: Option<String>,
    address_home: Option<String>,
    address_work: Option<String>,
    notes: Option<String>,
    avatar_url: Option<String>,
    birthday: Option<i64>,
    anniversary: Option<i64>,
    gender: Option<String>,
) -> Result<i64, String> {
    let pool = get_db_pool().await.map_err(|e| e.to_string())?;
    let conn = pool.get().map_err(|e| e.to_string())?;
    crate::db::account::contacts::create_contact(
        &conn,
        &email,
        name.as_deref(),
        first_name.as_deref(),
        middle_name.as_deref(),
        last_name.as_deref(),
        suffix.as_deref(),
        nickname.as_deref(),
        phone.as_deref(),
        phone_work.as_deref(),
        phone_home.as_deref(),
        phone_fax.as_deref(),
        work_email.as_deref(),
        company.as_deref(),
        job_title.as_deref(),
        department.as_deref(),
        role.as_deref(),
        website.as_deref(),
        address_home.as_deref(),
        address_work.as_deref(),
        notes.as_deref(),
        avatar_url.as_deref(),
        birthday,
        anniversary,
        gender.as_deref(),
    )
    .map_err(|e| e.to_string())
}

#[command]
pub async fn update_contact(
    id: i64,
    name: Option<String>,
    first_name: Option<String>,
    middle_name: Option<String>,
    last_name: Option<String>,
    suffix: Option<String>,
    nickname: Option<String>,
    email: String,
    phone: Option<String>,
    phone_work: Option<String>,
    phone_home: Option<String>,
    phone_fax: Option<String>,
    work_email: Option<String>,
    company: Option<String>,
    job_title: Option<String>,
    department: Option<String>,
    role: Option<String>,
    website: Option<String>,
    address_home: Option<String>,
    address_work: Option<String>,
    notes: Option<String>,
    avatar_url: Option<String>,
    birthday: Option<i64>,
    anniversary: Option<i64>,
    gender: Option<String>,
) -> Result<(), String> {
    let pool = get_db_pool().await.map_err(|e| e.to_string())?;
    let conn = pool.get().map_err(|e| e.to_string())?;
    crate::db::account::contacts::update_contact(
        &conn,
        id,
        &email,
        name.as_deref(),
        first_name.as_deref(),
        middle_name.as_deref(),
        last_name.as_deref(),
        suffix.as_deref(),
        nickname.as_deref(),
        phone.as_deref(),
        phone_work.as_deref(),
        phone_home.as_deref(),
        phone_fax.as_deref(),
        work_email.as_deref(),
        company.as_deref(),
        job_title.as_deref(),
        department.as_deref(),
        role.as_deref(),
        website.as_deref(),
        address_home.as_deref(),
        address_work.as_deref(),
        notes.as_deref(),
        avatar_url.as_deref(),
        birthday,
        anniversary,
        gender.as_deref(),
    )
    .map_err(|e| e.to_string())
}

#[command]
pub async fn delete_contact(id: i64) -> Result<(), String> {
    let pool = get_db_pool().await.map_err(|e| e.to_string())?;
    let conn = pool.get().map_err(|e| e.to_string())?;
    crate::db::account::contacts::delete_contact(&conn, id).map_err(|e| e.to_string())
}

#[derive(Serialize)]
pub struct ContactSearchResults {
    pub contacts: Vec<Contact>,
    pub groups: Vec<ContactGroup>,
}

#[command]
pub async fn search_contacts_and_groups(
    query: String,
    limit: Option<u32>,
) -> Result<ContactSearchResults, String> {
    let limit = limit.unwrap_or(50);
    let pool = get_db_pool().await.map_err(|e| e.to_string())?;
    let conn = pool.get().map_err(|e| e.to_string())?;

    let contacts = crate::db::account::contacts::search_contacts(&conn, &query, limit)
        .map_err(|e| e.to_string())?;
    
    let groups = crate::db::account::contact_groups::search_groups(&query)
        .await
        .map_err(|e| e.to_string())?;

    Ok(ContactSearchResults { contacts, groups })
}
