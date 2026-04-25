use crate::error::DBError;
use crate::globals::get_db_pool;
use chrono::Utc;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use super::contacts::Contact;

#[derive(Debug, Serialize, Deserialize)]
pub struct ContactGroup {
    pub id: i64,
    pub name: String,
    pub color: Option<String>,
    pub created_at: i64,
    pub member_count: i32,
}

pub async fn create_group(name: &str, color: Option<&str>) -> Result<i64, DBError> {
    let pool = get_db_pool().await?;
    let conn = pool.get()?;
    let now = Utc::now().timestamp();

    conn.execute(
        "INSERT INTO contact_groups (name, color, created_at) VALUES (?, ?, ?)",
        params![name, color, now],
    )?;

    Ok(conn.last_insert_rowid())
}

pub async fn delete_group(id: i64) -> Result<(), DBError> {
    let pool = get_db_pool().await?;
    let conn = pool.get()?;
    conn.execute("DELETE FROM contact_groups WHERE id = ?", params![id])?;
    Ok(())
}

pub async fn rename_group(id: i64, name: &str) -> Result<(), DBError> {
    let pool = get_db_pool().await?;
    let conn = pool.get()?;
    conn.execute(
        "UPDATE contact_groups SET name = ? WHERE id = ?",
        params![name, id],
    )?;
    Ok(())
}

pub async fn list_groups() -> Result<Vec<ContactGroup>, DBError> {
    let pool = get_db_pool().await?;
    let conn = pool.get()?;
    let mut stmt = conn.prepare(
        "SELECT id, name, color, created_at, 
         (SELECT COUNT(*) FROM contact_group_members WHERE group_id = contact_groups.id) as member_count
         FROM contact_groups 
         ORDER BY name ASC",
    )?;

    let rows = stmt.query_map([], |row| {
        Ok(ContactGroup {
            id: row.get(0)?,
            name: row.get(1)?,
            color: row.get(2)?,
            created_at: row.get(3)?,
            member_count: row.get(4)?,
        })
    })?;

    let mut groups = Vec::new();
    for row in rows {
        groups.push(row?);
    }
    Ok(groups)
}

pub async fn add_contact_to_group(group_id: i64, contact_id: i64) -> Result<(), DBError> {
    let pool = get_db_pool().await?;
    let conn = pool.get()?;
    conn.execute(
        "INSERT OR IGNORE INTO contact_group_members (group_id, contact_id) VALUES (?, ?)",
        params![group_id, contact_id],
    )?;
    Ok(())
}

pub async fn remove_contact_from_group(group_id: i64, contact_id: i64) -> Result<(), DBError> {
    let pool = get_db_pool().await?;
    let conn = pool.get()?;
    conn.execute(
        "DELETE FROM contact_group_members WHERE group_id = ? AND contact_id = ?",
        params![group_id, contact_id],
    )?;
    Ok(())
}

pub async fn get_contacts_in_group(group_id: i64) -> Result<Vec<Contact>, DBError> {
    let pool = get_db_pool().await?;
    let conn = pool.get()?;
    let mut stmt = conn.prepare(
        "SELECT id, email, name, frequency, phone, company, notes, avatar_url, birthday 
         FROM contacts 
         INNER JOIN contact_group_members ON contacts.id = contact_group_members.contact_id 
         WHERE contact_group_members.group_id = ? 
         ORDER BY name ASC, email ASC",
    )?;

    let rows = stmt.query_map(params![group_id], |row| {
        Ok(Contact {
            id: row.get(0)?,
            email: row.get(1)?,
            name: row.get(2)?,
            frequency: row.get(3)?,
            phone: row.get(4)?,
            company: row.get(5)?,
            notes: row.get(6)?,
            avatar_url: row.get(7)?,
            birthday: row.get(8)?,
        })
    })?;

    let mut contacts = Vec::new();
    for row in rows {
        contacts.push(row?);
    }
    Ok(contacts)
}

pub async fn get_groups_for_contact(contact_id: i64) -> Result<Vec<ContactGroup>, DBError> {
    let pool = get_db_pool().await?;
    let conn = pool.get()?;
    let mut stmt = conn.prepare(
        "SELECT g.id, g.name, g.color, g.created_at, 
         (SELECT COUNT(*) FROM contact_group_members WHERE group_id = g.id) as member_count
         FROM contact_groups g
         INNER JOIN contact_group_members m ON g.id = m.group_id
         WHERE m.contact_id = ?
         ORDER BY g.name ASC",
    )?;

    let rows = stmt.query_map(params![contact_id], |row| {
        Ok(ContactGroup {
            id: row.get(0)?,
            name: row.get(1)?,
            color: row.get(2)?,
            created_at: row.get(3)?,
            member_count: row.get(4)?,
        })
    })?;

    let mut groups = Vec::new();
    for row in rows {
        groups.push(row?);
    }
    Ok(groups)
}
