# Postail IMAP/SMTP Development Checklist

## IMAP Sync

- [x] Fix MutexGuard Send issues in async commands (use tokio::sync::Mutex or spawn_blocking)
- [x] Implement robust IMAP connection with IDLE/poll fallback
- [x] Add mailbox fetching and caching
- [x] Implement header fetching with pagination
- [x] Add full message fetching with body parsing (HTML/plain, attachments)
- [x] Handle OAuth refresh during IMAP operations
- [x] Implement sync status tracking
- [x] Add error recovery for network/auth failures
- [ ] Optimize for large mailboxes (virtualization, batching)

## SMTP & Outbox

- [x] Implement SMTP sending with authentication (password/OAuth)
- [x] Add outbox queue with retry policy (exponential backoff)
- [x] Handle sending failures and status updates
- [x] Integrate with IMAP to move sent emails to Sent folder
- [x] Implement outbox worker (background task with tokio::spawn)
- [ ] Add email composition with HTML/CSS inlining
- [ ] Handle attachments in sending
- [ ] Implement cancel/retry for queued emails

## Database Integration

### Async & Concurrency

- [x] Replace std::sync::Mutex with tokio::sync::Mutex (DB_CONN, IMAP_MANAGER, SMTP_MANAGER)
- [x] Update .lock().unwrap() to .lock().await in all async functions
- [x] Add spawn_blocking for batch operations (messages, vacuum, backup)

### IMAP Data Storage

- [x] Implement check_uidvalidity function with mailbox resync on mismatch
- [x] Add get_mailbox_metadata (uid_validity, highest_modseq)
- [x] Implement update_highest_modseq for CONDSTORE support
- [x] Add batch_insert_messages with transaction commits every 50 messages
- [x] Implement update_message_flags with comparison optimization
- [ ] Populate has_attachments flag by checking attachments table

### Full-Text Search (FTS5)

- [x] Implement search_messages function with account/mailbox filters
- [x] Add FTS triggers (INSERT/UPDATE/DELETE) for auto-sync
- [x] Add escape_fts_query helper for user input sanitization
- [ ] Support phrase/boolean search syntax (default FTS5)

### Outbox Persistence

- [x] Implement extract_headers_from_eml using mailparse
- [x] Update enqueue_message to store subject and recipient
- [x] Add update_outbox_status and increment_outbox_attempts functions
- [x] Implement calculate_backoff (5s, 30s, 5m, 15m, 1h)
- [x] Add cleanup_old_sent_messages (delete SENT >30 days)

### Attachment Management

- [ ] Implement get_attachment_cache_path with sharded structure
- [ ] Add save_attachment function (encrypt + write + store metadata)
- [ ] Add load_attachment function (decrypt + return data + mime type)
- [ ] Implement enforce_attachment_cache_limit with LRU eviction (2GB max)

### Message Body Storage

- [x] Create message_bodies table
- [x] Implement save_message_body with ammonia sanitization
- [x] Update fetch_message_full to load actual body content
- [ ] Add lazy loading pattern for large messages

### Performance Optimization

- [x] Add indexes: messages(account_id, mailbox, uid, internal_date), outbox(status, next_retry)
- [x] Configure PRAGMA: journal_mode=WAL, synchronous=NORMAL, cache_size=64MB, mmap_size=256MB
- [x] Implement schedule_maintenance with weekly VACUUM/ANALYZE
- [x] Add WAL checkpoint scheduling

### Database Migrations

- [ ] Create migrations module with version tracking
- [ ] Implement run_migrations with automatic upgrade
- [ ] Add migration for message_bodies table
- [ ] Add migration for FTS triggers

### Backup & Recovery

- [x] Implement export_backup (partial DB + re-encrypted creds)
- [x] Implement import_backup (validate + restore + re-encrypt)
- [ ] Add backup passphrase handling (separate from master key)

### Testing

- [x] Add test_fts_search with various query patterns
- [x] Add test_concurrent_access for parallel operations
- [x] Add test_migration_up_down for migration verification
- [x] Add test_attachment_cache_lru for LRU eviction
- [x] Add test_uidvalidity_mismatch for resync logic
- [x] Add performance tests for large datasets (10k+ messages)

## Known Issues

- [x] MutexGuard not Send in async Tauri commands - migrated to tokio::sync::Mutex
- [x] Outbox worker not implemented (background task needed) - now implemented with tokio::spawn
- [ ] Email parsing and sanitization incomplete
- [ ] Attachment streaming and caching not fully implemented
- [x] IDLE mode not implemented for real-time sync - now implemented with fallback
