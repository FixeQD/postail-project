use crate::db::MailHeader;
use crate::db::mail::messages::safe_timestamp_from_utc;
use crate::error::DBError;
use chrono::Utc;
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use serde_json;

#[derive(Debug, Serialize, Deserialize)]
pub struct Contact {
    pub id: i64,
    pub email: String,
    pub name: Option<String>,
    pub first_name: Option<String>,
    pub middle_name: Option<String>,
    pub last_name: Option<String>,
    pub suffix: Option<String>,
    pub nickname: Option<String>,
    pub frequency: i32,
    pub phone: Option<String>,
    pub phone_work: Option<String>,
    pub phone_home: Option<String>,
    pub phone_fax: Option<String>,
    pub work_email: Option<String>,
    pub company: Option<String>,
    pub job_title: Option<String>,
    pub department: Option<String>,
    pub role: Option<String>,
    pub website: Option<String>,
    pub address_home: Option<String>,
    pub address_work: Option<String>,
    pub notes: Option<String>,
    pub avatar_url: Option<String>,
    pub birthday: Option<i64>,
    pub anniversary: Option<i64>,
    pub gender: Option<String>,
}

pub fn upsert_contact(conn: &Connection, email: &str, name: Option<&str>) -> Result<(), DBError> {
    let now = Utc::now().timestamp();

    conn.execute(
        "INSERT INTO contacts (email, name, last_contact_at, frequency)
         VALUES (?, ?, ?, 1)
         ON CONFLICT(email) DO UPDATE SET
            name = COALESCE(excluded.name, name),
            last_contact_at = excluded.last_contact_at,
            frequency = frequency + 1",
        params![email, name, now],
    )?;

    Ok(())
}

pub fn upsert_from_address_string(conn: &Connection, address: &str) -> Result<(), DBError> {
    let address = address.trim();
    if address.is_empty() {
        return Ok(());
    }
    // Try full "Name <email>" format first, then fall back to plain email
    if let Some((name, email)) = parse_address(address) {
        upsert_contact(conn, &email, name.as_deref())?;
    } else if address.contains('@') {
        upsert_contact(conn, address, None)?;
    }
    Ok(())
}

fn parse_address(address: &str) -> Option<(Option<String>, String)> {
    let address = address.trim();
    if address.is_empty() {
        return None;
    }

    let start = address.find('<');
    let end = address.rfind('>');

    // Require both brackets to be present
    match (start, end) {
        (Some(start), Some(end)) if start < end => {
            let name = address[..start].trim().trim_matches('"').trim();
            let email = address[start + 1..end].trim();

            // Require @ in the email between brackets
            if !email.contains('@') {
                return None;
            }

            let name_opt = if name.is_empty() {
                None
            } else {
                Some(name.to_string())
            };
            Some((name_opt, email.to_string()))
        }
        // If brackets are malformed or missing, don't accept as plain email
        _ => None,
    }
}

pub fn search_contacts(
    conn: &Connection,
    query: &str,
    limit: u32,
) -> Result<Vec<Contact>, DBError> {
    let trimmed_query = query.trim();
    if trimmed_query.is_empty() {
        return Ok(Vec::new());
    }

    let mut stmt = conn.prepare(
        "SELECT id, email, name, first_name, middle_name, last_name, suffix, nickname, 
                frequency, phone, phone_work, phone_home, phone_fax, work_email,
                company, job_title, department, role, website, address_home, address_work,
                notes, avatar_url, birthday, anniversary, gender
         FROM contacts c
         JOIN contacts_fts f ON f.rowid = c.id
         WHERE contacts_fts MATCH ?
         ORDER BY c.frequency DESC, c.last_contact_at DESC
         LIMIT ?",
    )?;

    let fts_query = format!("{}*", crate::db::search::escape_fts_query(trimmed_query));
    let contact_iter = stmt.query_map(params![fts_query, limit], |row| {
        Ok(Contact {
            id: row.get(0)?,
            email: row.get(1)?,
            name: row.get(2)?,
            first_name: row.get(3)?,
            middle_name: row.get(4)?,
            last_name: row.get(5)?,
            suffix: row.get(6)?,
            nickname: row.get(7)?,
            frequency: row.get(8)?,
            phone: row.get(9)?,
            phone_work: row.get(10)?,
            phone_home: row.get(11)?,
            phone_fax: row.get(12)?,
            work_email: row.get(13)?,
            company: row.get(14)?,
            job_title: row.get(15)?,
            department: row.get(16)?,
            role: row.get(17)?,
            website: row.get(18)?,
            address_home: row.get(19)?,
            address_work: row.get(20)?,
            notes: row.get(21)?,
            avatar_url: row.get(22)?,
            birthday: row.get(23)?,
            anniversary: row.get(24)?,
            gender: row.get(25)?,
        })
    })?;

    let mut contacts = Vec::new();
    for contact in contact_iter {
        contacts.push(contact?);
    }

    Ok(contacts)
}

