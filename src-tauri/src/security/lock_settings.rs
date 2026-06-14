use crate::db::settings::{get_setting, set_setting};

pub async fn load_settings() {
    let timeout_str = get_setting("lock_timeout_minutes").await.ok().flatten();
    let pin_hash = get_setting("lock_pin_hash").await.ok().flatten();
    let use_pass = get_setting("lock_use_encryption_password")
        .await
        .ok()
        .flatten();

    postail_security::lock::apply_settings_state(
        timeout_str
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(0) * 60,
        pin_hash.filter(|h| !h.is_empty()),
        use_pass == Some("true".to_string()),
    );
}

pub async fn set_timeout(minutes: u32) {
    postail_security::lock::apply_set_timeout_state(minutes);
    let _ = set_setting("lock_timeout_minutes", &minutes.to_string()).await;
}

pub async fn set_pin(pin: &str) -> Result<(), String> {
    let hash = postail_security::lock::hash_pin(pin)?;
    postail_security::lock::apply_set_pin_state(hash.clone());
    let _ = set_setting("lock_pin_hash", &hash).await;
    let _ = set_setting("lock_use_encryption_password", "false").await;
    Ok(())
}

pub async fn use_encryption_password() {
    postail_security::lock::apply_use_encryption_password_state();
    let _ = set_setting("lock_use_encryption_password", "true").await;
    let _ = set_setting("lock_pin_hash", "").await;
}
