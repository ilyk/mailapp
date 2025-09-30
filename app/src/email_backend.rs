//! Simplified email backend for Asgard Mail

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use time::OffsetDateTime;
use uuid::Uuid;

/// Email account configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailAccount {
    pub id: Uuid,
    pub email: String,
    pub display_name: String,
    pub imap_server: String,
    pub imap_port: u16,
    pub smtp_server: String,
    pub smtp_port: u16,
    pub username: String,
    pub password: String,
    pub use_tls: bool,
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Clone)]
pub struct GnomeEmailAccount {
    pub id: String,
    pub provider: String,
    pub email: String,
    pub identity: String,
    pub imap_host: String,
    pub imap_port: u16,
    pub imap_use_ssl: bool,
    pub smtp_host: String,
    pub smtp_port: u16,
    pub smtp_use_tls: bool,
}

impl EmailAccount {
    pub fn new(
        email: String,
        display_name: String,
        imap_server: String,
        imap_port: u16,
        smtp_server: String,
        smtp_port: u16,
        username: String,
        password: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            email,
            display_name,
            imap_server,
            imap_port,
            smtp_server,
            smtp_port,
            username,
            password,
            use_tls: true,
            created_at: OffsetDateTime::now_utc(),
        }
    }
}

/// Email message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailMessage {
    pub id: Uuid,
    pub account_id: Uuid,
    pub uid: Option<u32>,
    pub subject: String,
    pub from: String,
    pub to: Vec<String>,
    pub cc: Vec<String>,
    pub bcc: Vec<String>,
    pub date: OffsetDateTime,
    pub body_text: String,
    pub body_html: Option<String>,
    pub is_read: bool,
    pub is_flagged: bool,
    pub mailbox: String,
    pub size: usize,
    
    // Threading headers
    pub message_id: Option<String>,
    pub in_reply_to: Option<String>,
    pub references: Vec<String>,
    pub server_thread_id: Option<String>,
    pub has_attachments: bool,
}

impl EmailMessage {
    pub fn new(
        account_id: Uuid,
        subject: String,
        from: String,
        to: Vec<String>,
        body_text: String,
        mailbox: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            account_id,
            uid: None,
            subject,
            from,
            to,
            cc: vec![],
            bcc: vec![],
            date: OffsetDateTime::now_utc(),
            body_text,
            body_html: None,
            is_read: false,
            is_flagged: false,
            mailbox,
            size: 0,
            message_id: None,
            in_reply_to: None,
            references: vec![],
            server_thread_id: None,
            has_attachments: false,
        }
    }

}

/// Mailbox
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mailbox {
    pub id: Uuid,
    pub account_id: Uuid,
    pub name: String,
    pub display_name: String,
    pub message_count: u32,
    pub unread_count: u32,
}

impl Mailbox {
    pub fn new(account_id: Uuid, name: String, display_name: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            account_id,
            name,
            display_name,
            message_count: 0,
            unread_count: 0,
        }
    }
}

/// Email backend manager
pub struct EmailBackend {
    accounts: HashMap<Uuid, EmailAccount>,
    mailboxes: HashMap<Uuid, Vec<Mailbox>>,
    messages: HashMap<Uuid, Vec<EmailMessage>>,
}

impl EmailBackend {
    pub fn new() -> Self {
        Self {
            accounts: HashMap::new(),
            mailboxes: HashMap::new(),
            messages: HashMap::new(),
        }
    }

    /// Add an email account
    pub fn add_account(&mut self, account: EmailAccount) -> Uuid {
        let account_id = account.id;
        self.accounts.insert(account_id, account);
        self.mailboxes.insert(account_id, vec![]);
        self.messages.insert(account_id, vec![]);
        account_id
    }

