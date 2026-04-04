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
    - [x] 10.6 — Editor: Implement "Source Mode" skeleton using Monaco Editor (basic mounting)
    - [x] 10.7 — Editor: Create ModeToggle UI and logic for switching between Lexical and Monaco
    - [x] 10.8 — Rust: Implement `process_email_html` command with `css-inline` basic integration
    - [x] 10.9 — Rust: Configure `ammonia` Builder with strict whitelist for email safety
    - [x] 10.10 — Rust: Add custom `attribute_filter` in Ammonia to catch and sanitize style properties
    - [x] 10.11 — Translation: Implement Lexical -> HTML serializer (using @lexical/html)
    - [x] 10.12 — Translation: Implement HTML -> Lexical deserializer for loading drafts/source mode
    - [x] 10.13 — UI: Build AddressInput component for "To" field with chip-style rendering
    - [x] 10.14 — UI: Extend AddressInput to support Cc and Bcc toggle-able fields
    - [x] 10.15 — Logic: Implement local SQLite contact suggestions for AddressInput autocomplete with FTS5
    - [x] 10.16 — UI: Create SubjectInput with auto-focus and "Tab" navigation logic
    - [x] 10.17 — DB: Implement `save_draft` Tauri command (SQL INSERT/UPDATE for drafts table)
    - [x] 10.18 — Logic: Add debounced auto-save effect to the Compose screen (30s interval)
    - [x] 10.19 — Attachments: Create `upload_attachment` command (move file to encrypted storage)
    - [x] 10.20 — UI: Build AttachmentList component with file metadata and "Remove" action
    - [x] 10.21 — Editor: Implement Drag-and-Drop file listener for the editor area
    - [x] 10.22 — Editor: Handle inline image pasting (Clipboard API -> Asset URL conversion)
    - [x] 10.23 — Logic: Generate Content-ID (CID) for inline images and update HTML references
    - [x] 10.24 — Validation: Implement Rust-side check for common CSS issues (z-index, position, etc.)
    - [x] 10.25 — UI: Create Sidebar/Floating panel for "Email Compatibility" warnings
    - [x] 10.26 — Logic: Implement "Auto-fix" button to strip problematic CSS via Rust backend (and inline CSS values)
    - [x] 10.27 — Integration: Connect "Send" button to SMTP worker and Outbox queue
    - [x] 10.28 — UI: Implement "Discard Draft" with confirmation dialog and DB cleanup
    - [x] 10.29 — UX: Add keyboard shortcuts: Ctrl+Enter (Send), Ctrl+S (Manual Save), Esc (Close)
          + Additional shortcuts: Ctrl+N (New), Ctrl+F (Search), Ctrl+R (Refresh), Ctrl+1/2/3/4 (Navigation)
          + Compose: Ctrl+Shift+A (Attach), Ctrl+K (Link), Ctrl+Shift+C/B (Toggle Cc/Bcc), Esc (Close)
          + Inbox Gmail-style: J/K (Navigate), Enter (Open), Delete/# (Trash), R/Shift+R (Reply/All), F (Forward)
          + N (New), U/Shift+U (Read/Unread), S (Star), / (Search focus)
    - [x] 10.30 — UI: Polish: Add sending animations, success/error toasts, and focus management

- [x] 11 - Flags & Labels
    - [x] 11.1 - DB: Add `starred` column to messages table (separate flag, not IMAP \Flagged)
    - [x] 11.2 - IMAP: Sync \Flagged ↔ starred both ways
    - [x] 11.3 - UI: Star button in MessageList and MessageView with animation
    - [x] 11.4 - UI: "Starred" filter in sidebar as virtual mailbox
    - [x] 11.5 - DB+UI: Per-message tags/labels system (`message_tags` table, multiple tags per message)
    - [x] 11.6 - UI: Tag picker in MessageView — add/remove tag with one click
    - [x] 11.7 - UI: Tag list in sidebar as virtual mailboxes (click = filter)
    - [x] 11.8 - DB: Migration adding `tags` and `message_tags` tables
    - [x] 11.9 - UI: Tag management in settings (name, color, delete)
    - [x] 11.10 - Rust: IMAP STORE for \Flagged when toggling star
    - [x] 11.11 - IMAP: Sync Tags ↔ Keywords

