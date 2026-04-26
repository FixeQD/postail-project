use tauri::command;
use crate::calendar::{CalendarEvent, list_calendar_events as list_events, create_calendar_event as create_event};

#[command]
pub async fn list_calendar_events(start: i64, end: i64) -> Result<Vec<CalendarEvent>, String> {
    list_events(start, end).await
}

#[command]
pub async fn create_calendar_event(
    title: String,
    description: Option<String>,
    location: Option<String>,
    start: i64,
    end: i64,
    is_all_day: bool,
) -> Result<String, String> {
    create_event(title, description, location, start, end, is_all_day).await
}
