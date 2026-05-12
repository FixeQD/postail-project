use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CalendarEvent {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub location: Option<String>,
    pub start: i64, // Unix timestamp
    pub end: i64,   // Unix timestamp
    pub is_all_day: bool,
    pub calendar_name: String,
    pub color: Option<String>,
}

pub async fn list_calendar_events(start: i64, end: i64) -> Result<Vec<CalendarEvent>, String> {
    #[cfg(target_os = "windows")]
    {
        list_windows_events(start, end).await
    }
    #[cfg(target_os = "linux")]
    {
        list_linux_events(start, end).await
    }
    #[cfg(not(any(target_os = "windows", target_os = "linux")))]
    {
        Ok(vec![])
    }
}

pub async fn create_calendar_event(
    title: String,
    description: Option<String>,
    location: Option<String>,
    start: i64,
    end: i64,
    is_all_day: bool,
) -> Result<String, String> {
    #[cfg(target_os = "windows")]
    {
        create_windows_event(title, description, location, start, end, is_all_day).await
    }
    #[cfg(target_os = "linux")]
    {
        create_linux_event(title, description, location, start, end, is_all_day).await
    }
    #[cfg(not(any(target_os = "windows", target_os = "linux")))]
    {
        Err("Unsupported platform".to_string())
    }
}

#[cfg(target_os = "windows")]
async fn list_windows_events(start: i64, end: i64) -> Result<Vec<CalendarEvent>, String> {
    use windows::ApplicationModel::Appointments::{AppointmentManager, AppointmentStoreAccessType};
    use windows::Foundation::{DateTime as WinDateTime, TimeSpan};

    // Convert Unix timestamps to WinRT DateTime (100-nanosecond intervals since Jan 1, 1601)
    let win_start = WinDateTime {
        UniversalTime: (start * 10_000_000) + 116_444_736_000_000_000,
    };
    let win_end = WinDateTime {
        UniversalTime: (end * 10_000_000) + 116_444_736_000_000_000,
    };

    let store =
        AppointmentManager::RequestStoreAsync(AppointmentStoreAccessType::AllCalendarsReadOnly)
            .map_err(|e| e.to_string())?
            .await
            .map_err(|e| e.to_string())?;

    let appointments = store
        .FindAppointmentsAsync(
            win_start,
            TimeSpan {
                Duration: (win_end.UniversalTime - win_start.UniversalTime) as i64,
            },
        )
        .map_err(|e| e.to_string())?
        .await
        .map_err(|e| e.to_string())?;

    let mut events = Vec::new();
    for appointment in appointments {
        let start_time = (appointment
            .StartTime()
            .map_err(|e| e.to_string())?
            .UniversalTime
            - 116_444_736_000_000_000)
            / 10_000_000;
        let duration = appointment.Duration().map_err(|e| e.to_string())?.Duration / 10_000_000;

        events.push(CalendarEvent {
            id: appointment
                .LocalId()
                .map_err(|e| e.to_string())?
                .to_string(),
            title: appointment
                .Subject()
                .map_err(|e| e.to_string())?
                .to_string(),
            description: Some(
                appointment
                    .Details()
                    .map_err(|e| e.to_string())?
                    .to_string(),
            ),
            location: Some(
                appointment
                    .Location()
                    .map_err(|e| e.to_string())?
                    .to_string(),
            ),
            start: start_time,
            end: start_time + duration,
            is_all_day: appointment.AllDay().map_err(|e| e.to_string())?,
            calendar_name: "Windows Calendar".to_string(),
            color: None,
        });
    }

    Ok(events)
}

#[cfg(target_os = "linux")]
async fn list_linux_events(start: i64, end: i64) -> Result<Vec<CalendarEvent>, String> {
    use std::fs;

    let mut events = Vec::new();
    let home = dirs::home_dir().ok_or("Could not find home directory")?;

    // Common paths for Evolution/GNOME Calendar
    let search_paths = vec![
        home.join(".local/share/evolution/calendar/system/calendar.ics"),
        home.join(".local/share/evolution/calendar"), // Recursive search
    ];

    for path in search_paths {
        if path.is_file() {
            process_ics_file(&path, start, end, &mut events);
        } else if path.is_dir() {
            // Search for .ics files in subdirectories
            if let Ok(entries) = fs::read_dir(path) {
                for entry in entries.flatten() {
                    let p = entry.path();
                    if p.is_dir() {
                        let ics_path = p.join("calendar.ics");
                        if ics_path.exists() {
                            process_ics_file(&ics_path, start, end, &mut events);
                        }
                    }
                }
            }
        }
    }

    Ok(events)
}

