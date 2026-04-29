use tauri::command;
use crate::db::account::contact_groups::ContactGroup;
use crate::db::account::contacts::Contact;

#[command]
pub async fn create_contact_group(name: String, color: Option<String>) -> Result<i64, String> {
    crate::db::account::contact_groups::create_group(&name, color.as_deref())
        .await
        .map_err(|e| e.to_string())
}

#[command]
pub async fn delete_contact_group(id: i64) -> Result<(), String> {
    crate::db::account::contact_groups::delete_group(id)
        .await
        .map_err(|e| e.to_string())
}

#[command]
pub async fn rename_contact_group(id: i64, name: String) -> Result<(), String> {
    crate::db::account::contact_groups::rename_group(id, &name)
        .await
        .map_err(|e| e.to_string())
}

#[command]
pub async fn list_contact_groups() -> Result<Vec<ContactGroup>, String> {
    crate::db::account::contact_groups::list_groups()
        .await
        .map_err(|e| e.to_string())
}

#[command]
pub async fn add_contact_to_group(group_id: i64, contact_id: i64) -> Result<(), String> {
    crate::db::account::contact_groups::add_contact_to_group(group_id, contact_id)
        .await
        .map_err(|e| e.to_string())
}

#[command]
pub async fn remove_contact_from_group(group_id: i64, contact_id: i64) -> Result<(), String> {
    crate::db::account::contact_groups::remove_contact_from_group(group_id, contact_id)
        .await
        .map_err(|e| e.to_string())
}

#[command]
pub async fn get_contacts_in_group(group_id: i64) -> Result<Vec<Contact>, String> {
    crate::db::account::contact_groups::get_contacts_in_group(group_id)
        .await
        .map_err(|e| e.to_string())
}

#[command]
pub async fn get_groups_for_contact(contact_id: i64) -> Result<Vec<ContactGroup>, String> {
    crate::db::account::contact_groups::get_groups_for_contact(contact_id)
        .await
        .map_err(|e| e.to_string())
}

#[command]
pub async fn search_contact_groups(query: String) -> Result<Vec<ContactGroup>, String> {
    crate::db::account::contact_groups::search_groups(&query)
        .await
        .map_err(|e| e.to_string())
}
