//! Mailbox management for Asgard Mail

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::error::{AsgardError, AsgardResult};

/// Mailbox types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MailboxType {
    /// Inbox folder
    Inbox,
    /// Sent folder
    Sent,
    /// Drafts folder
    Drafts,
    /// Trash folder
    Trash,
    /// Spam folder
    Spam,
    /// Archive folder
    Archive,
    /// Custom folder
    Custom,
    /// Gmail label
    Label,
}

impl std::fmt::Display for MailboxType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MailboxType::Inbox => write!(f, "Inbox"),
            MailboxType::Sent => write!(f, "Sent"),
            MailboxType::Drafts => write!(f, "Drafts"),
            MailboxType::Trash => write!(f, "Trash"),
            MailboxType::Spam => write!(f, "Spam"),
            MailboxType::Archive => write!(f, "Archive"),
            MailboxType::Custom => write!(f, "Custom"),
            MailboxType::Label => write!(f, "Label"),
        }
    }
}

/// Mailbox flags
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MailboxFlags {
    /// No flags
    None,
    /// No select flag (cannot be selected)
    NoSelect,
    /// No inferiors flag (no child mailboxes)
    NoInferiors,
    /// Marked flag (marked for deletion)
    Marked,
    /// Unmarked flag (not marked for deletion)
    Unmarked,
    /// Has children flag (has child mailboxes)
    HasChildren,
    /// Has no children flag (no child mailboxes)
    HasNoChildren,
    /// All flag (all messages)
    All,
    /// Archive flag (archive mailbox)
    Archive,
    /// Drafts flag (drafts mailbox)
    Drafts,
    /// Flagged flag (flagged messages)
    Flagged,
    /// Junk flag (junk/spam mailbox)
    Junk,
    /// Sent flag (sent messages)
    Sent,
    /// Trash flag (trash mailbox)
    Trash,
}

impl std::fmt::Display for MailboxFlags {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MailboxFlags::None => write!(f, "None"),
            MailboxFlags::NoSelect => write!(f, "NoSelect"),
            MailboxFlags::NoInferiors => write!(f, "NoInferiors"),
            MailboxFlags::Marked => write!(f, "Marked"),
            MailboxFlags::Unmarked => write!(f, "Unmarked"),
            MailboxFlags::HasChildren => write!(f, "HasChildren"),
            MailboxFlags::HasNoChildren => write!(f, "HasNoChildren"),
            MailboxFlags::All => write!(f, "All"),
            MailboxFlags::Archive => write!(f, "Archive"),
            MailboxFlags::Drafts => write!(f, "Drafts"),
            MailboxFlags::Flagged => write!(f, "Flagged"),
            MailboxFlags::Junk => write!(f, "Junk"),
            MailboxFlags::Sent => write!(f, "Sent"),
            MailboxFlags::Trash => write!(f, "Trash"),
        }
    }
}

/// Mailbox attributes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MailboxAttributes {
    /// Mailbox flags
    pub flags: Vec<MailboxFlags>,
    /// Mailbox permissions
    pub permissions: Vec<String>,
    /// Mailbox capabilities
    pub capabilities: Vec<String>,
}

/// Mailbox statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MailboxStats {
    /// Total number of messages
    pub total_messages: u32,
    /// Number of unread messages
    pub unread_messages: u32,
    /// Number of recent messages
    pub recent_messages: u32,
    /// Next UID validity
    pub uid_validity: Option<u32>,
    /// Next UID
    pub uid_next: Option<u32>,
    /// Highest modification sequence
    pub highest_modseq: Option<u64>,
    /// Mailbox size in bytes
    pub size: u64,
    /// Last message received time
    pub last_message_received: Option<OffsetDateTime>,
}

/// Mailbox represents a folder or label in an email account
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mailbox {
    /// Unique mailbox ID
    pub id: Uuid,
    /// Account ID this mailbox belongs to
    pub account_id: Uuid,
    /// Mailbox name
    pub name: String,
    /// Display name
    pub display_name: String,
    /// Mailbox type
    pub mailbox_type: MailboxType,
    /// Parent mailbox ID (for hierarchy)
    pub parent_id: Option<Uuid>,
    /// Mailbox attributes
    pub attributes: MailboxAttributes,
    /// Mailbox statistics
    pub stats: MailboxStats,
    /// Mailbox-specific settings
    pub settings: HashMap<String, serde_json::Value>,
    /// Creation time
    pub created_at: OffsetDateTime,
    /// Last modification time
    pub updated_at: OffsetDateTime,
    /// Last sync time
    pub last_sync: Option<OffsetDateTime>,
}