- [x] 12 - Filters & Rules
    - [x] 12.1 - DB: `filter_rules` table (conditions: from/to/subject/body, actions: move/tag/star/mark read/delete)
    - [x] 12.2 - Rust: Rules engine — run on new message sync
    - [x] 12.3 - UI: Rules editor in settings — add/edit/delete/reorder
    - [x] 12.4 - UI: Condition builder (AND/OR, field, operator, value)
    - [x] 12.5 - UI: Action builder (move to folder, add tag, star, mark read, delete)
    - [x] 12.6 - Rust: `apply_filters_to_mailbox` command — run all rules on existing messages
    - [x] 12.7 - UI: "Apply rules now" button in filter settings
    - [x] 12.8 - Rust: "Messages from sender" filter — auto-detect & suggest rule when deleting

- [ ] 13 - Folders & Organization
    - [x] 13.1 - IMAP: Create new folders (CREATE)
    - [x] 13.2 - IMAP: Rename folder (RENAME)
    - [x] 13.3 - IMAP: Delete folder (DELETE) with confirmation
    - [x] 13.4 - UI: Context menu on sidebar folder (rename/delete/create subfolder)
    - [ ] 13.5 - IMAP: Move messages between folders via drag & drop in sidebar
    - [ ] 13.6 - UI: Drag & drop messages from list to folder in sidebar
    - [ ] 13.7 - UI: "Move to..." button in MessageView with folder list
    - [ ] 13.8 - IMAP: Archive (COPY to Archive + STORE \Deleted) with one shortcut
    - [ ] 13.9 - UI: Keyboard shortcut `E` = archive (like Gmail)
    - [ ] 13.10 - IMAP: Subscribe/unsubscribe folders (SUBSCRIBE/UNSUBSCRIBE)
    - [ ] 13.11 - UI: "Hide folder" option in account settings without unsubscribing

- [ ] 14 - Advanced Search
    - [ ] 14.1 - UI: Advanced search panel (from/to/subject/body/date range/has attachment/folder)
    - [ ] 14.2 - Rust: IMAP SEARCH for server-side search when local FTS isn't enough
    - [ ] 14.3 - UI: Search operators in search field (from:, to:, subject:, before:, after:, has:attachment)
    - [ ] 14.4 - UI: Search history (last 20 queries in localStorage)
    - [ ] 14.5 - UI: Saved searches as virtual mailboxes in sidebar
    - [ ] 14.6 - DB: `saved_searches` table (name, query, icon)
    - [ ] 14.7 - UI: Highlight matching fragments in search results
    - [ ] 14.8 - Rust: Full-text search in message body via FTS5

- [ ] 15 - Templates & Signatures
    - [ ] 15.1 - DB: `signatures` table (id, account_id, name, html_content, is_default)
    - [ ] 15.2 - Rust: CRUD commands for signatures
    - [ ] 15.3 - UI: Signature editor in account settings (same Lexical editor as compose)
    - [ ] 15.4 - UI: Auto-insert default signature on new/reply messages
    - [ ] 15.5 - UI: Signature selector in ComposeScreen (dropdown near footer)
    - [ ] 15.6 - UI: Inline editing of signature in compose
    - [ ] 15.7 - DB: `templates` table (id, account_id, name, subject, html_body)
    - [ ] 15.8 - Rust: CRUD commands for templates
    - [ ] 15.9 - UI: Template gallery accessible from ComposeScreen
    - [ ] 15.10 - UI: "Save as template" from current message
    - [ ] 15.11 - UI: Insert template into compose with edit before sending
    - [ ] 15.12 - UI: Variables in templates: {{name}}, {{email}}, {{date}} with preview

