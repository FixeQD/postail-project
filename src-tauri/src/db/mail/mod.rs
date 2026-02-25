pub mod attachments;
pub mod eml_cache;
pub mod flag_queue;
pub mod mailbox;
pub mod message_bodies;
pub mod messages;

pub use super::attachments::*;
pub use super::eml_cache::*;
pub use super::flag_queue::*;
pub use super::mailbox::{fetch_mailboxes, get_mailbox_by_role, upsert_mailbox};
pub use super::message_bodies::parse_mail_with_fallback;
pub use super::messages::{
    batch_insert_messages, fetch_headers, fetch_message_full, get_message_table_id, mark_read,
    move_to_trash, upsert_message, MessageBatchItem, MessageUpsertData, DEFAULT_BATCH_SIZE,
};
