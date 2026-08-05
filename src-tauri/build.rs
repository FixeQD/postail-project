use std::process::Command;

fn main() {
    tauri_build::build();

    bake_dotenv_env();

    let timestamp = build_timestamp_unix();
    println!("cargo:rustc-env=BUILD_TIMESTAMP={}", timestamp);

    let git_hash = git_hash().unwrap_or_else(|| "unknown".to_string());
    println!("cargo:rustc-env=GIT_HASH={}", git_hash);

    let git_branch = git_branch().unwrap_or_else(|| "unknown".to_string());
    println!("cargo:rustc-env=GIT_BRANCH={}", git_branch);

    let profile = std::env::var("PROFILE").unwrap_or_else(|_| "unknown".to_string());
    println!("cargo:rustc-env=BUILD_PROFILE={}", profile);

    let rustc = rustc_version().unwrap_or_else(|| "unknown".to_string());
    println!("cargo:rustc-env=RUSTC_VERSION={}", rustc);

    println!("cargo:rerun-if-changed=.git/HEAD");
    println!("cargo:rerun-if-changed=.git/refs");
}

fn build_timestamp_unix() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn git_hash() -> Option<String> {
    let out = Command::new("git")
        .args(["rev-parse", "--short=8", "HEAD"])
        .output()
        .ok()?;
    if out.status.success() {
        Some(String::from_utf8_lossy(&out.stdout).trim().to_string())
    } else {
        None
    }
}

fn git_branch() -> Option<String> {
    let out = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .ok()?;
    if out.status.success() {
        Some(String::from_utf8_lossy(&out.stdout).trim().to_string())
    } else {
        None
    }
}

fn rustc_version() -> Option<String> {
    let out = Command::new("rustc").arg("--version").output().ok()?;
    if out.status.success() {
        Some(String::from_utf8_lossy(&out.stdout).trim().to_string())
    } else {
        None
    }
}

fn bake_dotenv_env() {
    let iter = match dotenvy::dotenv_iter() {
        Ok(iter) => iter,
        Err(_) => {
            println!("cargo:warning=No .env file found - no env vars baked into the binary");
            println!("cargo:rerun-if-changed=../.env");
            return;
        }
    };

    for item in iter {
        let Ok((key, value)) = item else { continue };
        if !value.trim().is_empty() {
            println!("cargo:rustc-env={key}={value}");
        }
    }

    println!("cargo:rerun-if-changed=../.env");
}
