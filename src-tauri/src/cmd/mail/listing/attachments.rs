use std::fs;
use tauri::command;

#[command]
pub async fn save_attachment(
    account_id: String,
    mailbox: String,
    uid: u32,
    part_id: String,
    save_path: String,
) -> Result<(), String> {
    let cache_dir = crate::utils::config::get_data_dir()
        .join("cache")
        .join(&account_id)
        .join(crate::utils::fs::safe_filename(&mailbox))
        .join(uid.to_string());

    let cache_file = cache_dir.join(format!("{}.part", part_id));

    if cache_file.exists() {
        fs::copy(&cache_file, &save_path).map_err(|e| e.to_string())?;
        return Ok(());
    }

    // Not in cache, fetch from IMAP
    let imap = crate::globals::IMAP_MANAGER.lock().await;
    let mut session = imap.connect_imap(&account_id).await?;
    session.select(&mailbox).await.map_err(|e| e.to_string())?;

    let query = format!("BODY.PEEK[{}]", part_id);

    // Fetch the attachment body
    let body = {
        let mut fetches = session
            .uid_fetch(uid.to_string(), &query)
            .await
            .map_err(|e| e.to_string())?;

        let Some(fetch) = futures::StreamExt::next(&mut fetches).await else {
            return Err("Attachment not found".to_string());
        };
        let fetch = fetch.map_err(|e| e.to_string())?;
        fetch
            .body()
            .ok_or_else(|| "Failed to fetch attachment body".to_string())?
            .to_vec()
    };

    if body.is_empty() {
        let _ = session.logout().await;
        return Err("Attachment not found".to_string());
    }

    // We need the BODYSTRUCTURE to know the encoding
    let encoding = {
        let mut bs_fetches = session
            .uid_fetch(uid.to_string(), "BODYSTRUCTURE")
            .await
            .map_err(|e| e.to_string())?;

        if let Some(bs_fetch) = futures::StreamExt::next(&mut bs_fetches).await {
            let bs_fetch = bs_fetch.map_err(|e| e.to_string())?;
            let bs = bs_fetch.bodystructure().ok_or("No bodystructure")?;

            fn find_encoding(
                bs: &imap_proto::types::BodyStructure,
                target_path: &str,
                current_path: &mut Vec<u32>,
            ) -> Option<String> {
                use imap_proto::types::{BodyStructure, ContentEncoding};

                let path_str = if current_path.is_empty() {
                    "1".to_string()
                } else {
                    current_path
                        .iter()
                        .map(|n| n.to_string())
                        .collect::<Vec<_>>()
                        .join(".")
                };

                if path_str == target_path {
                    return match bs {
                        BodyStructure::Text { other, .. }
                        | BodyStructure::Basic { other, .. } => Some(match &other.transfer_encoding {
                            ContentEncoding::Base64 => "base64".to_string(),
                            ContentEncoding::QuotedPrintable => "quoted-printable".to_string(),
                            _ => "8bit".to_string(),
                        }),
                        _ => None,
                    };
                }

                if let BodyStructure::Multipart { bodies, .. } = bs {
                    for (i, sub) in bodies.iter().enumerate() {
                        current_path.push(i as u32 + 1);
                        if let Some(enc) = find_encoding(sub, target_path, current_path) {
                            return Some(enc);
                        }
                        current_path.pop();
                    }
                }
                None
            }

            find_encoding(bs, &part_id, &mut Vec::new())
                .unwrap_or_else(|| "8bit".to_string())
        } else {
            "8bit".to_string()
        }
    };

    let decoded = match encoding.as_str() {
        "base64" => {
            use base64::Engine;
            let stripped: Vec<u8> = body
                .iter()
                .copied()
                .filter(|&b| b != b'\r' && b != b'\n')
                .collect();
            base64::engine::general_purpose::STANDARD
                .decode(stripped)
                .map_err(|e| format!("Base64 decode failed: {}", e))?
        }
        "quoted-printable" => {
            let wrapped = [
                format!("Content-Transfer-Encoding: quoted-printable\r\n\r\n").as_bytes(),
                &body,
            ]
            .concat();
            mailparse::parse_mail(&wrapped)
                .map_err(|e| e.to_string())?
                .get_body_raw()
                .map_err(|e| e.to_string())?
        }
        _ => body,
    };

    fs::write(&save_path, decoded).map_err(|e| e.to_string())?;
    let _ = session.logout().await;
    Ok(())
}
