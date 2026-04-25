use crate::db::{Contact, MailHeader};
use crate::globals::get_db_pool;
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
    account_id: String,
    email: String,
    limit: Option<u32>,
) -> Result<Vec<MailHeader>, String> {
    let limit = limit.unwrap_or(50);
    let pool = get_db_pool().await.map_err(|e| e.to_string())?;
    let conn = pool.get().map_err(|e| e.to_string())?;
    crate::db::account::contacts::get_contact_messages(&conn, &account_id, &email, limit)
        .map_err(|e| e.to_string())
}

#[command]
pub async fn update_contact(
    id: i64,
    name: Option<String>,
    email: String,
    phone: Option<String>,
    company: Option<String>,
    notes: Option<String>,
    avatar_url: Option<String>,
    birthday: Option<i64>,
) -> Result<(), String> {
    let pool = get_db_pool().await.map_err(|e| e.to_string())?;
    let conn = pool.get().map_err(|e| e.to_string())?;
    crate::db::account::contacts::update_contact(
        &conn,
        id,
        name.as_deref(),
        &email,
        phone.as_deref(),
        company.as_deref(),
        notes.as_deref(),
        avatar_url.as_deref(),
        birthday,
    )
    .map_err(|e| e.to_string())
}

#[command]
pub async fn create_contact(
    name: Option<String>,
    email: String,
    phone: Option<String>,
    company: Option<String>,
    notes: Option<String>,
    avatar_url: Option<String>,
    birthday: Option<i64>,
) -> Result<i64, String> {
    let pool = get_db_pool().await.map_err(|e| e.to_string())?;
    let conn = pool.get().map_err(|e| e.to_string())?;
    crate::db::account::contacts::create_contact(
        &conn,
        name.as_deref(),
        &email,
        phone.as_deref(),
        company.as_deref(),
        notes.as_deref(),
        avatar_url.as_deref(),
        birthday,
    )
    .map_err(|e| e.to_string())
}

#[command]
pub async fn delete_contact(id: i64) -> Result<(), String> {
    let pool = get_db_pool().await.map_err(|e| e.to_string())?;
    let conn = pool.get().map_err(|e| e.to_string())?;
    crate::db::account::contacts::delete_contact(&conn, id).map_err(|e| e.to_string())
}

#[derive(serde::Serialize)]
pub struct ImportContactsResult {
    pub imported: u32,
    pub updated: u32,
    pub errors: u32,
}