- [ ] 16 - Contacts
    - [ ] 16.1 - DB: Extend `contacts` table with phone, company, notes, avatar_url, birthday
    - [ ] 16.2 - UI: Dedicated contacts screen from sidebar
    - [ ] 16.3 - UI: Contact card with message history (messages from/to)
    - [ ] 16.4 - UI: Edit contact (name, email, phone, company, note)
    - [ ] 16.5 - UI: Contact avatar — fetch from Gravatar based on email hash (optional)
    - [ ] 16.6 - UI: Import contacts from VCard (.vcf)
    - [ ] 16.7 - UI: Export contacts to VCard
    - [ ] 16.8 - Rust: Auto-create/update contacts on send (collect To/Cc)
    - [ ] 16.9 - UI: Contact groups (`contact_groups` table)
    - [ ] 16.10 - UI: Send to contact group from single To field

- [ ] 17 - Advanced Compose
    - [ ] 17.1 - UI: Reply-to field in compose (toggle under From)
    - [ ] 17.2 - UI: Message priority (High/Normal/Low) — X-Priority header + UI badge
    - [ ] 17.3 - UI: Request delivery receipt (Disposition-Notification-To — MDN exists, add for sending)
    - [ ] 17.4 - UI: Scheduled send — date picker + Rust scheduler (`scheduled_messages` table)
    - [ ] 17.5 - Rust: Worker for scheduled messages (checks every minute)
    - [ ] 17.6 - UI: Text formatting: H1/H2/H3 in Lexical toolbar
    - [ ] 17.7 - UI: Tables in editor (Lexical TablePlugin)
    - [ ] 17.8 - UI: Emoji picker (`emoji-picker-element`)
    - [ ] 17.9 - UI: Spell check — red underline via native browser spellcheck in iframe
    - [ ] 17.10 - UI: Word/character count in compose footer
    - [ ] 17.11 - UI: Paste as plain text (Ctrl+Shift+V)
    - [ ] 17.12 - UI: Format painter — copy style & apply to selection
    - [ ] 17.13 - UI: Undo/Redo history (Lexical built-in, expose via shortcuts)
    - [ ] 17.14 - UI: Preview message before sending (render like MessageView)
    - [ ] 17.15 - UI: "Send & archive" option next to Send
    - [ ] 17.16 - UI: Inline reply — reply inside quoted text (click on quote)
    - [ ] 17.17 - Rust: Validate To/Cc/Bcc emails before sending

- [ ] 18 - Virtual Mailboxes
    - [ ] 18.1 - UI: "Unread" mailbox in sidebar (query: flags NOT \Seen, all accounts)
    - [ ] 18.2 - UI: "Today" — messages from last 24h from all mailboxes
    - [ ] 18.3 - UI: "Important" — starred from all accounts
    - [ ] 18.4 - UI: "Messages with attachments" — filter has_attachments=1
    - [ ] 18.5 - UI: "Sent" — aggregate Sent from all accounts
    - [ ] 18.6 - UI: Unified Inbox — all Inboxes in one list
    - [ ] 18.7 - DB: `virtual_mailboxes` table (id, name, icon, query_json, sort_order)
    - [ ] 18.8 - UI: Create custom virtual mailboxes with query builder

- [ ] 19 - Notifications
    - [ ] 19.1 - UI: Group notifications per account (not one per message)
    - [ ] 19.2 - UI: Notification with sender preview & snippet
    - [ ] 19.3 - UI: Click notification = open message
    - [ ] 19.4 - UI: Do Not Disturb — silence hours e.g. 22:00-08:00
    - [ ] 19.5 - UI: DND exceptions — selected contacts always notify (VIP list)
    - [ ] 19.6 - UI: Folder notification filter — e.g., Inbox only, not Spam
    - [ ] 19.7 - UI: Notification sound — pick tone or mute
    - [ ] 19.8 - Rust: Badge unread count on taskbar icon (Tauri `set_badge_count`)

