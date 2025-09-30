//! Core types for email threading and message metadata

use time::OffsetDateTime;
use serde::{Deserialize, Serialize};

/// Email message metadata for threading
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MsgMeta {
    pub uid: String,                  // server UID or stable local id
    pub folder: String,
    pub date: OffsetDateTime,
    pub from: String,
    pub subject: String,
    pub body_preview: String,         // text/plain fallback / generated snippet
    pub has_attachments: bool,
    pub is_read: bool,
    pub is_outgoing: bool,

    // Headers for threading:
    pub message_id: Option<String>,
    pub in_reply_to: Option<String>,
    pub references: Vec<String>,

    // Provider hints:
    pub server_thread_id: Option<String>, // e.g., Gmail X-GM-THRID
}

impl MsgMeta {
    pub fn new(
        uid: String,
        folder: String,
        date: OffsetDateTime,
        from: String,
        subject: String,
        body_preview: String,
    ) -> Self {
        Self {
            uid,
            folder,
            date,
            from,
            subject,
            body_preview,
            has_attachments: false,
            is_read: false,
            is_outgoing: false,
            message_id: None,
            in_reply_to: None,
            references: Vec::new(),
            server_thread_id: None,
        }
    }
}

/// Email thread representation
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Thread {
    pub id: String,                   // server_thread_id or synthetic id
    pub subject: String,              // canonical subject
    pub messages: Vec<MsgMeta>,       // sorted oldest→newest
    pub last_date: OffsetDateTime,
    pub any_unread: bool,
    pub last_is_outgoing_reply: bool,
    pub has_attachments: bool,
}

impl Thread {
    pub fn new(id: String, subject: String, messages: Vec<MsgMeta>) -> Self {
        let any_unread = messages.iter().any(|m| !m.is_read);
        let has_attachments = messages.iter().any(|m| m.has_attachments);
        let last_date = messages.last().map(|m| m.date).unwrap_or(OffsetDateTime::UNIX_EPOCH);
        let last_is_outgoing_reply = messages.last()
            .map(|m| m.is_outgoing && (m.in_reply_to.is_some() || m.subject.starts_with("Re:")))
            .unwrap_or(false);

        Self {
            id,
            subject,
            messages,
            last_date,
            any_unread,
            last_is_outgoing_reply,
            has_attachments,
        }
    }

    /// Get the last message in the thread
    pub fn last(&self) -> Option<&MsgMeta> {
        self.messages.last()
    }

    /// Get the number of messages in the thread
    pub fn count(&self) -> usize {
        self.messages.len()
    }

    /// Check if any message in the thread is unread
    pub fn any_unread(&self) -> bool {
        self.any_unread
    }

    /// Check if the thread has attachments
    pub fn has_attachments(&self) -> bool {
        self.has_attachments
    }

    /// Check if the last message is an outgoing reply
    pub fn last_is_outgoing_reply(&self) -> bool {
        self.last_is_outgoing_reply
    }
}
