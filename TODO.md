- [x] 1 — Fundament
    - Repo, Tauri + React, logger, SQLite (WAL + FTS)

- [x] 2 — Security Core
    - Master_K (32B), OS Keyring, TPM (feature-gated), Argon2 fallback, AES-256-GCM, zeroize, tests

- [ ] 3 — Account Management
    - UI + DB + OAuth flow (local server + PKCE), encrypted creds storage

- [ ] 4 — IMAP Basic (headers only)
    - Fetch mailboxes, headers with pagination (anchor/UID), UIDVALIDITY handling

- [ ] 5 — UI Listy (virtual scroll)
    - Virtualized list (react-virtuoso), efficient DB queries, incremental loading

- [ ] 6 — Mail View (body, no attachments)
    - BODYSTRUCTURE fetch, parse MIME, sanitize HTML (ammonia), sandbox iframe preview

- [ ] 7 — SMTP & Outbox (compose plain/html)
    - Compose UI, save EML, outbox table, background worker, retry/backoff, append to Sent

- [ ] 8 — Załączniki (download + cache)
    - Stream BODY.PEEK to encrypted files, serve via asset protocol, LRU cache, sharded dirs

- [ ] 9 — Sync Loop (IDLE)
    - IDLE long-poll loop, NOOP/poll fallback, handle EXISTS/EXPUNGE, OAuth token refresh

- [ ] 10 — Release Prep
    - Icons, builds (.deb/.msi/AppImage), signing, CI matrix, cargo/npm audits
