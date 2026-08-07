use std::path::{Path, PathBuf};
use tauri::AppHandle;
use tauri_plugin_opener::OpenerExt;

/// Opens the operating system's default calendar application.
pub async fn open_system_calendar(app: &AppHandle) -> Result<(), String> {
    let ics_path = write_placeholder_ics().map_err(|e| e.to_string())?;
    let ics_path = ics_path.to_string_lossy().to_string();
    let opener = app.opener();
    #[cfg(target_os = "linux")]
    {
        match find_linux_calendar_binary() {
            Some(binary) => {
                return opener
                    .open_path(ics_path, Some(binary.clone()))
                    .map_err(|e| format!("Could not open {binary}: {e}"));
            }
            None => {
                return Err(
                    "No calendar app found (looked for GNOME Calendar, KOrganizer, Evolution). Install one, or set a default app for .ics files."
                        .to_string(),
                );
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        // Outlook is the closest thing to a reliable, explicitly-targetable calendar app on Windows
        if let Some(outlook) = find_windows_outlook() {
            return opener
                .open_path(ics_path, Some(outlook.to_string_lossy().into_owned()))
                .map_err(|e| format!("Could not open Outlook: {e}"));
        }

        // No Outlook found. We have no other reliable way to target a
        // specific calendar app without shelling out, so as a last resort we fall back to whatever the user has registered for .ics files
        return opener
            .open_path(ics_path, None::<String>)
            .map_err(|e| format!("Could not open your calendar app: {e}"));
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        let _ = ics_path;
        Err("Unsupported platform".to_string())
    }
}

/// Writes an empty (zero `VEVENT`) calendar file. Most calendar apps treat
/// this as "nothing to import" and simply come to the foreground rather
/// than showing an import dialog, which is what we actually want here
fn write_placeholder_ics() -> std::io::Result<PathBuf> {
    let path = std::env::temp_dir().join(format!(
        "postail-open-calendar-{}.ics",
        uuid::Uuid::new_v4()
    ));

    std::fs::write(
        &path,
        "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//Postail//Open Calendar//EN\r\nEND:VCALENDAR\r\n",
    )?;

    Ok(path)
}

/// Checks `dir` entries on `$PATH` for `binary`, without spawning anything
#[cfg(target_os = "linux")]
fn is_on_path(binary: &str) -> bool {
    let Some(path_var) = std::env::var_os("PATH") else {
        return false;
    };

    std::env::split_paths(&path_var).any(|dir| dir.join(binary).is_file())
}

/// Known Linux calendar apps, in rough order of how likely they are to be
/// the user's actual calendar rather than a side effect of some other app being installed
#[cfg(target_os = "linux")]
const LINUX_CALENDAR_CANDIDATES: &[&str] =
    &["gnome-calendar", "korganizer", "evolution", "thunderbird"];

#[cfg(target_os = "linux")]
fn find_linux_calendar_binary() -> Option<String> {
    LINUX_CALENDAR_CANDIDATES
        .iter()
        .find(|bin| is_on_path(bin))
        .map(|bin| bin.to_string())
}

/// Best-effort lookup of OUTLOOK.EXE across the install locations Office actually uses
#[cfg(target_os = "windows")]
fn find_windows_outlook() -> Option<PathBuf> {
    let mut roots = vec![];
    if let Some(pf) = std::env::var_os("ProgramFiles") {
        roots.push(PathBuf::from(pf));
    }
    if let Some(pf86) = std::env::var_os("ProgramFiles(x86)") {
        roots.push(PathBuf::from(pf86));
    }

    let office_subpaths = [
        "Microsoft Office\\root\\Office16\\OUTLOOK.EXE",
        "Microsoft Office\\Office16\\OUTLOOK.EXE",
        "Microsoft Office\\Office15\\OUTLOOK.EXE",
    ];

    for root in &roots {
        for subpath in office_subpaths {
            let candidate: PathBuf = [root.as_path(), Path::new(subpath)].iter().collect();
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }

    None
}