pub fn get_contact_by_id(conn: &Connection, id: i64) -> Result<Option<Contact>, DBError> {
    let mut stmt = conn.prepare(
        "SELECT id, email, name, first_name, middle_name, last_name, suffix, nickname, 
                frequency, phone, phone_work, phone_home, phone_fax, work_email,
                company, job_title, department, role, website, address_home, address_work,
                notes, avatar_url, birthday, anniversary, gender
         FROM contacts WHERE id = ?1"
    )?;
    
    let mut rows = stmt.query([id])?;
    if let Some(row) = rows.next()? {
        Ok(Some(Contact {
            id: row.get(0)?,
            email: row.get(1)?,
            name: row.get(2)?,
            first_name: row.get(3)?,
            middle_name: row.get(4)?,
            last_name: row.get(5)?,
            suffix: row.get(6)?,
            nickname: row.get(7)?,
            frequency: row.get(8)?,
            phone: row.get(9)?,
            phone_work: row.get(10)?,
            phone_home: row.get(11)?,
            phone_fax: row.get(12)?,
            work_email: row.get(13)?,
            company: row.get(14)?,
            job_title: row.get(15)?,
            department: row.get(16)?,
            role: row.get(17)?,
            website: row.get(18)?,
            address_home: row.get(19)?,
            address_work: row.get(20)?,
            notes: row.get(21)?,
            avatar_url: row.get(22)?,
            birthday: row.get(23)?,
            anniversary: row.get(24)?,
            gender: row.get(25)?,
        }))
    } else {
        Ok(None)
    }
}
pub fn list_contacts(conn: &Connection) -> Result<Vec<Contact>, DBError> {
    let mut stmt = conn.prepare(
        "SELECT id, email, name, first_name, middle_name, last_name, suffix, nickname, 
                frequency, phone, phone_work, phone_home, phone_fax, work_email,
                company, job_title, department, role, website, address_home, address_work,
                notes, avatar_url, birthday, anniversary, gender
         FROM contacts ORDER BY (name IS NULL AND email IS NULL), name ASC, email ASC"
    )?;
    let contact_iter = stmt.query_map([], |row| {
        Ok(Contact {
            id: row.get(0)?,
            email: row.get(1)?,
            name: row.get(2)?,
            first_name: row.get(3)?,
            middle_name: row.get(4)?,
            last_name: row.get(5)?,
            suffix: row.get(6)?,
            nickname: row.get(7)?,
            frequency: row.get(8)?,
            phone: row.get(9)?,
            phone_work: row.get(10)?,
            phone_home: row.get(11)?,
            phone_fax: row.get(12)?,
            work_email: row.get(13)?,
            company: row.get(14)?,
            job_title: row.get(15)?,
            department: row.get(16)?,
            role: row.get(17)?,
            website: row.get(18)?,
            address_home: row.get(19)?,
            address_work: row.get(20)?,
            notes: row.get(21)?,
            avatar_url: row.get(22)?,
            birthday: row.get(23)?,
            anniversary: row.get(24)?,
            gender: row.get(25)?,
        })
    })?;
    let mut contacts = Vec::new();
    for contact in contact_iter {
        contacts.push(contact?);
    }
    Ok(contacts)
}

