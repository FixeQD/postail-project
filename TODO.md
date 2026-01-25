- [x] 1 — Foundation
    - Repo, Tauri + React, logger, SQLite (WAL + FTS)

- [x] 2 — Security Core
    - Master_K (32B), OS Keyring, TPM (feature-gated), Argon2 fallback, AES-256-GCM, zeroize, tests

- [x] 3 — Account Management
    - DB, OAuth flow (local server + PKCE), encrypted creds storage, account metadata

- [x] 4 — IMAP Basic (headers only)
    - Fetch mailboxes, headers (pagination/anchor), UIDVALIDITY handling

- [x] 5 — Message list / indexing
    - Virtualization/list performance (react-virtuoso), FTS5 + fast queries, incremental loading

- [x] 6 — Message fetching (body, no attachments)
    - BODYSTRUCTURE, fetch parts (text/html/plain), MIME parsing

- [x] 7 — SMTP & Outbox
    - Save EML, outbox table, background worker, retry/backoff, append to Sent

- [x] 8 — Attachments (download + cache)
    - Stream BODY.PEEK -> encrypted file, serve via asset://, LRU cache, sharded dirs

- [x] 9 — Sync Loop (IDLE)
    - IDLE + poll fallback, NOOP, handle EXISTS/EXPUNGE, OAuth token refresh

- [ ] 10 — UI
    - Accounts screen, Inbox (virtual list + pagination), MailView (sanitize HTML, iframe sandbox, block remote images by default), Compose (HTML editor — Monaco/ProseMirror, paste images, CID/attachments, autosave), Attachments UI (download/manage), Settings, sync status indicators, keyboard shortcuts

    Compose subpoints:
    - [x] 10.1 — UI: Create ComposeScreen modal/view container with header and basic layout
    - [x] 10.2 — State: Set up Zustand/Redux store for draft state (recipients, subject, body)
    - [x] 10.3 — Editor: Basic Lexical setup with RichTextPlugin and ContentEditable
    - [x] 10.4 — Editor: Implement Toolbar component with Bold, Italic, Underline, and Strikethrough (plus active states, lists & link support)
        - [x] 10.5 — Editor: Add ListPlugin (Ordered/Unordered) and LinkPlugin support
    - [ ] 10.6 — Editor: Implement "Source Mode" skeleton using Monaco Editor (basic mounting)
    - [ ] 10.7 — Editor: Create ModeToggle UI and logic for switching between Lexical and Monaco
    - [x] 10.8 — Rust: Implement `process_email_html` command with `css-inline` basic integration
    - [x] 10.9 — Rust: Configure `ammonia` Builder with strict whitelist for email safety
    - [ ] 10.10 — Rust: Add custom `attribute_filter` in Ammonia to catch and sanitize style properties
    - [ ] 10.11 — Translation: Implement Lexical -> HTML serializer (using @lexical/html)
    - [ ] 10.12 — Translation: Implement HTML -> Lexical deserializer for loading drafts/source mode
    - [ ] 10.13 — UI: Build AddressInput component for "To" field with chip-style rendering
    - [ ] 10.14 — UI: Extend AddressInput to support Cc and Bcc toggle-able fields
    - [ ] 10.15 — Logic: Implement local SQLite contact suggestions for AddressInput autocomplete
    - [ ] 10.16 — UI: Create SubjectInput with auto-focus and "Tab" navigation logic
    - [x] 10.17 — DB: Implement `save_draft` Tauri command (SQL INSERT/UPDATE for drafts table)
    - [x] 10.18 — Logic: Add debounced auto-save effect to the Compose screen (30s interval)
    - [ ] 10.19 — Attachments: Create `upload_attachment` command (move file to encrypted storage)
    - [ ] 10.20 — UI: Build AttachmentList component with file metadata and "Remove" action
    - [ ] 10.21 — Editor: Implement Drag-and-Drop file listener for the editor area
    - [ ] 10.22 — Editor: Handle inline image pasting (Clipboard API -> Asset URL conversion)
    - [ ] 10.23 — Logic: Generate Content-ID (CID) for inline images and update HTML references
    - [ ] 10.24 — Validation: Implement Rust-side check for common CSS issues (z-index, position, etc.)
    - [ ] 10.25 — UI: Create Sidebar/Floating panel for "Email Compatibility" warnings
    - [ ] 10.26 — Logic: Implement "Auto-fix" button to strip problematic CSS via Rust backend
    - [ ] 10.27 — Integration: Connect "Send" button to SMTP worker and Outbox queue
    - [ ] 10.28 — UI: Implement "Discard Draft" with confirmation dialog and DB cleanup
    - [ ] 10.29 — UX: Add keyboard shortcuts: Ctrl+Enter (Send), Ctrl+S (Manual Save), Esc (Close)
    - [ ] 10.30 — UI: Polish: Add sending animations, success/error toasts, and focus management

- [ ] 11 — Release Prep
    - Icons, builds (.deb/.msi/AppImage), signing, CI matrix, cargo/npm audits