impl Mailbox {
    /// Create a new mailbox
    pub fn new(
        account_id: Uuid,
        name: String,
        display_name: Option<String>,
        mailbox_type: MailboxType,
        parent_id: Option<Uuid>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            account_id,
            name: name.clone(),
            display_name: display_name.unwrap_or_else(|| name),
            mailbox_type,
            parent_id,
            attributes: MailboxAttributes {
                flags: vec![],
                permissions: vec![],
                capabilities: vec![],
            },
            stats: MailboxStats::default(),
            settings: HashMap::new(),
            created_at: OffsetDateTime::now_utc(),
            updated_at: OffsetDateTime::now_utc(),
            last_sync: None,
        }
    }

    /// Create a new inbox mailbox
    pub fn new_inbox(account_id: Uuid) -> Self {
        Self::new(
            account_id,
            "INBOX".to_string(),
            Some("Inbox".to_string()),
            MailboxType::Inbox,
            None,
        )
    }

    /// Create a new sent mailbox
    pub fn new_sent(account_id: Uuid, name: String) -> Self {
        Self::new(
            account_id,
            name,
            Some("Sent".to_string()),
            MailboxType::Sent,
            None,
        )
    }

    /// Create a new drafts mailbox
    pub fn new_drafts(account_id: Uuid, name: String) -> Self {
        Self::new(
            account_id,
            name,
            Some("Drafts".to_string()),
            MailboxType::Drafts,
            None,
        )
    }

    /// Create a new trash mailbox
    pub fn new_trash(account_id: Uuid, name: String) -> Self {
        Self::new(
            account_id,
            name,
            Some("Trash".to_string()),
            MailboxType::Trash,
            None,
        )
    }

    /// Create a new spam mailbox
    pub fn new_spam(account_id: Uuid, name: String) -> Self {
        Self::new(
            account_id,
            name,
            Some("Spam".to_string()),
            MailboxType::Spam,
            None,
        )
    }

    /// Create a new archive mailbox
    pub fn new_archive(account_id: Uuid, name: String) -> Self {
        Self::new(
            account_id,
            name,
            Some("Archive".to_string()),
            MailboxType::Archive,
            None,
        )
    }

    /// Create a new Gmail label
    pub fn new_gmail_label(account_id: Uuid, name: String, display_name: Option<String>) -> Self {
        Self::new(
            account_id,
            name,
            display_name,
            MailboxType::Label,
            None,
        )
    }

    /// Get the mailbox name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the display name
    pub fn display_name(&self) -> &str {
        &self.display_name
    }

    /// Get the mailbox type
    pub fn mailbox_type(&self) -> MailboxType {
        self.mailbox_type
    }

    /// Get the account ID
    pub fn account_id(&self) -> Uuid {
        self.account_id
    }

    /// Get the parent ID
    pub fn parent_id(&self) -> Option<Uuid> {
        self.parent_id
    }

    /// Check if this is an inbox
    pub fn is_inbox(&self) -> bool {
        self.mailbox_type == MailboxType::Inbox
    }

    /// Check if this is a sent mailbox
    pub fn is_sent(&self) -> bool {
        self.mailbox_type == MailboxType::Sent
    }

    /// Check if this is a drafts mailbox
    pub fn is_drafts(&self) -> bool {
        self.mailbox_type == MailboxType::Drafts
    }

    /// Check if this is a trash mailbox
    pub fn is_trash(&self) -> bool {
        self.mailbox_type == MailboxType::Trash
    }

    /// Check if this is a spam mailbox
    pub fn is_spam(&self) -> bool {
        self.mailbox_type == MailboxType::Spam
    }

    /// Check if this is an archive mailbox
    pub fn is_archive(&self) -> bool {
        self.mailbox_type == MailboxType::Archive
    }

    /// Check if this is a Gmail label
    pub fn is_gmail_label(&self) -> bool {
        self.mailbox_type == MailboxType::Label
    }

    /// Check if this is a system mailbox
    pub fn is_system(&self) -> bool {
        matches!(
            self.mailbox_type,
            MailboxType::Inbox
                | MailboxType::Sent
                | MailboxType::Drafts
                | MailboxType::Trash
                | MailboxType::Spam
                | MailboxType::Archive
        )
    }

    /// Check if this mailbox can be selected
    pub fn can_select(&self) -> bool {
        !self.attributes.flags.contains(&MailboxFlags::NoSelect)
    }

    /// Check if this mailbox has children
    pub fn has_children(&self) -> bool {
        self.attributes.flags.contains(&MailboxFlags::HasChildren)
    }

    /// Check if this mailbox has no children
    pub fn has_no_children(&self) -> bool {
        self.attributes.flags.contains(&MailboxFlags::HasNoChildren)
    }

    /// Update mailbox statistics
    pub fn update_stats(&mut self, stats: MailboxStats) {
        self.stats = stats;
        self.updated_at = OffsetDateTime::now_utc();
    }

    /// Update the last sync time
    pub fn update_last_sync(&mut self) {
        self.last_sync = Some(OffsetDateTime::now_utc());
        self.updated_at = OffsetDateTime::now_utc();
    }

    /// Set mailbox attributes
    pub fn set_attributes(&mut self, attributes: MailboxAttributes) {
        self.attributes = attributes;
        self.updated_at = OffsetDateTime::now_utc();
    }

    /// Add a flag to the mailbox
    pub fn add_flag(&mut self, flag: MailboxFlags) {
        if !self.attributes.flags.contains(&flag) {
            self.attributes.flags.push(flag);
            self.updated_at = OffsetDateTime::now_utc();
        }
    }

    /// Remove a flag from the mailbox
    pub fn remove_flag(&mut self, flag: MailboxFlags) {
        self.attributes.flags.retain(|&f| f != flag);
        self.updated_at = OffsetDateTime::now_utc();
    }

    /// Check if the mailbox has a specific flag
    pub fn has_flag(&self, flag: MailboxFlags) -> bool {
        self.attributes.flags.contains(&flag)
    }

    /// Get the full path of the mailbox (for hierarchy display)
    pub fn full_path(&self, mailboxes: &[Mailbox]) -> String {
        if let Some(parent_id) = self.parent_id {
            if let Some(parent) = mailboxes.iter().find(|m| m.id == parent_id) {
                format!("{}/{}", parent.full_path(mailboxes), self.display_name)
            } else {
                self.display_name.clone()
            }
        } else {
            self.display_name.clone()
        }
    }

    /// Get the hierarchy level (0 for root mailboxes)
    pub fn hierarchy_level(&self, mailboxes: &[Mailbox]) -> usize {
        if let Some(parent_id) = self.parent_id {
            if let Some(parent) = mailboxes.iter().find(|m| m.id == parent_id) {
                1 + parent.hierarchy_level(mailboxes)
            } else {
                0
            }
        } else {
            0
        }
    }

    /// Get child mailboxes
    pub fn children<'a>(&self, mailboxes: &'a [Mailbox]) -> Vec<&'a Mailbox> {
        mailboxes
            .iter()
            .filter(|m| m.parent_id == Some(self.id))
            .collect()
    }

    /// Get all descendant mailboxes
    pub fn descendants<'a>(&self, mailboxes: &'a [Mailbox]) -> Vec<&'a Mailbox> {
        let mut descendants = Vec::new();
        let children = self.children(mailboxes);
        
        for child in children {
            descendants.push(child);
            descendants.extend(child.descendants(mailboxes));
        }
        
        descendants
    }

    /// Validate the mailbox
    pub fn validate(&self) -> AsgardResult<()> {
        if self.name.is_empty() {
            return Err(AsgardError::validation("Mailbox name cannot be empty"));
        }
        
        if self.display_name.is_empty() {
            return Err(AsgardError::validation("Mailbox display name cannot be empty"));
        }
        
        Ok(())
    }
}

