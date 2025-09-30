//! Message management for Asgard Mail

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::error::{AsgardError, AsgardResult};

/// Message flags
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageFlags {
    /// Message is seen/read
    Seen,
    /// Message is answered
    Answered,
    /// Message is flagged
    Flagged,
    /// Message is deleted
    Deleted,
    /// Message is draft
    Draft,
    /// Message is recent
    Recent,
}

impl std::fmt::Display for MessageFlags {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MessageFlags::Seen => write!(f, "Seen"),
            MessageFlags::Answered => write!(f, "Answered"),
            MessageFlags::Flagged => write!(f, "Flagged"),
            MessageFlags::Deleted => write!(f, "Deleted"),
            MessageFlags::Draft => write!(f, "Draft"),
            MessageFlags::Recent => write!(f, "Recent"),
        }
    }
}

/// Message importance/priority
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageImportance {
    /// Low importance
    Low,
    /// Normal importance
    Normal,
    /// High importance
    High,
}

impl std::fmt::Display for MessageImportance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MessageImportance::Low => write!(f, "Low"),
            MessageImportance::Normal => write!(f, "Normal"),
            MessageImportance::High => write!(f, "High"),
        }
    }
}

/// Message part types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessagePartType {
    /// Text content
    Text,
    /// HTML content
    Html,
    /// Attachment
    Attachment,
    /// Embedded image
    EmbeddedImage,
    /// Other content
    Other,
}

/// Message part
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessagePart {
    /// Part ID
    pub id: String,
    /// Part type
    pub part_type: MessagePartType,
    /// MIME type
    pub mime_type: String,
    /// Content disposition
    pub disposition: Option<String>,
    /// Filename (for attachments)
    pub filename: Option<String>,
    /// Content size in bytes
    pub size: usize,
    /// Content encoding
    pub encoding: Option<String>,
    /// Content ID (for embedded images)
    pub content_id: Option<String>,
    /// Content location
    pub content_location: Option<String>,
    /// Raw content
    pub content: Option<Vec<u8>>,
    /// Child parts (for multipart messages)
    pub children: Vec<MessagePart>,
}

/// Attachment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attachment {
    /// Attachment ID
    pub id: Uuid,
    /// Message ID
    pub message_id: Uuid,
    /// Part ID
    pub part_id: String,
    /// Filename
    pub filename: String,
    /// MIME type
    pub mime_type: String,
    /// Content size in bytes
    pub size: usize,
    /// Content hash (for deduplication)
    pub content_hash: String,
    /// File path (if cached locally)
    pub file_path: Option<String>,
    /// Content
    pub content: Option<Vec<u8>>,
    /// Creation time
    pub created_at: OffsetDateTime,
}

/// Email address
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EmailAddress {
    /// Display name
    pub name: Option<String>,
    /// Email address
    pub email: String,
}

impl std::fmt::Display for EmailAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(name) = &self.name {
            write!(f, "{} <{}>", name, self.email)
        } else {
            write!(f, "{}", self.email)
        }
    }
}

/// Message headers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageHeaders {
    /// Message ID
    pub message_id: Option<String>,
    /// In-Reply-To header
    pub in_reply_to: Option<String>,
    /// References header
    pub references: Option<String>,
    /// Subject
    pub subject: String,
    /// From addresses
    pub from: Vec<EmailAddress>,
    /// To addresses
    pub to: Vec<EmailAddress>,
    /// CC addresses
    pub cc: Vec<EmailAddress>,
    /// BCC addresses
    pub bcc: Vec<EmailAddress>,
    /// Reply-To addresses
    pub reply_to: Vec<EmailAddress>,
    /// Date
    pub date: Option<OffsetDateTime>,
    /// Received date
    pub received_date: Option<OffsetDateTime>,
    /// Importance
    pub importance: MessageImportance,
    /// Custom headers
    pub custom: HashMap<String, String>,
}

/// Message represents an email message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Unique message ID
    pub id: Uuid,
    /// Account ID
    pub account_id: Uuid,
    /// Mailbox ID
    pub mailbox_id: Uuid,
    /// IMAP UID
    pub uid: Option<u32>,
    /// IMAP UID validity
    pub uid_validity: Option<u32>,
    /// Message sequence number
    pub sequence_number: Option<u32>,
    /// Message headers
    pub headers: MessageHeaders,
    /// Message flags
    pub flags: Vec<MessageFlags>,
    /// Message labels (for Gmail)
    pub labels: Vec<String>,
    /// Message parts
    pub parts: Vec<MessagePart>,
    /// Attachments
    pub attachments: Vec<Attachment>,
    /// Thread ID (for conversation threading)
    pub thread_id: Option<Uuid>,
    /// Conversation ID (for Gmail conversations)
    pub conversation_id: Option<String>,
    /// Message size in bytes
    pub size: usize,
    /// Raw message content
    pub raw_content: Option<Vec<u8>>,
    /// Creation time
    pub created_at: OffsetDateTime,
    /// Last modification time
    pub updated_at: OffsetDateTime,
    /// Last sync time
    pub last_sync: Option<OffsetDateTime>,
}

