//! Gmail labels management

use crate::error::{AsgardError, AsgardResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Gmail label information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GmailLabel {
    /// Label ID
    pub id: String,
    /// Label name
    pub name: String,
    /// Label type
    pub label_type: GmailLabelType,
    /// Message list visibility
    pub message_list_visibility: GmailLabelVisibility,
    /// Label list visibility
    pub label_list_visibility: GmailLabelVisibility,
    /// Number of messages with this label
    pub messages_total: u32,
    /// Number of unread messages with this label
    pub messages_unread: u32,
    /// Number of threads with this label
    pub threads_total: u32,
    /// Number of unread threads with this label
    pub threads_unread: u32,
}

/// Gmail label types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GmailLabelType {
    /// System label
    System,
    /// User label
    User,
}

/// Gmail label visibility
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GmailLabelVisibility {
    /// Label is visible
    Show,
    /// Label is hidden
    Hide,
}

/// Gmail labels manager
pub struct GmailLabels {
    /// Cached labels
    labels: HashMap<String, GmailLabel>,
}

impl GmailLabels {
    /// Create a new Gmail labels manager
    pub fn new() -> Self {
        Self {
            labels: HashMap::new(),
        }
    }

    /// Add a label
    pub fn add_label(&mut self, label: GmailLabel) {
        self.labels.insert(label.id.clone(), label);
    }

    /// Get a label by ID
    pub fn get_label(&self, label_id: &str) -> Option<&GmailLabel> {
        self.labels.get(label_id)
    }

    /// Get a label by name
    pub fn get_label_by_name(&self, name: &str) -> Option<&GmailLabel> {
        self.labels.values().find(|label| label.name == name)
    }

    /// Get all labels
    pub fn get_all_labels(&self) -> Vec<&GmailLabel> {
        self.labels.values().collect()
    }

    /// Get system labels
    pub fn get_system_labels(&self) -> Vec<&GmailLabel> {
        self.labels.values()
            .filter(|label| label.label_type == GmailLabelType::System)
            .collect()
    }

    /// Get user labels
    pub fn get_user_labels(&self) -> Vec<&GmailLabel> {
        self.labels.values()
            .filter(|label| label.label_type == GmailLabelType::User)
            .collect()
    }

    /// Get visible labels
    pub fn get_visible_labels(&self) -> Vec<&GmailLabel> {
        self.labels.values()
            .filter(|label| label.label_list_visibility == GmailLabelVisibility::Show)
            .collect()
    }

    /// Update label statistics
    pub fn update_label_stats(&mut self, label_id: &str, messages_total: u32, messages_unread: u32, threads_total: u32, threads_unread: u32) -> AsgardResult<()> {
        if let Some(label) = self.labels.get_mut(label_id) {
            label.messages_total = messages_total;
            label.messages_unread = messages_unread;
            label.threads_total = threads_total;
            label.threads_unread = threads_unread;
            Ok(())
        } else {
            Err(AsgardError::not_found(format!("Label not found: {}", label_id)))
        }
    }

    /// Remove a label
    pub fn remove_label(&mut self, label_id: &str) -> Option<GmailLabel> {
        self.labels.remove(label_id)
    }

    /// Clear all labels
    pub fn clear(&mut self) {
        self.labels.clear();
    }

    /// Get label count
    pub fn count(&self) -> usize {
        self.labels.len()
    }

    /// Check if a label exists
    pub fn contains(&self, label_id: &str) -> bool {
        self.labels.contains_key(label_id)
    }

    /// Check if a label exists by name
    pub fn contains_by_name(&self, name: &str) -> bool {
        self.labels.values().any(|label| label.name == name)
    }
}

impl Default for GmailLabels {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gmail_labels_creation() {
        let labels = GmailLabels::new();
        assert_eq!(labels.count(), 0);
    }

    #[test]
    fn test_add_and_get_label() {
        let mut labels = GmailLabels::new();
        
        let label = GmailLabel {
            id: "INBOX".to_string(),
            name: "INBOX".to_string(),
            label_type: GmailLabelType::System,
            message_list_visibility: GmailLabelVisibility::Show,
            label_list_visibility: GmailLabelVisibility::Show,
            messages_total: 10,
            messages_unread: 3,
            threads_total: 5,
            threads_unread: 2,
        };
        
        labels.add_label(label.clone());
        
        assert_eq!(labels.count(), 1);
        assert!(labels.contains("INBOX"));
        assert!(labels.contains_by_name("INBOX"));
        
        let retrieved = labels.get_label("INBOX").unwrap();
        assert_eq!(retrieved.name, "INBOX");
        assert_eq!(retrieved.messages_total, 10);
    }

    #[test]
    fn test_system_and_user_labels() {
        let mut labels = GmailLabels::new();
        
        let system_label = GmailLabel {
            id: "INBOX".to_string(),
            name: "INBOX".to_string(),
            label_type: GmailLabelType::System,
            message_list_visibility: GmailLabelVisibility::Show,
            label_list_visibility: GmailLabelVisibility::Show,
            messages_total: 10,
            messages_unread: 3,
            threads_total: 5,
            threads_unread: 2,
        };
        
        let user_label = GmailLabel {
            id: "custom-label".to_string(),
            name: "Custom Label".to_string(),
            label_type: GmailLabelType::User,
            message_list_visibility: GmailLabelVisibility::Show,
            label_list_visibility: GmailLabelVisibility::Show,
            messages_total: 5,
            messages_unread: 1,
            threads_total: 3,
            threads_unread: 1,
        };
        
        labels.add_label(system_label);
        labels.add_label(user_label);
        
        let system_labels = labels.get_system_labels();
        let user_labels = labels.get_user_labels();
        
        assert_eq!(system_labels.len(), 1);
        assert_eq!(user_labels.len(), 1);
        assert_eq!(system_labels[0].name, "INBOX");
        assert_eq!(user_labels[0].name, "Custom Label");
    }

    #[test]
    fn test_update_label_stats() {
        let mut labels = GmailLabels::new();
        
        let label = GmailLabel {
            id: "INBOX".to_string(),
            name: "INBOX".to_string(),
            label_type: GmailLabelType::System,
            message_list_visibility: GmailLabelVisibility::Show,
            label_list_visibility: GmailLabelVisibility::Show,
            messages_total: 10,
            messages_unread: 3,
            threads_total: 5,
            threads_unread: 2,
        };
        
        labels.add_label(label);
        
        labels.update_label_stats("INBOX", 20, 5, 10, 3).unwrap();
        
        let updated_label = labels.get_label("INBOX").unwrap();
        assert_eq!(updated_label.messages_total, 20);
        assert_eq!(updated_label.messages_unread, 5);
        assert_eq!(updated_label.threads_total, 10);
        assert_eq!(updated_label.threads_unread, 3);
    }

    #[test]
    fn test_remove_label() {
        let mut labels = GmailLabels::new();
        
        let label = GmailLabel {
            id: "INBOX".to_string(),
            name: "INBOX".to_string(),
            label_type: GmailLabelType::System,
            message_list_visibility: GmailLabelVisibility::Show,
            label_list_visibility: GmailLabelVisibility::Show,
            messages_total: 10,
            messages_unread: 3,
            threads_total: 5,
            threads_unread: 2,
        };
        
        labels.add_label(label);
        assert_eq!(labels.count(), 1);
        
        let removed = labels.remove_label("INBOX");
        assert!(removed.is_some());
        assert_eq!(labels.count(), 0);
        assert!(!labels.contains("INBOX"));
    }
}
