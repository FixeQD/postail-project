// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg(target_os = "linux")]
use std::env;
#[cfg(target_os = "linux")]
use std::process::{Command, exit};

#[cfg(target_os = "linux")]
fn get_executable_path() -> std::path::PathBuf {
    if let Ok(appimage) = env::var("APPIMAGE") {
        return std::path::PathBuf::from(appimage);
    }
    env::current_exe().expect("Failed to get current executable path")
}

/// Spawn a child process with the given env vars and wait for it.
/// Yes, I'm TRYING my best to make this "software" work on your shitty system bruh.
#[cfg(target_os = "linux")]
fn try_launch(
    exe: &std::path::Path,
    args: &[String],
    env_vars: &[(&str, &str)],
    recovery_level: &str,
) -> Option<i32> {
    let mut cmd = Command::new(exe);
    cmd.args(args).env("POSTAIL_RECOVERY_MODE", recovery_level);
    for (k, v) in env_vars {
        cmd.env(k, v);
    }
    match cmd.spawn() {
        Ok(mut child) => {
            let status = child.wait().expect("Wait failed");
            status.code()
        }
        Err(e) => {
            tracing::error!("Postail: Failed to spawn process: {}", e);
            None
        }
    }
}

fn main() {
    #[cfg(target_os = "linux")]
    {
        // ── TPM helper mode ──────────────────────────────────────────
        #[cfg(feature = "tpm")]
        if env::var("POSTAIL_TPM_HELPER").is_ok() {
            match postail_project_lib::tpm_helper_init() {
                Ok(()) => {
                    tracing::info!("TPM initialized successfully");
                    exit(0);
                }
                Err(e) => {
                    tracing::error!("TPM initialization failed: {}", e);
                    exit(1);
                }
            }
        }

        let recovery_mode = env::var("POSTAIL_RECOVERY_MODE").unwrap_or_default();

        if recovery_mode.is_empty() {
            // ── Master process: cascading launcher ───────────────────
            let exe = get_executable_path();
            let args: Vec<String> = env::args().skip(1).collect();

            // Level 0 — standard launch, just disable the known-broken DMA-buf renderer
            tracing::warn!("Postail: Starting with recovery mode (level 0)...");
            let code = try_launch(&exe, &args, &[("WEBKIT_DISABLE_DMABUF_RENDERER", "1")], "0");

            if code == Some(0) {
                return;
            }
            tracing::warn!(
                "Postail: Level 0 failed (code {:?}). Trying level 1 (force X11)...",
                code
            );

            // Level 1 — force X11 backend (helps on Wayland with broken EGL)
            let code = try_launch(
                &exe,
                &args,
                &[
                    ("WEBKIT_DISABLE_DMABUF_RENDERER", "1"),
                    ("GDK_BACKEND", "x11"),
                    ("WAYLAND_DISPLAY", ""),
                ],
                "1",
            );

            if code == Some(0) {
                return;
            }
            tracing::warn!(
                "Postail: Level 1 failed (code {:?}). Trying level 2 (disable compositing)...",
                code
            );

            // Level 2 — disable WebKit compositing entirely (no GPU path at all)
            let code = try_launch(
                &exe,
                &args,
                &[
                    ("WEBKIT_DISABLE_DMABUF_RENDERER", "1"),
                    ("WEBKIT_DISABLE_COMPOSITING_MODE", "1"),
                    ("GDK_BACKEND", "x11"),
                    ("WAYLAND_DISPLAY", ""),
                ],
                "2",
            );

            if code == Some(0) {
                return;
            }
            tracing::warn!(
                "Postail: Level 2 failed (code {:?}). Trying level 3 (software rendering)...",
                code
            );

            // Level 3 — force Mesa software renderer (always works, slower)
            let code = try_launch(
                &exe,
                &args,
                &[
                    ("WEBKIT_DISABLE_DMABUF_RENDERER", "1"),
                    ("WEBKIT_DISABLE_COMPOSITING_MODE", "1"),
                    ("LIBGL_ALWAYS_SOFTWARE", "1"),
                    ("GDK_BACKEND", "x11"),
                    ("WAYLAND_DISPLAY", ""),
                    ("GALLIUM_DRIVER", "softpipe"),
                ],
                "3",
            );

            let final_code = code.unwrap_or(1);
            if final_code != 0 {
                tracing::warn!("Postail: All launch attempts failed. Please report this issue.");
                // Damn...
            }
            exit(final_code);
        }
        // else: we are a child process, fall through to run()
    }

    postail_project_lib::run();
}
