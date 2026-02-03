// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg(target_os = "linux")]
use std::env;
#[cfg(target_os = "linux")]
use std::process::{exit, Command};

/// Performs program startup, applying Linux-specific relaunch and recovery behavior before running the application.
///
/// On Linux, when `POSTAIL_RECOVERY_MODE` is not set, the program launches a monitored child process of the same executable. If that child exits with a non‑zero status, the program prints a diagnostic and relaunches the executable in recovery mode with these environment overrides: `WEBKIT_DISABLE_DMABUF_RENDERER=1`, `GDK_BACKEND=x11`, `WAYLAND_DISPLAY=""`, and `POSTAIL_RECOVERY_MODE=1`. The process then exits with the recovery child's exit code (or 0 if unavailable). In all other cases execution proceeds to the normal application entry `postail_project_lib::run()`.
///
/// # Examples
///
/// ```no_run
/// // Simulate normal startup (no recovery mode).
/// std::env::remove_var("POSTAIL_RECOVERY_MODE");
/// main();
/// ```
fn main() {
    #[cfg(target_os = "linux")]
    {
        let recovery_mode = env::var("POSTAIL_RECOVERY_MODE").unwrap_or_default();

        if recovery_mode.is_empty() {
            // Master process: try to launch normally and monitor for issues (Case 1: Error 71, Case 2: Panic)
            let current_exe = env::current_exe().expect("Failed to get current executable path");
            let args: Vec<String> = env::args().skip(1).collect();

            let mut child = Command::new(&current_exe)
                .args(&args)
                .env("WEBKIT_DISABLE_DMABUF_RENDERER", "1")
                .env("POSTAIL_RECOVERY_MODE", "0")
                .spawn()
                .expect("Failed to spawn process");

            let status = child.wait().expect("Wait failed");

            // If the process failed (exit code != 0), it probably encountered display issues
            if !status.success() {
                eprintln!("Postail: Initial launch failed. Re-launching in recovery mode...");

                // Case 4: The Holy Trinity of Linux compatibility flags
                let mut recovery_child = Command::new(current_exe)
                    .args(&args)
                    .env("WEBKIT_DISABLE_DMABUF_RENDERER", "1")
                    .env("GDK_BACKEND", "x11")
                    .env("WAYLAND_DISPLAY", "")
                    .env("POSTAIL_RECOVERY_MODE", "1")
                    .spawn()
                    .expect("Failed to spawn recovery process");

                let final_status = recovery_child.wait().expect("Wait failed");
                exit(final_status.code().unwrap_or(0));
            }
            return;
        }
    }

    // Default path
    postail_project_lib::run();
}