impl Message {
    /// Create a new message
    pub fn new(
        account_id: Uuid,
        mailbox_id: Uuid,
        headers: MessageHeaders,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            account_id,
            mailbox_id,
            uid: None,
            uid_validity: None,
            sequence_number: None,
            headers,
            flags: vec![],
            labels: vec![],
            parts: vec![],
            attachments: vec![],
            thread_id: None,
            conversation_id: None,
            size: 0,
            raw_content: None,
            created_at: OffsetDateTime::now_utc(),
            updated_at: OffsetDateTime::now_utc(),
            last_sync: None,
        }
    }

    /// Get the message ID
    pub fn id(&self) -> Uuid {
        self.id
    }

    /// Get the account ID
    pub fn account_id(&self) -> Uuid {
        self.account_id
    }

    /// Get the mailbox ID
    pub fn mailbox_id(&self) -> Uuid {
        self.mailbox_id
    }

    /// Get the subject
    pub fn subject(&self) -> &str {
        &self.headers.subject
    }

    /// Get the from addresses
    pub fn from(&self) -> &[EmailAddress] {
        &self.headers.from
    }

    /// Get the to addresses
    pub fn to(&self) -> &[EmailAddress] {
        &self.headers.to
    }

    /// Get the date
    pub fn date(&self) -> Option<OffsetDateTime> {
        self.headers.date
    }

    /// Check if the message is read
    pub fn is_read(&self) -> bool {
        self.flags.contains(&MessageFlags::Seen)
    }

    /// Check if the message is unread
    pub fn is_unread(&self) -> bool {
        !self.is_read()
    }

    /// Check if the message is flagged
    pub fn is_flagged(&self) -> bool {
        self.flags.contains(&MessageFlags::Flagged)
    }

    /// Check if the message is answered
    pub fn is_answered(&self) -> bool {
        self.flags.contains(&MessageFlags::Answered)
    }

    /// Check if the message is deleted
    pub fn is_deleted(&self) -> bool {
        self.flags.contains(&MessageFlags::Deleted)
    }

    /// Check if the message is a draft
    pub fn is_draft(&self) -> bool {
        self.flags.contains(&MessageFlags::Draft)
    }

    /// Check if the message has attachments
    pub fn has_attachments(&self) -> bool {
        !self.attachments.is_empty()
    }

    /// Get the number of attachments
    pub fn attachment_count(&self) -> usize {
        self.attachments.len()
    }

    /// Get the total size of attachments
    pub fn attachment_size(&self) -> usize {
        self.attachments.iter().map(|a| a.size).sum()
    }

    /// Get the first from address
    pub fn first_from(&self) -> Option<&EmailAddress> {
        self.headers.from.first()
    }

    /// Get the first to address
    pub fn first_to(&self) -> Option<&EmailAddress> {
        self.headers.to.first()
    }

    /// Get the sender email address
    pub fn sender_email(&self) -> Option<&str> {
        self.first_from().map(|addr| addr.email.as_str())
    }

    /// Get the recipient email addresses
    pub fn recipient_emails(&self) -> Vec<&str> {
        self.headers.to.iter().map(|addr| addr.email.as_str()).collect()
    }

    /// Get the display name of the sender
    pub fn sender_name(&self) -> Option<&str> {
        self.first_from().and_then(|addr| addr.name.as_deref())
    }

    /// Get the preview text (first 200 characters of text content)
    pub fn preview_text(&self) -> String {
        // Find the first text part
        for part in &self.parts {
            if part.part_type == MessagePartType::Text {
                if let Some(content) = &part.content {
                    let text = String::from_utf8_lossy(content);
                    let preview = text.chars().take(200).collect::<String>();
                    if preview.len() < text.len() {
                        return format!("{}...", preview);
                    }
                    return preview;
                }
            }
        }
        
        // Fallback to subject
        self.headers.subject.clone()
    }

    /// Get the HTML content
    pub fn html_content(&self) -> Option<&[u8]> {
        for part in &self.parts {
            if part.part_type == MessagePartType::Html {
                return part.content.as_deref();
            }
        }
        None
    }

    /// Get the text content
    pub fn text_content(&self) -> Option<&[u8]> {
        for part in &self.parts {
            if part.part_type == MessagePartType::Text {
                return part.content.as_deref();
            }
        }
        None
    }

    /// Add a flag to the message
    pub fn add_flag(&mut self, flag: MessageFlags) {
        if !self.flags.contains(&flag) {
            self.flags.push(flag);
            self.updated_at = OffsetDateTime::now_utc();
        }
    }

    /// Remove a flag from the message
    pub fn remove_flag(&mut self, flag: MessageFlags) {
        self.flags.retain(|&f| f != flag);
        self.updated_at = OffsetDateTime::now_utc();
    }

    /// Set message flags
    pub fn set_flags(&mut self, flags: Vec<MessageFlags>) {
        self.flags = flags;
        self.updated_at = OffsetDateTime::now_utc();
    }

    /// Mark the message as read
    pub fn mark_as_read(&mut self) {
        self.add_flag(MessageFlags::Seen);
    }

    /// Mark the message as unread
    pub fn mark_as_unread(&mut self) {
        self.remove_flag(MessageFlags::Seen);
    }

    /// Toggle the flagged state
    pub fn toggle_flagged(&mut self) {
        if self.is_flagged() {
            self.remove_flag(MessageFlags::Flagged);
        } else {
            self.add_flag(MessageFlags::Flagged);
        }
    }

    /// Add a label (for Gmail)
    pub fn add_label(&mut self, label: String) {
        if !self.labels.contains(&label) {
            self.labels.push(label);
            self.updated_at = OffsetDateTime::now_utc();
        }
    }

    /// Remove a label (for Gmail)
    pub fn remove_label(&mut self, label: &str) {
        self.labels.retain(|l| l != label);
        self.updated_at = OffsetDateTime::now_utc();
    }

    /// Set message labels
    pub fn set_labels(&mut self, labels: Vec<String>) {
        self.labels = labels;
        self.updated_at = OffsetDateTime::now_utc();
    }

    /// Add a message part
    pub fn add_part(&mut self, part: MessagePart) {
        self.parts.push(part);
        self.updated_at = OffsetDateTime::now_utc();
    }

    /// Add an attachment
    pub fn add_attachment(&mut self, attachment: Attachment) {
        self.attachments.push(attachment);
        self.updated_at = OffsetDateTime::now_utc();
    }

    /// Set the thread ID
    pub fn set_thread_id(&mut self, thread_id: Uuid) {
        self.thread_id = Some(thread_id);
        self.updated_at = OffsetDateTime::now_utc();
    }

    /// Set the conversation ID
    pub fn set_conversation_id(&mut self, conversation_id: String) {
        self.conversation_id = Some(conversation_id);
        self.updated_at = OffsetDateTime::now_utc();
    }

    /// Update the last sync time
    pub fn update_last_sync(&mut self) {
        self.last_sync = Some(OffsetDateTime::now_utc());
        self.updated_at = OffsetDateTime::now_utc();
    }

    /// Set the IMAP UID
    pub fn set_uid(&mut self, uid: u32, uid_validity: u32) {
        self.uid = Some(uid);
        self.uid_validity = Some(uid_validity);
        self.updated_at = OffsetDateTime::now_utc();
    }

    /// Set the sequence number
    pub fn set_sequence_number(&mut self, sequence_number: u32) {
        self.sequence_number = Some(sequence_number);
        self.updated_at = OffsetDateTime::now_utc();
    }

    /// Set the message size
    pub fn set_size(&mut self, size: usize) {
        self.size = size;
        self.updated_at = OffsetDateTime::now_utc();
    }

    /// Set the raw content
    pub fn set_raw_content(&mut self, content: Vec<u8>) {
        self.raw_content = Some(content);
        self.updated_at = OffsetDateTime::now_utc();
    }

    /// Validate the message
    pub fn validate(&self) -> AsgardResult<()> {
        if self.headers.subject.is_empty() {
            return Err(AsgardError::validation("Message subject cannot be empty"));
        }
        
        if self.headers.from.is_empty() {
            return Err(AsgardError::validation("Message must have at least one from address"));
        }
        
        if self.headers.to.is_empty() && self.headers.cc.is_empty() && self.headers.bcc.is_empty() {
            return Err(AsgardError::validation("Message must have at least one recipient"));
        }
        
        Ok(())
    }
}

