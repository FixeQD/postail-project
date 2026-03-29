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
    DEFAULT_BATCH_SIZE, MessageBatchItem, MessageUpsertData, batch_insert_messages, fetch_headers,
    fetch_message_full, fetch_starred_headers, get_message_table_id, mark_read, move_to_trash,
    set_starred, toggle_starred, upsert_message,
};