#[cfg(target_os = "linux")]
fn process_ics_file(
    path: &std::path::Path,
    start_ts: i64,
    end_ts: i64,
    events: &mut Vec<CalendarEvent>,
) {
    use icalendar::{Calendar, CalendarDateTime, Component, DatePerhapsTime, EventLike};
    use std::fs;

    if let Ok(content) = fs::read_to_string(path) {
        if let Ok(calendar) = content.parse::<Calendar>() {
            for component in calendar.components {
                if let Some(event) = component.as_event() {
                    let start = match event.get_start() {
                        Some(DatePerhapsTime::DateTime(CalendarDateTime::Utc(dt))) => {
                            dt.timestamp()
                        }
                        Some(DatePerhapsTime::DateTime(CalendarDateTime::Floating(dt))) => {
                            dt.and_utc().timestamp()
                        }
                        Some(DatePerhapsTime::Date(d)) => {
                            d.and_hms_opt(0, 0, 0).unwrap().and_utc().timestamp()
                        }
                        _ => continue,
                    };

                    let end = match event.get_end() {
                        Some(DatePerhapsTime::DateTime(CalendarDateTime::Utc(dt))) => {
                            dt.timestamp()
                        }
                        Some(DatePerhapsTime::DateTime(CalendarDateTime::Floating(dt))) => {
                            dt.and_utc().timestamp()
                        }
                        Some(DatePerhapsTime::Date(d)) => {
                            d.and_hms_opt(23, 59, 59).unwrap().and_utc().timestamp()
                        }
                        _ => start + 3600,
                    };

                    if start <= end_ts && end >= start_ts {
                        events.push(CalendarEvent {
                            id: event.get_uid().unwrap_or("unknown").to_string(),
                            title: event.get_summary().unwrap_or("No Title").to_string(),
                            description: event.get_description().map(|s| s.to_string()),
                            location: event.get_location().map(|s| s.to_string()),
                            start,
                            end,
                            is_all_day: event
                                .get_start()
                                .map_or(false, |d| matches!(d, DatePerhapsTime::Date(_))),
                            calendar_name: "Local Calendar".to_string(),
                            color: None,
                        });
                    }
                }
            }
        }
    }
}

#[cfg(target_os = "windows")]
async fn create_windows_event(
    title: String,
    description: Option<String>,
    location: Option<String>,
    start: i64,
    end: i64,
    is_all_day: bool,
) -> Result<String, String> {
    use windows::ApplicationModel::Appointments::{
        Appointment, AppointmentCalendar, AppointmentManager, AppointmentStoreAccessType,
    };
    use windows::Foundation::DateTime as WinDateTime;
    use windows::Foundation::TimeSpan;

    let appointment = Appointment::new().map_err(|e| e.to_string())?;
    appointment
        .SetSubject(&windows::core::HSTRING::from(title))
        .map_err(|e| e.to_string())?;
    if let Some(desc) = description {
        appointment
            .SetDetails(&windows::core::HSTRING::from(desc))
            .map_err(|e| e.to_string())?;
    }
    if let Some(loc) = location {
        appointment
            .SetLocation(&windows::core::HSTRING::from(loc))
            .map_err(|e| e.to_string())?;
    }

    let win_start = WinDateTime {
        UniversalTime: (start * 10_000_000) + 116_444_736_000_000_000,
    };
    appointment
        .SetStartTime(win_start)
        .map_err(|e| e.to_string())?;
    appointment
        .SetDuration(TimeSpan {
            Duration: (end - start) * 10_000_000,
        })
        .map_err(|e| e.to_string())?;
    appointment
        .SetAllDay(is_all_day)
        .map_err(|e| e.to_string())?;

    let store =
        AppointmentManager::RequestStoreAsync(AppointmentStoreAccessType::AppCalendarsReadWrite)
            .map_err(|e| e.to_string())?
            .await
            .map_err(|e| e.to_string())?;

    let calendars = store
        .FindAppointmentCalendarsAsync()
        .map_err(|e| e.to_string())?
        .await
        .map_err(|e| e.to_string())?;

    let calendar = if calendars.Size().map_err(|e| e.to_string())? > 0 {
        calendars.GetAt(0).map_err(|e| e.to_string())?
    } else {
        store
            .CreateAppointmentCalendarAsync(&windows::core::HSTRING::from("Postail"))
            .map_err(|e| e.to_string())?
            .await
            .map_err(|e| e.to_string())?
    };

    calendar
        .SaveAppointmentAsync(&appointment)
        .map_err(|e| e.to_string())?
        .await
        .map_err(|e| e.to_string())?;

    Ok(appointment
        .LocalId()
        .map_err(|e| e.to_string())?
        .to_string())
}

#[cfg(target_os = "linux")]
async fn create_linux_event(
    title: String,
    description: Option<String>,
    location: Option<String>,
    start: i64,
    end: i64,
    is_all_day: bool,
) -> Result<String, String> {
    use icalendar::{Calendar, Component, Event, EventLike};
    use std::fs;
    use std::process::Command;

    let mut event = Event::new();
    event.summary(&title);
    if let Some(desc) = description {
        event.description(&desc);
    }
    if let Some(loc) = location {
        event.location(&loc);
    }

    // Set start/end
    if is_all_day {
        event.starts(
            chrono::DateTime::from_timestamp(start, 0)
                .unwrap()
                .date_naive(),
        );
        event.ends(
            chrono::DateTime::from_timestamp(end, 0)
                .unwrap()
                .date_naive(),
        );
    } else {
        event.starts(chrono::DateTime::from_timestamp(start, 0).unwrap());
        event.ends(chrono::DateTime::from_timestamp(end, 0).unwrap());
    }

    let mut calendar = Calendar::new();
    calendar.push(event);

    let tmp_dir = std::env::temp_dir();
    let file_path = tmp_dir.join(format!("event_{}.ics", uuid::Uuid::new_v4()));
    fs::write(&file_path, calendar.to_string()).map_err(|e| e.to_string())?;

    // Open with default calendar app
    Command::new("xdg-open")
        .arg(&file_path)
        .spawn()
        .map_err(|e| e.to_string())?;

    Ok("temp-ics-opened".to_string())
}
