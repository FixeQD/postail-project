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

/// Insert a new contact or update an existing one for the given email, recording the current time and incrementing contact frequency.
///
/// If a row with the same email already exists, the function updates `name` to the provided value when present, sets `last_contact_at` to the current timestamp, and increments `frequency` by 1. If `name` is `None`, the existing name is preserved.
///
/// # Examples
///
/// ```ignore
/// // Insert a new contact with a name
/// upsert_contact(&conn, "alice@example.com", Some("Alice")).unwrap();
///
/// // Increment frequency and preserve existing name when name is None
/// upsert_contact(&conn, "alice@example.com", None).unwrap();
/// ```
///
/// Returns `Ok(())` on success or `Err(DBError)` if the database operation fails.
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

/// Parse an address string and upsert the extracted contact into the database.
///
/// If the address parses as either `"Name <email>"` or a bare email, the extracted
/// name (optional) and email are inserted or merged into the contacts table
/// (incrementing frequency and updating last_contact_at on conflict). If the
/// address cannot be parsed, the function does nothing.
///
/// # Parameters
///
/// - `address`: A display address to parse; accepted forms include `"Name <email>"`
///   or a bare email like `"user@example.com"`. Leading/trailing whitespace is trimmed.
///
/// # Returns
///
/// `Ok(())` on success, or `Err(DBError)` if a database operation fails.
///
/// # Examples
///
/// ```rust,no_run
/// # use rusqlite::Connection;
/// # use your_crate::db::contacts::upsert_from_address_string;
/// let conn: Connection = /* open or obtain a Connection */ unimplemented!();
/// upsert_from_address_string(&conn, "Alice Example <alice@example.com>").unwrap();
/// ```
pub fn upsert_from_address_string(conn: &Connection, address: &str) -> Result<(), DBError> {
    if let Some((name, email)) = parse_address(address) {
        upsert_contact(conn, &email, name.as_deref())?;
    }
    Ok(())
}

/// Parses an address string into an optional display name and an email.

///

/// The function accepts inputs like `Name <user@example.com>` or a bare email

/// such as `user@example.com`. Leading and trailing whitespace is trimmed. A

/// quoted or empty name is converted to `None`.

///

/// # Examples

///

/// ```

/// assert_eq!(

///     parse_address("Alice Example <alice@example.com>"),

///     Some((Some("Alice Example".to_string()), "alice@example.com".to_string()))

/// );

/// assert_eq!(

///     parse_address("bob@example.com"),

///     Some((None, "bob@example.com".to_string()))

/// );

/// assert_eq!(parse_address("   "), None);

/// ```
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

/// Searches contacts using a full-text search query and returns matching contacts ordered by frequency and most recent contact time.
///
/// The provided `query` is escaped for FTS and matched with a trailing wildcard; results are ordered by `frequency` (descending) and `last_contact_at` (descending), and limited to `limit`.
///
/// # Returns
///
/// A `Vec<Contact>` containing matching contacts, up to `limit` entries.
///
/// # Examples
///
/// ```
/// // given an existing rusqlite::Connection `conn`
/// let matches = search_contacts(&conn, "alice", 5).unwrap();
/// assert!(matches.len() <= 5);
/// for contact in matches {
///     println!("{} <{}>", contact.name.unwrap_or_default(), contact.email);
/// }
/// ```
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