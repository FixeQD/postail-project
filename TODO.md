- [x] 1 — Foundation
    - Repo, Tauri + React, logger, SQLite (WAL + FTS)

- [x] 2 — Security Core
    - Master_K (32B), OS Keyring, TPM (feature-gated), Argon2 fallback, AES-256-GCM, zeroize, tests

- [x] 3 — Account Management
    - DB, OAuth flow (local server + PKCE), encrypted creds storage, account metadata

- [ ] 4 — IMAP Basic (headers only)
    - Fetch mailboxes, headers (pagination/anchor), UIDVALIDITY handling

- [ ] 5 — Message list / indexing
    - Virtualization/list performance (react-virtuoso), FTS5 + fast queries, incremental loading

- [ ] 6 — Message fetching (body, no attachments)
    - BODYSTRUCTURE, fetch parts (text/html/plain), MIME parsing

- [ ] 7 — SMTP & Outbox
    - Save EML, outbox table, background worker, retry/backoff, append to Sent

- [ ] 8 — Attachments (download + cache)
    - Stream BODY.PEEK -> encrypted file, serve via asset://, LRU cache, sharded dirs

- [ ] 9 — Sync Loop (IDLE)
    - IDLE + poll fallback, NOOP, handle EXISTS/EXPUNGE, OAuth token refresh

- [ ] 10 — UI
    - Accounts screen, Inbox (virtual list + pagination), MailView (sanitize HTML, iframe sandbox, block remote images by default), Compose (HTML editor — Monaco/ProseMirror, paste images, CID/attachments, autosave), Attachments UI (download/manage), Settings, sync status indicators, keyboard shortcuts

- [ ] 11 — Release Prep
    - Icons, builds (.deb/.msi/AppImage), signing, CI matrix, cargo/npm audits