/// Mailbox hierarchy builder
pub struct MailboxHierarchy {
    mailboxes: Vec<Mailbox>,
}

impl MailboxHierarchy {
    /// Create a new mailbox hierarchy
    pub fn new() -> Self {
        Self {
            mailboxes: Vec::new(),
        }
    }

    /// Add a mailbox to the hierarchy
    pub fn add_mailbox(&mut self, mailbox: Mailbox) {
        self.mailboxes.push(mailbox);
    }

    /// Get all mailboxes
    pub fn mailboxes(&self) -> &[Mailbox] {
        &self.mailboxes
    }

    /// Get root mailboxes (no parent)
    pub fn root_mailboxes(&self) -> Vec<&Mailbox> {
        self.mailboxes
            .iter()
            .filter(|m| m.parent_id.is_none())
            .collect()
    }

    /// Get mailboxes for a specific account
    pub fn account_mailboxes(&self, account_id: Uuid) -> Vec<&Mailbox> {
        self.mailboxes
            .iter()
            .filter(|m| m.account_id == account_id)
            .collect()
    }

    /// Get mailboxes by type
    pub fn mailboxes_by_type(&self, mailbox_type: MailboxType) -> Vec<&Mailbox> {
        self.mailboxes
            .iter()
            .filter(|m| m.mailbox_type == mailbox_type)
            .collect()
    }

