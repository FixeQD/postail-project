use tauri::command;
use serde::Serialize;
use crate::globals::get_db_pool;

#[derive(Serialize)]
pub struct ImportContactsResult {
    pub imported: u32,
    pub updated: u32,
    pub errors: u32,
}

#[command]
pub async fn import_contacts_vcf(path: String) -> Result<ImportContactsResult, String> {
    use calcard::{Entry, Parser};
    use calcard::vcard::{VCardProperty, VCardValue, VCardParameter, VCardType};
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
                let mut email = None;
                let mut name = None;
                let mut first_name = None;
                let mut middle_name = None;
                let mut last_name = None;
                let mut suffix = None;
                let mut nickname = None;
                let mut phone = None;
                let mut phone_work = None;
                let mut phone_home = None;
                let mut phone_fax = None;
                let mut work_email = None;
                let mut company = None;
                let mut job_title = None;
                let mut department = None;
                let mut role = None;
                let mut website = None;
                let mut address_home = None;
                let mut address_work = None;
                let mut notes = None;
                let mut birthday = None;
                let mut anniversary = None;
                let mut gender = None;

                for entry in vcard.entries {
                    let is_work = entry.params.iter().any(|p| 
                        matches!(p, VCardParameter::Type(v) if v.contains(&VCardType::Work))
                    );
                    let is_home = entry.params.iter().any(|p| 
                        matches!(p, VCardParameter::Type(v) if v.contains(&VCardType::Home))
                    );
                    let is_fax = entry.params.iter().any(|p| 
                        matches!(p, VCardParameter::Type(v) if v.contains(&VCardType::Fax))
                    );

                    match entry.name {
                        VCardProperty::Fn => {
                            if let Some(VCardValue::Text(s)) = entry.values.first() {
                                name = Some(s.clone());
                            }
                        }
                        VCardProperty::N => {
                            if let Some(VCardValue::Text(s)) = entry.values.first() {
                                let parts: Vec<&str> = s.split(';').collect();
                                if parts.len() > 0 && !parts[0].is_empty() { last_name = Some(parts[0].to_string()); }
                                if parts.len() > 1 && !parts[1].is_empty() { first_name = Some(parts[1].to_string()); }
                                if parts.len() > 2 && !parts[2].is_empty() { middle_name = Some(parts[2].to_string()); }
                                if parts.len() > 4 && !parts[4].is_empty() { suffix = Some(parts[4].to_string()); }
                            }
                        }
                        VCardProperty::Nickname => {
                            if let Some(VCardValue::Text(s)) = entry.values.first() {
                                nickname = Some(s.clone());
                            }
                        }
                        VCardProperty::Email => {
                            if let Some(VCardValue::Text(s)) = entry.values.first() {
                                if is_work {
                                    work_email = Some(s.clone());
                                } else if email.is_none() {
                                    email = Some(s.clone());
                                }
                            }
                        }
                        VCardProperty::Tel => {
                            if let Some(VCardValue::Text(s)) = entry.values.first() {
                                if is_work {
                                    phone_work = Some(s.clone());
                                } else if is_home {
                                    phone_home = Some(s.clone());
                                } else if is_fax {
                                    phone_fax = Some(s.clone());
                                } else if phone.is_none() {
                                    phone = Some(s.clone());
                                }
                            }
                        }
                        VCardProperty::Org => {
                            if let Some(VCardValue::Text(s)) = entry.values.first() {
                                let parts: Vec<&str> = s.split(';').collect();
                                if parts.len() > 0 && !parts[0].is_empty() { company = Some(parts[0].to_string()); }
                                if parts.len() > 1 && !parts[1].is_empty() { department = Some(parts[1].to_string()); }
                            }
                        }
                        VCardProperty::Title => {
                            if let Some(VCardValue::Text(s)) = entry.values.first() {
                                job_title = Some(s.clone());
                            }
                        }
                        VCardProperty::Role => {
                            if let Some(VCardValue::Text(s)) = entry.values.first() {
                                role = Some(s.clone());
                            }
                        }
                        VCardProperty::Url => {
                            if let Some(VCardValue::Text(s)) = entry.values.first() {
                                website = Some(s.clone());
                            }
                        }
                        VCardProperty::Adr => {
                            if let Some(VCardValue::Text(s)) = entry.values.first() {
                                if is_work {
                                    address_work = Some(s.clone());
                                } else {
                                    address_home = Some(s.clone());
                                }
                            }
                        }
                        VCardProperty::Note => {
                            if let Some(VCardValue::Text(s)) = entry.values.first() {
                                notes = Some(s.clone());
                            }
                        }
                        VCardProperty::Bday => {
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
                        VCardProperty::Anniversary => {
                            if let Some(VCardValue::Text(s)) = entry.values.first() {
                                if let Ok(date) = NaiveDate::parse_from_str(s, "%Y-%m-%d") {
                                    if let Some(dt) = date.and_hms_opt(0, 0, 0) {
                                        anniversary = Some(Utc.from_utc_datetime(&dt).timestamp());
                                    }
                                }
                            }
                        }
                        VCardProperty::Gender => {
                            if let Some(VCardValue::Text(s)) = entry.values.first() {
                                gender = Some(s.clone());
                            }
                        }
                        _ => {}
                    }
                }

                if let Some(email_val) = email.or(work_email.clone()) {
                    match crate::db::account::contacts::upsert_contact_full(
                        &conn,
                        &email_val,
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
                        None, // avatar_url
                        birthday,
                        anniversary,
                        gender.as_deref(),
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
    use calcard::vcard::{VCard, VCardEntry, VCardProperty, VCardValue, VCardParameter, VCardType};
    use chrono::DateTime;
    use std::fs;

    let pool = get_db_pool().await.map_err(|e| e.to_string())?;
    let conn = pool.get().map_err(|e| e.to_string())?;
    
    let contacts = if let Some(contact_id) = id {
        let contact = crate::db::account::contacts::get_contact_by_id(&conn, contact_id)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "Contact not found".to_string())?;
        vec![contact]
    } else {
        crate::db::account::contacts::list_contacts(&conn)
            .map_err(|e| e.to_string())?
    };

    let mut output = String::new();
    let mut exported = 0;

    for contact in contacts {
        let mut entries = Vec::new();
        
        entries.push(VCardEntry {
            group: None,
            name: VCardProperty::Fn,
            params: vec![],
            values: vec![VCardValue::Text(contact.name.clone().unwrap_or(contact.email.clone()))],
        });

        // Split N field
        if contact.first_name.is_some() || contact.last_name.is_some() {
            let mut n_val = String::new();
            n_val.push_str(contact.last_name.as_deref().unwrap_or(""));
            n_val.push(';');
            n_val.push_str(contact.first_name.as_deref().unwrap_or(""));
            n_val.push(';');
            n_val.push_str(contact.middle_name.as_deref().unwrap_or(""));
            n_val.push(';');
            n_val.push(';');
            n_val.push_str(contact.suffix.as_deref().unwrap_or(""));
            
            entries.push(VCardEntry {
                group: None,
                name: VCardProperty::N,
                params: vec![],
                values: vec![VCardValue::Text(n_val)],
            });
        }

        if let Some(nickname) = &contact.nickname {
            entries.push(VCardEntry {
                group: None,
                name: VCardProperty::Nickname,
                params: vec![],
                values: vec![VCardValue::Text(nickname.clone())],
            });
        }

        if let Some(phone) = &contact.phone {
            entries.push(VCardEntry {
                group: None,
                name: VCardProperty::Tel,
                params: vec![],
                values: vec![VCardValue::Text(phone.clone())],
            });
        }

        if let Some(phone_work) = &contact.phone_work {
            entries.push(VCardEntry {
                group: None,
                name: VCardProperty::Tel,
                params: vec![VCardParameter::Type(vec![VCardType::Work])],
                values: vec![VCardValue::Text(phone_work.clone())],
            });
        }

        if let Some(phone_home) = &contact.phone_home {
            entries.push(VCardEntry {
                group: None,
                name: VCardProperty::Tel,
                params: vec![VCardParameter::Type(vec![VCardType::Home])],
                values: vec![VCardValue::Text(phone_home.clone())],
            });
        }

        if let Some(phone_fax) = &contact.phone_fax {
            entries.push(VCardEntry {
                group: None,
                name: VCardProperty::Tel,
                params: vec![VCardParameter::Type(vec![VCardType::Fax])],
                values: vec![VCardValue::Text(phone_fax.clone())],
            });
        }

        entries.push(VCardEntry {
            group: None,
            name: VCardProperty::Email,
            params: vec![],
            values: vec![VCardValue::Text(contact.email.clone())],
        });

        if let Some(work_email) = &contact.work_email {
            entries.push(VCardEntry {
                group: None,
                name: VCardProperty::Email,
                params: vec![VCardParameter::Type(vec![VCardType::Work])],
                values: vec![VCardValue::Text(work_email.clone())],
            });
        }

        if let Some(company) = &contact.company {
            let mut org_val = company.clone();
            if let Some(dept) = &contact.department {
                org_val.push(';');
                org_val.push_str(dept);
            }
            entries.push(VCardEntry {
                group: None,
                name: VCardProperty::Org,
                params: vec![],
                values: vec![VCardValue::Text(org_val)],
            });
        }

        if let Some(title) = &contact.job_title {
            entries.push(VCardEntry {
                group: None,
                name: VCardProperty::Title,
                params: vec![],
                values: vec![VCardValue::Text(title.clone())],
            });
        }

        if let Some(role) = &contact.role {
            entries.push(VCardEntry {
                group: None,
                name: VCardProperty::Role,
                params: vec![],
                values: vec![VCardValue::Text(role.clone())],
            });
        }

        if let Some(website) = &contact.website {
            entries.push(VCardEntry {
                group: None,
                name: VCardProperty::Url,
                params: vec![],
                values: vec![VCardValue::Text(website.clone())],
            });
        }

        if let Some(addr) = &contact.address_home {
            entries.push(VCardEntry {
                group: None,
                name: VCardProperty::Adr,
                params: vec![VCardParameter::Type(vec![VCardType::Home])],
                values: vec![VCardValue::Text(addr.clone())],
            });
        }

        if let Some(addr) = &contact.address_work {
            entries.push(VCardEntry {
                group: None,
                name: VCardProperty::Adr,
                params: vec![VCardParameter::Type(vec![VCardType::Work])],
                values: vec![VCardValue::Text(addr.clone())],
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

        if let Some(anniversary) = contact.anniversary {
            if let Some(dt) = DateTime::from_timestamp(anniversary, 0) {
                entries.push(VCardEntry {
                    group: None,
                    name: VCardProperty::Anniversary,
                    params: vec![],
                    values: vec![VCardValue::Text(dt.format("%Y-%m-%d").to_string())],
                });
            }
        }

        if let Some(gender) = &contact.gender {
            entries.push(VCardEntry {
                group: None,
                name: VCardProperty::Gender,
                params: vec![],
                values: vec![VCardValue::Text(gender.clone())],
            });
        }

        let vcard = VCard { entries };
        use std::fmt::Write;
        write!(&mut output, "{}", vcard).map_err(|e| e.to_string())?;
        exported += 1;
    }

    fs::write(&path, output).map_err(|e| e.to_string())?;

    Ok(exported)
}