pub fn get_contact_messages(
    conn: &Connection,
    account_id: &str,
    email: &str,
    limit: u32,
) -> Result<Vec<MailHeader>, DBError> {
    let pattern = format!("%{}%", email);
    let mut stmt = conn.prepare(
        "SELECT uid, message_id, internal_date, subject, from_addr, to_json, cc_json, flags_json, snippet, has_attachments, starred, mailbox,
         (SELECT json_group_array(tag) FROM message_tags mt WHERE mt.message_id = m.id) as tags_json
         FROM messages m
         WHERE account_id = ? AND (from_addr LIKE ? OR to_json LIKE ?)
         ORDER BY internal_date DESC
         LIMIT ?"
    )?;
    let mut rows = stmt.query(params![account_id, pattern, pattern, limit])?;
    let mut headers = Vec::new();
    while let Some(row) = rows.next()? {
        let to_json: Option<String> = row.get(5)?;
        let to: Vec<String> = to_json
            .map(|s: String| serde_json::from_str(&s).unwrap_or_default())
            .unwrap_or_default();
        let cc_json: Option<String> = row.get(6)?;
        let cc: Vec<String> = cc_json
            .map(|s: String| serde_json::from_str(&s).unwrap_or_default())
            .unwrap_or_default();
        let flags_json: Option<String> = row.get(7)?;
        let flags: Vec<String> = flags_json
            .map(|s: String| serde_json::from_str(&s).unwrap_or_default())
            .unwrap_or_default();
        let tags_json: Option<String> = row.get(12)?;
        let tags: Vec<String> = tags_json
            .map(|s: String| serde_json::from_str(&s).unwrap_or_default())
            .unwrap_or_default();
        headers.push(MailHeader {
            uid: row.get(0)?,
            mailbox: row.get(11)?,
            message_id: row.get(1)?,
            internal_date: safe_timestamp_from_utc(row.get::<_, i64>(2)?)
                .ok_or_else(|| rusqlite::Error::InvalidColumnIndex(2))?,
            subject: row.get(3)?,
            from: vec![row.get::<_, Option<String>>(4)?.unwrap_or_default()],
            to,
            cc,
            flags,
            snippet: row.get(8)?,
            has_attachments: row.get::<_, i64>(9)? != 0,
            starred: row.get::<_, i64>(10)? != 0,
            tags,
        });
    }
    Ok(headers)
}

pub fn update_contact(
    conn: &Connection,
    id: i64,
    email: &str,
    name: Option<&str>,
    first_name: Option<&str>,
    middle_name: Option<&str>,
    last_name: Option<&str>,
    suffix: Option<&str>,
    nickname: Option<&str>,
    phone: Option<&str>,
    phone_work: Option<&str>,
    phone_home: Option<&str>,
    phone_fax: Option<&str>,
    work_email: Option<&str>,
    company: Option<&str>,
    job_title: Option<&str>,
    department: Option<&str>,
    role: Option<&str>,
    website: Option<&str>,
    address_home: Option<&str>,
    address_work: Option<&str>,
    notes: Option<&str>,
    avatar_url: Option<&str>,
    birthday: Option<i64>,
    anniversary: Option<i64>,
    gender: Option<&str>,
) -> Result<(), DBError> {
    conn.execute(
        "UPDATE contacts SET
            email = ?,
            name = ?,
            first_name = ?,
            middle_name = ?,
            last_name = ?,
            suffix = ?,
            nickname = ?,
            phone = ?,
            phone_work = ?,
            phone_home = ?,
            phone_fax = ?,
            work_email = ?,
            company = ?,
            job_title = ?,
            department = ?,
            role = ?,
            website = ?,
            address_home = ?,
            address_work = ?,
            notes = ?,
            avatar_url = ?,
            birthday = ?,
            anniversary = ?,
            gender = ?
         WHERE id = ?",
        params![
            email, name, first_name, middle_name, last_name, suffix, nickname,
            phone, phone_work, phone_home, phone_fax, work_email,
            company, job_title, department, role, website,
            address_home, address_work, notes, avatar_url,
            birthday, anniversary, gender, id
        ],
    )?;
    Ok(())
}