    /// Add an account from GNOME Online Accounts
    pub fn add_gnome_account(&mut self, gnome_account: GnomeEmailAccount) -> Uuid {
        let account_id = Uuid::new_v4();
        let email = gnome_account.email.clone();
        let account = EmailAccount {
            id: account_id,
            email: gnome_account.email,
            display_name: gnome_account.identity,
            imap_server: gnome_account.imap_host,
            imap_port: gnome_account.imap_port,
            smtp_server: gnome_account.smtp_host,
            smtp_port: gnome_account.smtp_port,
            username: email, // Use email as username
            password: "".to_string(), // GOA handles authentication
            use_tls: gnome_account.smtp_use_tls,
            created_at: OffsetDateTime::now_utc(),
        };
        
        self.accounts.insert(account_id, account);
        self.mailboxes.insert(account_id, vec![]);
        self.messages.insert(account_id, vec![]);
        
        // Create default mailboxes for this account
        self.create_default_mailboxes(account_id);
        
        account_id
    }

    /// Get all accounts
    pub fn get_accounts(&self) -> Vec<&EmailAccount> {
        self.accounts.values().collect()
    }

    /// Get mailboxes for an account
    pub fn get_mailboxes(&self, account_id: Uuid) -> Vec<&Mailbox> {
        self.mailboxes.get(&account_id).map(|v| v.iter().collect()).unwrap_or_default()
    }

