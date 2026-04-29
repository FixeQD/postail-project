use crate::globals::get_db_pool;
use crate::db::MailHeader;
use super::SnippetTarget;
use super::utils::decode_part_preview;
use futures::StreamExt;

/// Second pass: for each unique section path, batch-fetch snippet bytes an patch the in-memory headers + DB rows.
pub async fn fetch_snippets_pass2(
    session: &mut crate::imap::connection::ImapSession,
    targets: &[SnippetTarget],
    headers: &mut Vec<MailHeader>,
    account_id: &str,
    mailbox: &str,
) {
    use std::collections::HashMap;

    if targets.is_empty() {
        return;
    }

    let pool = match get_db_pool().await {
        Ok(p) => p,
        Err(e) => {
            tracing::warn!(target: "postail", "[Snippet] Failed to get DB pool: {}", e);
            return;
        }
    };

    let conn = match pool.get() {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!(target: "postail", "[Snippet] Failed to get DB connection: {}", e);
            return;
        }
    };

    // Group targets by section string so one uid_fetch covers all messages
    let mut by_section: HashMap<&str, Vec<&SnippetTarget>> = HashMap::new();
    for t in targets {
        by_section.entry(t.section.as_str()).or_default().push(t);
    }

    for (section_str, group) in &by_section {
        let uid_list = group
            .iter()
            .map(|t| t.uid.to_string())
            .collect::<Vec<_>>()
            .join(",");

        let query = format!("(UID BODY.PEEK[{}]<0.512>)", section_str);
        let mut fetches = match session.uid_fetch(&uid_list, &query).await {
            Ok(f) => f,
            Err(e) => {
                tracing::warn!(target: "postail", "[Snippet] uid_fetch failed section={} err={}", section_str, e);
                continue;
            }
        };

        while let Some(fetch) = fetches.next().await {
            let Ok(fetch) = fetch else { continue };
            let Some(uid) = fetch.uid else { continue };

            let Some(target) = group.iter().find(|t| t.uid == uid) else {
                continue;
            };

            let path_nums: Vec<u32> = section_str
                .split('.')
                .filter_map(|s| s.parse().ok())
                .collect();

            use imap_proto::types::SectionPath;
            let section_path = SectionPath::Part(path_nums, None);

            let snippet = fetch
                .section(&section_path)
                .filter(|b| !b.is_empty())
                .map(|b| decode_part_preview(b, &target.mime, &target.charset, &target.encoding))
                .filter(|s| !s.is_empty());

            if let Some(ref s) = snippet {
                if let Some(h) = headers.iter_mut().find(|h| h.uid == uid) {
                    h.snippet = Some(s.clone());
                }
                let _ = conn.execute(
                    "UPDATE messages SET snippet = ? \
                     WHERE account_id = ? AND mailbox = ? AND uid = ? \
                     AND (snippet IS NULL OR snippet = '')",
                    rusqlite::params![s, account_id, mailbox, uid],
                );
            }
        }
    }
}