- [ ] 20 - Security & Extensions
    - [ ] 20.1 - UI: SPF/DKIM/DMARC indicator in message header (parse Authentication-Results)
    - [ ] 20.2 - Rust: Authentication-Results parser with pass/fail/none color code
    - [ ] 20.3 - UI: Warning on first message from new sender
    - [ ] 20.4 - UI: Phishing detection — URL in content different from displayed link
    - [ ] 20.5 - Rust: TLS cert validation on IMAP/SMTP (optional pinning)
    - [ ] 20.6 - UI: "Paranoid" mode — block all external resources, no per-session override
    - [ ] 20.7 - UI: Login history — when & where app connected (`connection_log` table)
    - [ ] 20.8 - Rust: Encrypt attachments in cache per-key (one key per file, not master key)
    - [ ] 20.9 - UI: Export master key (backup) to password-protected encrypted file
    - [ ] 20.10 - UI: Virtual keyboard for Argon2 password field (anti-keylogger, optional)

- [ ] 21 - Offline & Sync
    - [ ] 21.1 - Rust: Offline mode — detect no network & switch to cache
    - [ ] 21.2 - UI: Network status in StatusBar (online/offline/reconnecting)
    - [ ] 21.3 - Rust: Offline operations queue (move/delete/mark) executed on reconnect
    - [ ] 21.4 - DB: `offline_queue` table (operation, account_id, params_json, created_at)
    - [ ] 21.5 - Rust: Exponential backoff on IMAP reconnect (not instant retries)
    - [ ] 21.6 - Rust: Prefetch next N messages in background after opening Inbox
    - [ ] 21.7 - Rust: Full history sync on first account setup (bulk fetch)
    - [ ] 21.8 - UI: Progress bar during first account sync

- [ ] 22 - Import / Export
    - [ ] 22.1 - Rust: Import messages from .eml file
    - [ ] 22.2 - Rust: Import mailbox from mbox format (`.mbox`)
    - [ ] 22.3 - Rust: Export selected messages to .eml
    - [ ] 22.4 - Rust: Export folder to mbox
    - [ ] 22.5 - Rust: Export message to PDF (headless WebKit print)
    - [ ] 22.6 - UI: Import/export options in settings
    - [ ] 22.7 - Rust: Import contacts from CSV (name, email, phone)

- [ ] 23 - Appearance & Personalization
    - [ ] 23.1 - UI: Message list density — Compact / Normal / Relaxed
    - [ ] 23.2 - UI: Message preview pane — horizontal or vertical like Thunderbird
    - [ ] 23.3 - UI: Columns in message list — hide/show (snippet, date, size)
    - [ ] 23.4 - UI: Sort message list — by date, sender, subject, size
    - [ ] 23.5 - UI: Group messages by date (Today, Yesterday, This week, Older)
    - [ ] 23.6 - UI: Custom fonts for message body (Inter, Mono, System)
    - [ ] 23.7 - UI: Font size in message body — slider 12–20px
    - [ ] 23.8 - UI: Custom CSS for email body (power user override)
    - [ ] 23.9 - UI: More color themes (not just accent color — preset palettes)
    - [ ] 23.10 - UI: High contrast mode (accessibility)
    - [ ] 23.11 - UI: Animations — full control (disable all / transitions only / everything)

- [ ] 24 - Release Prep (extended)
    - [ ] 24.1 - CI: GitHub Actions matrix (Linux x64, Windows x64, macOS arm64)
    - [ ] 24.2 - CI: `cargo audit` + `npm audit` in pipeline before build
    - [ ] 24.3 - CI: Auto integration tests for html_transpiler on PR
    - [ ] 24.4 - Packaging: .deb with GPG signature
    - [ ] 24.5 - Packaging: AppImage with update URL
    - [ ] 24.6 - Packaging: .msi for Windows (self-signed initially)
    - [ ] 24.7 - Packaging: .dmg for macOS (Apple notarization)
    - [ ] 24.8 - UI: "What's New" screen on first launch after update (changelog overlay)
    - [ ] 24.9 - Rust: Auto-updater (Tauri plugin) with signed manifest
    - [ ] 24.10 - UI: "Check for updates" button in AboutSettings
    - [ ] 24.11 - Docs: README with screenshots, features list, build instructions
    - [ ] 24.12 - Docs: SECURITY.md with responsible disclosure process
