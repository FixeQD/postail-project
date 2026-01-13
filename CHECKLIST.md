# Postail IMAP/SMTP Development Checklist

## IMAP Sync

- [ ] Fix MutexGuard Send issues in async commands (use tokio::sync::Mutex or spawn_blocking)
- [ ] Implement robust IMAP connection with IDLE/poll fallback
- [ ] Add mailbox fetching and caching
- [ ] Implement header fetching with pagination
- [ ] Add full message fetching with body parsing (HTML/plain, attachments)
- [ ] Handle OAuth refresh during IMAP operations
- [ ] Implement sync status tracking
- [ ] Add error recovery for network/auth failures
- [ ] Optimize for large mailboxes (virtualization, batching)

## SMTP & Outbox

- [ ] Implement SMTP sending with authentication (password/OAuth)
- [ ] Add outbox queue with retry policy (exponential backoff)
- [ ] Handle sending failures and status updates
- [ ] Integrate with IMAP to move sent emails to Sent folder
- [ ] Implement outbox worker (background task with tokio::spawn)
- [ ] Add email composition with HTML/CSS inlining
- [ ] Handle attachments in sending
- [ ] Implement cancel/retry for queued emails

## Database Integration

- [ ] Ensure IMAP data is properly stored (messages, flags, structure)
- [ ] Implement FTS search for IMAP messages
- [ ] Add outbox persistence and recovery
- [ ] Handle concurrent access (IMAP sync + SMTP sending)

## Known Issues

- [ ] MutexGuard not Send in async Tauri commands - needs refactoring
- [ ] Outbox worker not implemented (background task needed)
- [ ] Email parsing and sanitization incomplete
- [ ] Attachment streaming and caching not fully implemented
- [ ] IDLE mode not implemented for real-time sync