    /// Build a tree structure for display
    pub fn build_tree(&self, account_id: Uuid) -> Vec<MailboxTreeNode> {
        let account_mailboxes = self.account_mailboxes(account_id);
        let mut tree = Vec::new();
        
        // Add root mailboxes
        for mailbox in account_mailboxes.iter().filter(|m| m.parent_id.is_none()) {
            tree.push(self.build_tree_node(mailbox, &account_mailboxes));
        }
        
        tree
    }

    fn build_tree_node(&self, mailbox: &Mailbox, all_mailboxes: &[&Mailbox]) -> MailboxTreeNode {
        // Convert &[&Mailbox] to Vec<Mailbox> for the children method
        let mailboxes_owned: Vec<Mailbox> = all_mailboxes.iter().map(|m| (*m).clone()).collect();
        let children = mailbox
            .children(&mailboxes_owned)
            .into_iter()
            .map(|child| self.build_tree_node(child, all_mailboxes))
            .collect();
        
        MailboxTreeNode {
            mailbox: mailbox.clone(),
            children,
        }
    }
}

/// Tree node for mailbox hierarchy display
#[derive(Debug, Clone)]
pub struct MailboxTreeNode {
    pub mailbox: Mailbox,
    pub children: Vec<MailboxTreeNode>,
}

impl MailboxTreeNode {
    /// Get the mailbox
    pub fn mailbox(&self) -> &Mailbox {
        &self.mailbox
    }

    /// Get the children
    pub fn children(&self) -> &[MailboxTreeNode] {
        &self.children
    }

    /// Check if this node has children
    pub fn has_children(&self) -> bool {
        !self.children.is_empty()
    }

    /// Get the total number of descendants
    pub fn descendant_count(&self) -> usize {
        let mut count = self.children.len();
        for child in &self.children {
            count += child.descendant_count();
        }
        count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mailbox_creation() {
        let account_id = Uuid::new_v4();
        let mailbox = Mailbox::new(
            account_id,
            "INBOX".to_string(),
            Some("Inbox".to_string()),
            MailboxType::Inbox,
            None,
        );

        assert_eq!(mailbox.name(), "INBOX");
        assert_eq!(mailbox.display_name(), "Inbox");
        assert_eq!(mailbox.mailbox_type(), MailboxType::Inbox);
        assert_eq!(mailbox.account_id(), account_id);
        assert!(mailbox.is_inbox());
        assert!(mailbox.is_system());
    }

    #[test]
    fn test_gmail_label_creation() {
        let account_id = Uuid::new_v4();
        let label = Mailbox::new_gmail_label(
            account_id,
            "Important".to_string(),
            Some("Important".to_string()),
        );

        assert_eq!(label.name(), "Important");
        assert_eq!(label.display_name(), "Important");
        assert_eq!(label.mailbox_type(), MailboxType::Label);
        assert!(label.is_gmail_label());
        assert!(!label.is_system());
    }

    #[test]
    fn test_mailbox_hierarchy() {
        let account_id = Uuid::new_v4();
        let mut hierarchy = MailboxHierarchy::new();
        
        let inbox = Mailbox::new_inbox(account_id);
        let sent = Mailbox::new_sent(account_id, "Sent".to_string());
        let drafts = Mailbox::new_drafts(account_id, "Drafts".to_string());
        
        hierarchy.add_mailbox(inbox);
        hierarchy.add_mailbox(sent);
        hierarchy.add_mailbox(drafts);
        
        let root_mailboxes = hierarchy.root_mailboxes();
        assert_eq!(root_mailboxes.len(), 3);
        
        let account_mailboxes = hierarchy.account_mailboxes(account_id);
        assert_eq!(account_mailboxes.len(), 3);
    }

    #[test]
    fn test_mailbox_validation() {
        let account_id = Uuid::new_v4();
        let mailbox = Mailbox::new(
            account_id,
            "INBOX".to_string(),
            Some("Inbox".to_string()),
            MailboxType::Inbox,
            None,
        );

        assert!(mailbox.validate().is_ok());
    }

    #[test]
    fn test_mailbox_flags() {
        let account_id = Uuid::new_v4();
        let mut mailbox = Mailbox::new(
            account_id,
            "INBOX".to_string(),
            Some("Inbox".to_string()),
            MailboxType::Inbox,
            None,
        );

        mailbox.add_flag(MailboxFlags::HasChildren);
        assert!(mailbox.has_flag(MailboxFlags::HasChildren));
        
        mailbox.remove_flag(MailboxFlags::HasChildren);
        assert!(!mailbox.has_flag(MailboxFlags::HasChildren));
    }
}