pub fn create_contact(
    conn: &Connection,
    email: &str,
    name: Option<&str>,
    first_name: Option<&str>,
    middle_name: Option<&str>,
    last_name: Option<&str>,
    suffix: Option<&str>,
    nickname: Option<&str>,
    phone: Option<&str>,
    phone_work: Option<&str>,
    phone_home: Option<&str>,
    phone_fax: Option<&str>,
    work_email: Option<&str>,
    company: Option<&str>,
    job_title: Option<&str>,
    department: Option<&str>,
    role: Option<&str>,
    website: Option<&str>,
    address_home: Option<&str>,
    address_work: Option<&str>,
    notes: Option<&str>,
    avatar_url: Option<&str>,
    birthday: Option<i64>,
    anniversary: Option<i64>,
    gender: Option<&str>,
) -> Result<i64, DBError> {
    let now = Utc::now().timestamp();
    conn.execute(
        "INSERT INTO contacts (
            email, name, first_name, middle_name, last_name, suffix, nickname,
            phone, phone_work, phone_home, phone_fax, work_email,
            company, job_title, department, role, website,
            address_home, address_work, notes, avatar_url,
            birthday, anniversary, gender, last_contact_at, frequency
         ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 1)",
        params![
            email, name, first_name, middle_name, last_name, suffix, nickname,
            phone, phone_work, phone_home, phone_fax, work_email,
            company, job_title, department, role, website,
            address_home, address_work, notes, avatar_url,
            birthday, anniversary, gender, now
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn delete_contact(conn: &Connection, id: i64) -> Result<(), DBError> {
    conn.execute("DELETE FROM contacts WHERE id = ?", params![id])?;
    Ok(())
}

pub fn upsert_contact_full(
    conn: &Connection,
    email: &str,
    name: Option<&str>,
    first_name: Option<&str>,
    middle_name: Option<&str>,
    last_name: Option<&str>,
    suffix: Option<&str>,
    nickname: Option<&str>,
    phone: Option<&str>,
    phone_work: Option<&str>,
    phone_home: Option<&str>,
    phone_fax: Option<&str>,
    work_email: Option<&str>,
    company: Option<&str>,
    job_title: Option<&str>,
    department: Option<&str>,
    role: Option<&str>,
    website: Option<&str>,
    address_home: Option<&str>,
    address_work: Option<&str>,
    notes: Option<&str>,
    avatar_url: Option<&str>,
    birthday: Option<i64>,
    anniversary: Option<i64>,
    gender: Option<&str>,
) -> Result<bool, DBError> {
    let exists = conn.query_row("SELECT 1 FROM contacts WHERE email = ?", params![email], |_| Ok(true)).unwrap_or(false);
    let now = Utc::now().timestamp();
    
    conn.execute(
        "INSERT INTO contacts (
            email, name, first_name, middle_name, last_name, suffix, nickname,
            phone, phone_work, phone_home, phone_fax, work_email,
            company, job_title, department, role, website,
            address_home, address_work, notes, avatar_url,
            birthday, anniversary, gender, last_contact_at, frequency
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23, ?24, ?25, 1)
         ON CONFLICT(email) DO UPDATE SET
            name = COALESCE(excluded.name, name),
            first_name = COALESCE(excluded.first_name, first_name),
            middle_name = COALESCE(excluded.middle_name, middle_name),
            last_name = COALESCE(excluded.last_name, last_name),
            suffix = COALESCE(excluded.suffix, suffix),
            nickname = COALESCE(excluded.nickname, nickname),
            phone = COALESCE(excluded.phone, phone),
            phone_work = COALESCE(excluded.phone_work, phone_work),
            phone_home = COALESCE(excluded.phone_home, phone_home),
            phone_fax = COALESCE(excluded.phone_fax, phone_fax),
            work_email = COALESCE(excluded.work_email, work_email),
            company = COALESCE(excluded.company, company),
            job_title = COALESCE(excluded.job_title, job_title),
            department = COALESCE(excluded.department, department),
            role = COALESCE(excluded.role, role),
            website = COALESCE(excluded.website, website),
            address_home = COALESCE(excluded.address_home, address_home),
            address_work = COALESCE(excluded.address_work, address_work),
            notes = COALESCE(excluded.notes, notes),
            avatar_url = COALESCE(excluded.avatar_url, avatar_url),
            birthday = COALESCE(excluded.birthday, birthday),
            anniversary = COALESCE(excluded.anniversary, anniversary),
            gender = COALESCE(excluded.gender, gender)",
        params![
            email, name, first_name, middle_name, last_name, suffix, nickname,
            phone, phone_work, phone_home, phone_fax, work_email,
            company, job_title, department, role, website,
            address_home, address_work, notes, avatar_url,
            birthday, anniversary, gender, now
        ],
    )?;
    
    Ok(!exists)
}
