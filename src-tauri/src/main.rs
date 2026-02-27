// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg(target_os = "linux")]
use std::env;
#[cfg(target_os = "linux")]
use std::process::{exit, Command};

#[cfg(target_os = "linux")]
fn get_executable_path() -> std::path::PathBuf {
    if let Ok(appimage) = env::var("APPIMAGE") {
        return std::path::PathBuf::from(appimage);
    }
    env::current_exe().expect("Failed to get current executable path")
}

fn main() {
    #[cfg(target_os = "linux")]
    {
        if env::var("POSTAIL_TPM_HELPER").is_ok() {
            // Helper mode: just initialize TPM and exit
            match postail_project_lib::tpm_helper_init() {
                Ok(()) => {
                    eprintln!("TPM initialized successfully");
                    exit(0);
                }
                Err(e) => {
                    eprintln!("TPM initialization failed: {}", e);
                    exit(1);
                }
            }
        }

        let recovery_mode = env::var("POSTAIL_RECOVERY_MODE").unwrap_or_default();

        if recovery_mode.is_empty() {
            // Master process: try to launch normally and monitor for issues (Case 1: Error 71, Case 2: Panic)
            let current_exe = get_executable_path();
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