#[command]
pub async fn import_contacts_vcf(path: String) -> Result<ImportContactsResult, String> {
    use calcard::{Entry, Parser};
    use calcard::vcard::{VCardProperty, VCardValue};
    use chrono::{NaiveDate, TimeZone, Utc};
    use std::fs;

    let contents = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let mut parser = Parser::new(&contents);
    let mut result = ImportContactsResult {
        imported: 0,
        updated: 0,
        errors: 0,
    };

    let pool = get_db_pool().await.map_err(|e| e.to_string())?;
    let conn = pool.get().map_err(|e| e.to_string())?;

    loop {
        match parser.entry() {
            Entry::VCard(vcard) => {
                let mut name = None;
                let mut email = None;
                let mut phone = None;
                let mut company = None;
                let mut notes = None;
                let mut birthday = None;

                for entry in vcard.entries {
                    match entry.name {
                        VCardProperty::Fn => {
                            if let Some(VCardValue::Text(s)) = entry.values.first() {
                                name = Some(s.clone());
                            }
                        }
                        VCardProperty::Email => {
                            if email.is_none() {
                                if let Some(VCardValue::Text(s)) = entry.values.first() {
                                    email = Some(s.clone());
                                }
                            }
                        }
                        VCardProperty::Tel => {
                            if phone.is_none() {
                                if let Some(VCardValue::Text(s)) = entry.values.first() {
                                    phone = Some(s.clone());
                                }
                            }
                        }
                        VCardProperty::Org => {
                            if company.is_none() {
                                if let Some(VCardValue::Text(s)) = entry.values.first() {
                                    company = Some(s.split(';').next().unwrap_or(s).to_string());
                                }
                            }
                        }
                        VCardProperty::Note => {
                            if notes.is_none() {
                                if let Some(VCardValue::Text(s)) = entry.values.first() {
                                    notes = Some(s.clone());
                                }
                            }
                        }
                        VCardProperty::Bday => {
                            if birthday.is_none() {
                                if let Some(VCardValue::PartialDateTime(pdt)) = entry.values.first() {
                                    if let (Some(y), Some(m), Some(d)) = (pdt.year, pdt.month, pdt.day) {
                                        if let Some(date) = NaiveDate::from_ymd_opt(y as i32, m as u32, d as u32) {
                                            if let Some(dt) = date.and_hms_opt(0, 0, 0) {
                                                birthday = Some(Utc.from_utc_datetime(&dt).timestamp());
                                            }
                                        }
                                    }
                                } else if let Some(VCardValue::Text(s)) = entry.values.first() {
                                    if let Ok(date) = NaiveDate::parse_from_str(s, "%Y-%m-%d") {
                                        if let Some(dt) = date.and_hms_opt(0, 0, 0) {
                                            birthday = Some(Utc.from_utc_datetime(&dt).timestamp());
                                        }
                                    } else if let Ok(date) = NaiveDate::parse_from_str(s, "%Y%m%d") {
                                        if let Some(dt) = date.and_hms_opt(0, 0, 0) {
                                            birthday = Some(Utc.from_utc_datetime(&dt).timestamp());
                                        }
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }

                if let Some(email) = email {
                    match crate::db::account::contacts::upsert_contact_full(
                        &conn,
                        &email,
                        name.as_deref(),
                        phone.as_deref(),
                        company.as_deref(),
                        notes.as_deref(),
                        None, // avatar_url
                        birthday,
                    ) {
                        Ok(true) => result.imported += 1,
                        Ok(false) => result.updated += 1,
                        Err(_) => result.errors += 1,
                    }
                } else {
                    result.errors += 1;
                }
            }
            Entry::Eof => break,
            _ => { }
        }
    }

    Ok(result)
}

#[command]
pub async fn export_contacts_vcf(path: String, id: Option<i64>) -> Result<u32, String> {
    use calcard::vcard::{VCard, VCardEntry, VCardProperty, VCardValue};
    use chrono::DateTime;
    use std::fs;

    let pool = get_db_pool().await.map_err(|e| e.to_string())?;
    let conn = pool.get().map_err(|e| e.to_string())?;

    let contacts = if let Some(contact_id) = id {
        match crate::db::account::contacts::get_contact_by_id(&conn, contact_id).map_err(|e| e.to_string())? {
            Some(contact) => vec![contact],
            None => return Err("Contact not found".to_string()),
        }
    } else {
        crate::db::account::contacts::list_contacts(&conn).map_err(|e| e.to_string())?
    };
    
    let mut exported = 0;
    let mut output = String::new();

    for contact in &contacts {
        let mut entries = Vec::new();
        
        let name = contact.name.clone().unwrap_or_else(|| contact.email.clone());
        entries.push(VCardEntry {
            group: None,
            name: VCardProperty::Fn,
            params: vec![],
            values: vec![VCardValue::Text(name)],
        });

        entries.push(VCardEntry {
            group: None,
            name: VCardProperty::Email,
            params: vec![],
            values: vec![VCardValue::Text(contact.email.clone())],
        });

        if let Some(phone) = &contact.phone {
            entries.push(VCardEntry {
                group: None,
                name: VCardProperty::Tel,
                params: vec![],
                values: vec![VCardValue::Text(phone.clone())],
            });
        }

        if let Some(company) = &contact.company {
            entries.push(VCardEntry {
                group: None,
                name: VCardProperty::Org,
                params: vec![],
                values: vec![VCardValue::Text(company.clone())],
            });
        }

        if let Some(notes) = &contact.notes {
            entries.push(VCardEntry {
                group: None,
                name: VCardProperty::Note,
                params: vec![],
                values: vec![VCardValue::Text(notes.clone())],
            });
        }

        if let Some(birthday) = contact.birthday {
            if let Some(dt) = DateTime::from_timestamp(birthday, 0) {
                entries.push(VCardEntry {
                    group: None,
                    name: VCardProperty::Bday,
                    params: vec![],
                    values: vec![VCardValue::Text(dt.format("%Y-%m-%d").to_string())],
                });
            }
        }

        let vcard = VCard { entries };
        use std::fmt::Write;
        write!(&mut output, "{}", vcard).map_err(|e| e.to_string())?;
        exported += 1;
    }

    fs::write(&path, output).map_err(|e| e.to_string())?;

    Ok(exported)
}