/// Message thread for conversation view
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageThread {
    /// Thread ID
    pub id: Uuid,
    /// Account ID
    pub account_id: Uuid,
    /// Thread subject
    pub subject: String,
    /// Messages in the thread
    pub messages: Vec<Message>,
    /// Thread participants
    pub participants: Vec<EmailAddress>,
    /// Thread creation time
    pub created_at: OffsetDateTime,
    /// Last message time
    pub last_message_at: Option<OffsetDateTime>,
    /// Number of unread messages
    pub unread_count: u32,
    /// Thread size in bytes
    pub size: usize,
}

impl MessageThread {
    /// Create a new message thread
    pub fn new(account_id: Uuid, subject: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            account_id,
            subject,
            messages: vec![],
            participants: vec![],
            created_at: OffsetDateTime::now_utc(),
            last_message_at: None,
            unread_count: 0,
            size: 0,
        }
    }

    /// Add a message to the thread
    pub fn add_message(&mut self, message: Message) {
        self.messages.push(message);
        self.update_thread_info();
    }

    /// Update thread information
    fn update_thread_info(&mut self) {
        if let Some(last_message) = self.messages.last() {
            self.last_message_at = last_message.date();
            self.size = self.messages.iter().map(|m| m.size).sum();
            self.unread_count = self.messages.iter().filter(|m| m.is_unread()).count() as u32;
            
            // Update participants
            let mut participants = std::collections::HashSet::new();
            for message in &self.messages {
                for addr in &message.headers.from {
                    participants.insert(addr.clone());
                }
                for addr in &message.headers.to {
                    participants.insert(addr.clone());
                }
            }
            self.participants = participants.into_iter().collect();
        }
    }

    /// Get the number of messages in the thread
    pub fn message_count(&self) -> usize {
        self.messages.len()
    }

    /// Check if the thread has unread messages
    pub fn has_unread(&self) -> bool {
        self.unread_count > 0
    }

    /// Get the first message in the thread
    pub fn first_message(&self) -> Option<&Message> {
        self.messages.first()
    }

    /// Get the last message in the thread
    pub fn last_message(&self) -> Option<&Message> {
        self.messages.last()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_creation() {
        let account_id = Uuid::new_v4();
        let mailbox_id = Uuid::new_v4();
        
        let headers = MessageHeaders {
            message_id: Some("test-message-id".to_string()),
            in_reply_to: None,
            references: None,
            subject: "Test Subject".to_string(),
            from: vec![EmailAddress {
                name: Some("Test Sender".to_string()),
                email: "sender@example.com".to_string(),
            }],
            to: vec![EmailAddress {
                name: None,
                email: "recipient@example.com".to_string(),
            }],
            cc: vec![],
            bcc: vec![],
            reply_to: vec![],
            date: Some(OffsetDateTime::now_utc()),
            received_date: Some(OffsetDateTime::now_utc()),
            importance: MessageImportance::Normal,
            custom: HashMap::new(),
        };
        
        let message = Message::new(account_id, mailbox_id, headers);
        
        assert_eq!(message.subject(), "Test Subject");
        assert_eq!(message.sender_email(), Some("sender@example.com"));
        assert_eq!(message.sender_name(), Some("Test Sender"));
        assert!(message.is_unread());
        assert!(!message.is_flagged());
    }

    #[test]
    fn test_message_flags() {
        let account_id = Uuid::new_v4();
        let mailbox_id = Uuid::new_v4();
        
        let headers = MessageHeaders {
            message_id: None,
            in_reply_to: None,
            references: None,
            subject: "Test".to_string(),
            from: vec![EmailAddress {
                name: None,
                email: "test@example.com".to_string(),
            }],
            to: vec![EmailAddress {
                name: None,
                email: "test@example.com".to_string(),
            }],
            cc: vec![],
            bcc: vec![],
            reply_to: vec![],
            date: None,
            received_date: None,
            importance: MessageImportance::Normal,
            custom: HashMap::new(),
        };
        
        let mut message = Message::new(account_id, mailbox_id, headers);
        
        message.mark_as_read();
        assert!(message.is_read());
        assert!(!message.is_unread());
        
        message.toggle_flagged();
        assert!(message.is_flagged());
        
        message.toggle_flagged();
        assert!(!message.is_flagged());
    }

    #[test]
    fn test_message_thread() {
        let account_id = Uuid::new_v4();
        let thread = MessageThread::new(account_id, "Test Thread".to_string());
        
        assert_eq!(thread.subject, "Test Thread");
        assert_eq!(thread.message_count(), 0);
        assert!(!thread.has_unread());
    }

    #[test]
    fn test_message_validation() {
        let account_id = Uuid::new_v4();
        let mailbox_id = Uuid::new_v4();
        
        let headers = MessageHeaders {
            message_id: None,
            in_reply_to: None,
            references: None,
            subject: "".to_string(), // Empty subject should fail validation
            from: vec![],
            to: vec![],
            cc: vec![],
            bcc: vec![],
            reply_to: vec![],
            date: None,
            received_date: None,
            importance: MessageImportance::Normal,
            custom: HashMap::new(),
        };
        
        let message = Message::new(account_id, mailbox_id, headers);
        assert!(message.validate().is_err());
    }
}
