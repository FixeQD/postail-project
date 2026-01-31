use crate::error::DBError;
use chrono::Utc;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Contact {
    pub id: i64,
    pub email: String,
    pub name: Option<String>,
    pub frequency: i32,
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
    if let Some((name, email)) = parse_address(address) {
        upsert_contact(conn, &email, name.as_deref())?;
    }
    Ok(())
}

fn parse_address(address: &str) -> Option<(Option<String>, String)> {
    let address = address.trim();
    if address.is_empty() {
        return None;
    }

    if let (Some(start), Some(end)) = (address.find('<'), address.find('>')) {
        let name = address[..start].trim().trim_matches('"').trim();
        let email = address[start + 1..end].trim();

        let name_opt = if name.is_empty() {
            None
        } else {
            Some(name.to_string())
        };
        return Some((name_opt, email.to_string()));
    }

    if address.contains('@') {
        return Some((None, address.to_string()));
    }

    None
}

pub fn search_contacts(
    conn: &Connection,
    query: &str,
    limit: u32,
) -> Result<Vec<Contact>, DBError> {
    let mut stmt = conn.prepare(
        "SELECT c.id, c.email, c.name, c.frequency 
         FROM contacts c
         JOIN contacts_fts f ON f.rowid = c.id
         WHERE contacts_fts MATCH ? 
         ORDER BY c.frequency DESC, c.last_contact_at DESC 
         LIMIT ?",
    )?;

    let fts_query = format!("{}*", crate::db::search::escape_fts_query(query));
    let contact_iter = stmt.query_map(params![fts_query, limit], |row| {
        Ok(Contact {
            id: row.get(0)?,
            email: row.get(1)?,
            name: row.get(2)?,
            frequency: row.get(3)?,
        })
    })?;

    let mut contacts = Vec::new();
    for contact in contact_iter {
        contacts.push(contact?);
    }

    Ok(contacts)
}