    /// Get messages for a mailbox
    pub fn get_messages(&self, account_id: Uuid, mailbox_name: &str) -> Vec<&EmailMessage> {
        self.messages
            .get(&account_id)
            .map(|messages| {
                messages
                    .iter()
                    .filter(|msg| msg.mailbox == mailbox_name)
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get a specific message by ID
    pub fn get_message(&self, message_id: Uuid) -> Result<EmailMessage, String> {
        for messages in self.messages.values() {
            for message in messages {
                if message.id == message_id {
                    return Ok(message.clone());
                }
            }
        }
        Err("Message not found".to_string())
    }

    /// Create default mailboxes for an account
    pub fn create_default_mailboxes(&mut self, account_id: Uuid) {
        let mailboxes = vec![
            Mailbox::new(account_id, "INBOX".to_string(), "Inbox".to_string()),
            Mailbox::new(account_id, "Sent".to_string(), "Sent".to_string()),
            Mailbox::new(account_id, "Drafts".to_string(), "Drafts".to_string()),
            Mailbox::new(account_id, "Trash".to_string(), "Trash".to_string()),
            Mailbox::new(account_id, "Spam".to_string(), "Spam".to_string()),
        ];

        self.mailboxes.insert(account_id, mailboxes);
    }

    /// Add comprehensive test messages for threading demo
    pub fn add_sample_messages(&mut self, account_id: Uuid) {
        use time::{Date, Month, Time, UtcOffset, OffsetDateTime};
        
        // Create default mailboxes first
        self.create_default_mailboxes(account_id);
        
        // ===== COMPREHENSIVE TEST DATA =====
        
        // 1. THREAD WITH 3+ MESSAGES (All Read)
        // Thread A: Team Discussion - 4 messages, all read
        let mut thread_a_msg1 = EmailMessage::new(
            account_id,
            "Team Meeting Tomorrow".to_string(),
            "manager@company.com".to_string(),
            vec![self.accounts.get(&account_id).unwrap().email.clone(), "team@company.com".to_string()],
            "Hi team,\n\nJust a reminder that we have our weekly team meeting tomorrow at 10 AM. Please prepare your status updates and any blockers you're facing.\n\nAgenda:\n- Project status updates\n- Sprint planning\n- Q&A session\n\nSee you all there!\n\nBest,\nManager".to_string(),
            "INBOX".to_string(),
        );
        thread_a_msg1.date = OffsetDateTime::new_in_offset(
            Date::from_calendar_date(2024, Month::March, 10).unwrap(),
            Time::from_hms(9, 0, 0).unwrap(),
            UtcOffset::from_hms(0, 0, 0).unwrap(),
        );
        thread_a_msg1.is_read = true;
        thread_a_msg1.message_id = Some("<thread-a-msg1@company.com>".to_string());
        
        let mut thread_a_msg2 = EmailMessage::new(
            account_id,
            "Re: Team Meeting Tomorrow".to_string(),
            "colleague1@company.com".to_string(),
            vec!["manager@company.com".to_string(), self.accounts.get(&account_id).unwrap().email.clone()],
            "Thanks for the reminder! I'll have my status ready. Quick question - should we bring our laptops for the sprint planning session?\n\nColleague1".to_string(),
            "INBOX".to_string(),
        );
        thread_a_msg2.date = OffsetDateTime::new_in_offset(
            Date::from_calendar_date(2024, Month::March, 10).unwrap(),
            Time::from_hms(9, 15, 0).unwrap(),
            UtcOffset::from_hms(0, 0, 0).unwrap(),
        );
        thread_a_msg2.is_read = true;
        thread_a_msg2.message_id = Some("<thread-a-msg2@company.com>".to_string());
        thread_a_msg2.in_reply_to = Some("<thread-a-msg1@company.com>".to_string());
        thread_a_msg2.references = vec!["<thread-a-msg1@company.com>".to_string()];
        
        let mut thread_a_msg3 = EmailMessage::new(
            account_id,
            "Re: Team Meeting Tomorrow".to_string(),
            self.accounts.get(&account_id).unwrap().email.clone(),
            vec!["manager@company.com".to_string(), "colleague1@company.com".to_string()],
            "Yes, please bring laptops for the sprint planning. We'll be using Jira to create tickets and estimate story points.\n\nDemo".to_string(),
            "Sent".to_string(), // This is a reply from the user, so it goes to Sent
        );
        thread_a_msg3.date = OffsetDateTime::new_in_offset(
            Date::from_calendar_date(2024, Month::March, 10).unwrap(),
            Time::from_hms(9, 30, 0).unwrap(),
            UtcOffset::from_hms(0, 0, 0).unwrap(),
        );
        thread_a_msg3.is_read = true;
        thread_a_msg3.message_id = Some("<thread-a-msg3@example.com>".to_string());
        thread_a_msg3.in_reply_to = Some("<thread-a-msg2@company.com>".to_string());
        thread_a_msg3.references = vec!["<thread-a-msg1@company.com>".to_string(), "<thread-a-msg2@company.com>".to_string()];
        
        let mut thread_a_msg4 = EmailMessage::new(
            account_id,
            "Re: Team Meeting Tomorrow".to_string(),
            "colleague2@company.com".to_string(),
            vec!["manager@company.com".to_string(), self.accounts.get(&account_id).unwrap().email.clone()],
            "Perfect! I'll bring mine. Looking forward to the sprint planning session.\n\nColleague2".to_string(),
            "INBOX".to_string(),
        );
        thread_a_msg4.date = OffsetDateTime::new_in_offset(
            Date::from_calendar_date(2024, Month::March, 10).unwrap(),
            Time::from_hms(9, 45, 0).unwrap(),
            UtcOffset::from_hms(0, 0, 0).unwrap(),
        );
        thread_a_msg4.is_read = true;
        thread_a_msg4.message_id = Some("<thread-a-msg4@company.com>".to_string());
        thread_a_msg4.in_reply_to = Some("<thread-a-msg3@example.com>".to_string());
        thread_a_msg4.references = vec!["<thread-a-msg1@company.com>".to_string(), "<thread-a-msg2@company.com>".to_string(), "<thread-a-msg3@example.com>".to_string()];
        
        // 2. THREAD WITH ONLY ONE MESSAGE (Unread)
        // Thread B: Single message - unread
        let mut thread_b_msg1 = EmailMessage::new(
            account_id,
            "Weekly Tech Newsletter".to_string(),
            "newsletter@tech.com".to_string(),
            vec![self.accounts.get(&account_id).unwrap().email.clone()],
            "This week in tech:\n\n1. New AI developments in machine learning\n2. Latest updates from major tech companies\n3. Open source projects worth checking out\n4. Developer tools and resources\n\nRead the full newsletter on our website.\n\nUnsubscribe | Manage Preferences".to_string(),
            "INBOX".to_string(),
        );
        thread_b_msg1.date = OffsetDateTime::new_in_offset(
            Date::from_calendar_date(2024, Month::March, 12).unwrap(),
            Time::from_hms(9, 0, 0).unwrap(),
            UtcOffset::from_hms(0, 0, 0).unwrap(),
        );
        thread_b_msg1.is_read = false; // Unread
        thread_b_msg1.message_id = Some("<thread-b-msg1@tech.com>".to_string());
        
        // 3. THREAD WITH UNREAD MESSAGES (Mixed read/unread)
        // Thread C: Project Discussion - 3 messages, some unread
        let mut thread_c_msg1 = EmailMessage::new(
            account_id,
            "New Feature Request".to_string(),
            "product@company.com".to_string(),
            vec![self.accounts.get(&account_id).unwrap().email.clone()],
            "Hi Demo,\n\nWe have a new feature request from the client. They want to add dark mode support to the application. Can you provide an estimate for this?\n\nRequirements:\n- Toggle between light and dark themes\n- Persist user preference\n- Support for all existing components\n\nLet me know your thoughts!\n\nProduct Manager".to_string(),
            "INBOX".to_string(),
        );
        thread_c_msg1.date = OffsetDateTime::new_in_offset(
            Date::from_calendar_date(2024, Month::March, 11).unwrap(),
            Time::from_hms(14, 0, 0).unwrap(),
            UtcOffset::from_hms(0, 0, 0).unwrap(),
        );
        thread_c_msg1.is_read = true;
        thread_c_msg1.message_id = Some("<thread-c-msg1@company.com>".to_string());
        
        let mut thread_c_msg2 = EmailMessage::new(
            account_id,
            "Re: New Feature Request".to_string(),
            self.accounts.get(&account_id).unwrap().email.clone(),
            vec!["product@company.com".to_string()],
            "Hi Product Manager,\n\nI can definitely implement dark mode support. Based on the requirements, I estimate this would take about 2-3 days of development time.\n\nI'll need to:\n- Create a theme system\n- Update all CSS variables\n- Add user preference storage\n- Test across all components\n\nShould I start working on this?\n\nDemo".to_string(),
            "INBOX".to_string(),
        );
        thread_c_msg2.date = OffsetDateTime::new_in_offset(
            Date::from_calendar_date(2024, Month::March, 11).unwrap(),
            Time::from_hms(14, 30, 0).unwrap(),
            UtcOffset::from_hms(0, 0, 0).unwrap(),
        );
        thread_c_msg2.is_read = true;
        thread_c_msg2.message_id = Some("<thread-c-msg2@example.com>".to_string());
        thread_c_msg2.in_reply_to = Some("<thread-c-msg1@company.com>".to_string());
        thread_c_msg2.references = vec!["<thread-c-msg1@company.com>".to_string()];
        
        let mut thread_c_msg3 = EmailMessage::new(
            account_id,
            "Re: New Feature Request".to_string(),
            "product@company.com".to_string(),
            vec![self.accounts.get(&account_id).unwrap().email.clone()],
            "Perfect! Yes, please go ahead and start working on it. The client is really excited about this feature.\n\nLet me know if you need any clarification on the requirements.\n\nProduct Manager".to_string(),
            "INBOX".to_string(),
        );
        thread_c_msg3.date = OffsetDateTime::new_in_offset(
            Date::from_calendar_date(2024, Month::March, 11).unwrap(),
            Time::from_hms(15, 0, 0).unwrap(),
            UtcOffset::from_hms(0, 0, 0).unwrap(),
        );
        thread_c_msg3.is_read = false; // Unread
        thread_c_msg3.message_id = Some("<thread-c-msg3@company.com>".to_string());
        thread_c_msg3.in_reply_to = Some("<thread-c-msg2@example.com>".to_string());
        thread_c_msg3.references = vec!["<thread-c-msg1@company.com>".to_string(), "<thread-c-msg2@example.com>".to_string()];
        
        // 4. THREAD WITH ALL MESSAGES READ (2 messages)
        // Thread D: All read conversation
        let mut thread_d_msg1 = EmailMessage::new(
            account_id,
            "Meeting Notes from Yesterday".to_string(),
            "colleague@company.com".to_string(),
            vec![self.accounts.get(&account_id).unwrap().email.clone()],
            "Hi Demo,\n\nHere are the meeting notes from yesterday's client call:\n\n- Client is happy with the current progress\n- They want to add more customization options\n- Next milestone is due in 2 weeks\n- Budget approval for additional features\n\nLet me know if you have any questions!\n\nColleague".to_string(),
            "INBOX".to_string(),
        );
        thread_d_msg1.date = OffsetDateTime::new_in_offset(
            Date::from_calendar_date(2024, Month::March, 9).unwrap(),
            Time::from_hms(16, 0, 0).unwrap(),
            UtcOffset::from_hms(0, 0, 0).unwrap(),
        );
        thread_d_msg1.is_read = true;
        thread_d_msg1.message_id = Some("<thread-d-msg1@company.com>".to_string());
        
        let mut thread_d_msg2 = EmailMessage::new(
            account_id,
            "Re: Meeting Notes from Yesterday".to_string(),
            self.accounts.get(&account_id).unwrap().email.clone(),
            vec!["colleague@company.com".to_string()],
            "Thanks for sharing the notes! The customization options sound interesting. I'll review the requirements and get back to you with any questions.\n\nDemo".to_string(),
            "INBOX".to_string(),
        );
        thread_d_msg2.date = OffsetDateTime::new_in_offset(
            Date::from_calendar_date(2024, Month::March, 9).unwrap(),
            Time::from_hms(16, 15, 0).unwrap(),
            UtcOffset::from_hms(0, 0, 0).unwrap(),
        );
        thread_d_msg2.is_read = true;
        thread_d_msg2.message_id = Some("<thread-d-msg2@example.com>".to_string());
        thread_d_msg2.in_reply_to = Some("<thread-d-msg1@company.com>".to_string());
        thread_d_msg2.references = vec!["<thread-d-msg1@company.com>".to_string()];
        
        // 5. MESSAGE WITH ATTACHMENT (Single message with attachment)
        // Thread E: Document with attachment
        let mut thread_e_msg1 = EmailMessage::new(
            account_id,
            "Contract Review - Please Review Attached Document".to_string(),
            "legal@company.com".to_string(),
            vec![self.accounts.get(&account_id).unwrap().email.clone(), "team@example.com".to_string()],
            "Hi Demo,\n\nPlease review the attached contract document. This is the new service agreement with our client.\n\nKey points to review:\n- Service level agreements\n- Payment terms\n- Intellectual property clauses\n- Termination conditions\n\nPlease provide your feedback by end of week.\n\nLegal Team".to_string(),
            "Drafts".to_string(), // Move this to Drafts to demonstrate different mailbox
        );
        thread_e_msg1.date = OffsetDateTime::new_in_offset(
            Date::from_calendar_date(2024, Month::March, 13).unwrap(),
            Time::from_hms(10, 0, 0).unwrap(),
            UtcOffset::from_hms(0, 0, 0).unwrap(),
        );
        thread_e_msg1.is_read = false; // Unread
        thread_e_msg1.message_id = Some("<thread-e-msg1@company.com>".to_string());
        thread_e_msg1.has_attachments = true;
        thread_e_msg1.cc = vec!["cc@example.com".to_string()];
        
        // Add all messages to backend
        let all_messages = vec![
            thread_a_msg1, thread_a_msg2, thread_a_msg3, thread_a_msg4, // 4-message thread (all read)
            thread_b_msg1, // Single message (unread)
            thread_c_msg1, thread_c_msg2, thread_c_msg3, // 3-message thread (mixed read/unread)
            thread_d_msg1, thread_d_msg2, // 2-message thread (all read)
            thread_e_msg1, // Single message with attachment (unread)
        ];

        if let Some(messages) = self.messages.get_mut(&account_id) {
            messages.extend(all_messages);
        }

        // Update mailbox counts dynamically
        self.update_mailbox_counts_dynamically(account_id);
    }

    /// Mark message as read
    pub fn mark_as_read(&mut self, account_id: Uuid, message_id: Uuid) -> Result<()> {
        if let Some(messages) = self.messages.get_mut(&account_id) {
            if let Some(message) = messages.iter_mut().find(|m| m.id == message_id) {
                message.is_read = true;
            }
        }
        Ok(())
    }

    /// Delete message
    pub fn delete_message(&mut self, account_id: Uuid, message_id: Uuid) -> Result<()> {
        if let Some(messages) = self.messages.get_mut(&account_id) {
            messages.retain(|m| m.id != message_id);
        }
        Ok(())
    }

    /// Add multiple messages to an account
    pub fn add_messages(&mut self, account_id: Uuid, messages: Vec<EmailMessage>) {
        if let Some(account_messages) = self.messages.get_mut(&account_id) {
            account_messages.extend(messages);
        }
    }

    /// Update mailbox counts for an account
    pub fn update_mailbox_counts(&mut self, account_id: Uuid, message_count: u32, unread_count: u32) {
        if let Some(mailboxes) = self.mailboxes.get_mut(&account_id) {
            for mailbox in mailboxes.iter_mut() {
                if mailbox.name == "INBOX" {
                    mailbox.message_count = message_count;
                    mailbox.unread_count = unread_count;
                }
            }
        }
    }

    /// Update mailbox counts dynamically based on actual messages
    pub fn update_mailbox_counts_dynamically(&mut self, account_id: Uuid) {
        // First, get all messages for this account
        let messages = if let Some(account_messages) = self.messages.get(&account_id) {
            account_messages.clone()
        } else {
            return;
        };
        
        // Then update mailbox counts
        if let Some(mailboxes) = self.mailboxes.get_mut(&account_id) {
            for mailbox in mailboxes.iter_mut() {
                // Count messages in this mailbox
                let messages_in_mailbox: Vec<_> = messages.iter()
                    .filter(|msg| msg.mailbox.to_uppercase() == mailbox.name.to_uppercase())
                    .collect();
                mailbox.message_count = messages_in_mailbox.len() as u32;
                
                // Count unread messages in this mailbox
                let unread_count = messages_in_mailbox.iter()
                    .filter(|msg| !msg.is_read)
                    .count() as u32;
                mailbox.unread_count = unread_count;
            }
        }
    }

    /// Move a message from one mailbox to another
    pub fn move_message(&mut self, account_id: Uuid, message_id: Uuid, _from_mailbox: &str, to_mailbox: &str) -> Result<()> {
        if let Some(messages) = self.messages.get_mut(&account_id) {
            if let Some(message) = messages.iter_mut().find(|m| m.id == message_id) {
                message.mailbox = to_mailbox.to_string();
                // Update counts after moving
                self.update_mailbox_counts_dynamically(account_id);
                Ok(())
            } else {
                Err(anyhow::anyhow!("Message not found"))
            }
        } else {
            Err(anyhow::anyhow!("Account not found"))
        }
    }

    /// Get messages for a specific mailbox (real implementation)
    pub fn get_messages_for_mailbox(&self, account_id: Uuid, mailbox_name: &str) -> Vec<&EmailMessage> {
        if let Some(messages) = self.messages.get(&account_id) {
            messages.iter()
                .filter(|msg| msg.mailbox.to_uppercase() == mailbox_name.to_uppercase())
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Mark a message as read
    pub fn mark_message_as_read(&mut self, account_id: Uuid, message_id: Uuid) -> Result<()> {
        if let Some(messages) = self.messages.get_mut(&account_id) {
            if let Some(message) = messages.iter_mut().find(|m| m.id == message_id) {
                message.is_read = true;
                Ok(())
            } else {
                Err(anyhow::anyhow!("Message not found"))
            }
        } else {
            Err(anyhow::anyhow!("Account not found"))
        }
    }
}

impl Default for EmailBackend {
    fn default() -> Self {
        Self::new()
    }
}
