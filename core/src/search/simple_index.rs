//! Simple in-memory search index (fallback when tantivy is not available)

use crate::error::{AsgardError, AsgardResult};
use crate::search::{SearchQuery, SearchResult, SearchStats};
use crate::message::Message;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

/// Simple in-memory search index
pub struct SimpleSearchIndex {
    /// Indexed messages
    messages: HashMap<Uuid, IndexedMessage>,
}

/// Indexed message data
#[derive(Debug, Clone)]
struct IndexedMessage {
    /// Message ID
    message_id: Uuid,
    /// Account ID
    account_id: Uuid,
    /// Mailbox ID
    mailbox_id: Uuid,
    /// Subject
    subject: String,
    /// From address
    from_address: String,
    /// To addresses
    to_addresses: String,
    /// Body text
    body_text: String,
    /// Body HTML
    body_html: String,
    /// Date (Unix timestamp)
    date: i64,
    /// Has attachments
    has_attachments: bool,
    /// Is read
    is_read: bool,
    /// Is flagged
    is_flagged: bool,
}

impl SimpleSearchIndex {
    /// Create a new search index
    pub fn new() -> Self {
        Self {
            messages: HashMap::new(),
        }
    }

    /// Add a message to the index
    pub fn add_message(&mut self, message: &Message) -> AsgardResult<()> {
        let message_id = message.id;
        let account_id = message.account_id;
        let mailbox_id = message.mailbox_id;
        
        let subject = message.headers.subject.clone();
        let from_address = message.headers.from.first()
            .map(|addr| addr.email.clone())
            .unwrap_or_default();
        let to_addresses = message.headers.to.iter()
            .map(|addr| addr.email.as_str())
            .collect::<Vec<_>>()
            .join(" ");
        
        let body_text = message.text_content()
            .and_then(|content| String::from_utf8(content.to_vec()).ok())
            .unwrap_or_default();
        
        let body_html = message.html_content()
            .and_then(|content| String::from_utf8(content.to_vec()).ok())
            .unwrap_or_default();
        
        let date = message.headers.date
            .map(|dt| dt.unix_timestamp())
            .unwrap_or(0);
        
        let has_attachments = !message.attachments.is_empty();
        let is_read = message.is_read();
        let is_flagged = message.is_flagged();
        
        let indexed_message = IndexedMessage {
            message_id,
            account_id,
            mailbox_id,
            subject,
            from_address,
            to_addresses,
            body_text,
            body_html,
            date,
            has_attachments,
            is_read,
            is_flagged,
        };
        
        self.messages.insert(message_id, indexed_message);
        Ok(())
    }

    /// Update a message in the index
    pub fn update_message(&mut self, message: &Message) -> AsgardResult<()> {
        self.add_message(message)
    }

    /// Remove a message from the index
    pub fn remove_message(&mut self, message_id: Uuid) -> AsgardResult<()> {
        self.messages.remove(&message_id);
        Ok(())
    }

    /// Search the index
    pub fn search(&self, query: &SearchQuery) -> AsgardResult<Vec<SearchResult>> {
        let mut results = Vec::new();
        
        for (message_id, indexed_message) in &self.messages {
            // Apply filters
            if let Some(account_id) = query.account_id {
                if indexed_message.account_id != account_id {
                    continue;
                }
            }
            
            if let Some(mailbox_id) = query.mailbox_id {
                if indexed_message.mailbox_id != mailbox_id {
                    continue;
                }
            }
            
            if let Some(has_attachments) = query.has_attachments {
                if indexed_message.has_attachments != has_attachments {
                    continue;
                }
            }
            
            if let Some(is_read) = query.is_read {
                if indexed_message.is_read != is_read {
                    continue;
                }
            }
            
            if let Some(is_flagged) = query.is_flagged {
                if indexed_message.is_flagged != is_flagged {
                    continue;
                }
            }
            
            // Text search
            if !query.query.is_empty() {
                let mut matches = false;
                let query_lower = query.query.to_lowercase();
                
                if query.search_subject && indexed_message.subject.to_lowercase().contains(&query_lower) {
                    matches = true;
                }
                
                if query.search_from && indexed_message.from_address.to_lowercase().contains(&query_lower) {
                    matches = true;
                }
                
                if query.search_to && indexed_message.to_addresses.to_lowercase().contains(&query_lower) {
                    matches = true;
                }
                
                if query.search_body && indexed_message.body_text.to_lowercase().contains(&query_lower) {
                    matches = true;
                }
                
                if !matches {
                    continue;
                }
            }
            
            // Create snippets
            let mut snippets = Vec::new();
            if query.search_subject && indexed_message.subject.contains(&query.query) {
                snippets.push(format!("Subject: {}", indexed_message.subject));
            }
            if query.search_from && indexed_message.from_address.contains(&query.query) {
                snippets.push(format!("From: {}", indexed_message.from_address));
            }
            if query.search_body && indexed_message.body_text.contains(&query.query) {
                // Simple snippet extraction
                let body_lower = indexed_message.body_text.to_lowercase();
                let query_lower = query.query.to_lowercase();
                if let Some(pos) = body_lower.find(&query_lower) {
                    let start = pos.saturating_sub(50);
                    let end = (pos + query.query.len() + 50).min(indexed_message.body_text.len());
                    let snippet = &indexed_message.body_text[start..end];
                    snippets.push(format!("Body: ...{}...", snippet));
                }
            }
            
            results.push(SearchResult {
                message_id: *message_id,
                account_id: indexed_message.account_id,
                mailbox_id: indexed_message.mailbox_id,
                score: 1.0, // Simple scoring
                snippets,
            });
        }
        
        // Sort by date (newest first)
        results.sort_by(|a, b| {
            let a_date = self.messages.get(&a.message_id).map(|m| m.date).unwrap_or(0);
            let b_date = self.messages.get(&b.message_id).map(|m| m.date).unwrap_or(0);
            b_date.cmp(&a_date)
        });
        
        // Apply limit
        if let Some(limit) = query.limit {
            results.truncate(limit);
        }
        
        Ok(results)
    }

    /// Get search statistics
    pub fn get_stats(&self) -> AsgardResult<SearchStats> {
        Ok(SearchStats {
            total_results: 0,
            search_time_ms: 0,
            index_size_bytes: (self.messages.len() * 1000) as u64, // Rough estimate
            indexed_documents: self.messages.len(),
        })
    }

    /// Clear the entire index
    pub fn clear(&mut self) -> AsgardResult<()> {
        self.messages.clear();
        Ok(())
    }

    /// Optimize the index (no-op for simple index)
    pub fn optimize(&mut self) -> AsgardResult<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::message::{Message, MessageHeaders, EmailAddress, MessageImportance};
    use std::collections::HashMap;

    #[test]
    fn test_search_index_creation() {
        let _index = SimpleSearchIndex::new();
    }

    #[test]
    fn test_add_and_search_message() {
        let mut index = SimpleSearchIndex::new();

        let account_id = Uuid::new_v4();
        let mailbox_id = Uuid::new_v4();
        
        let headers = MessageHeaders {
            message_id: None,
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
            date: None,
            received_date: None,
            importance: MessageImportance::Normal,
            custom: HashMap::new(),
        };
        
        let message = Message::new(account_id, mailbox_id, headers);
        
        // Add message to index
        index.add_message(&message).unwrap();
        
        // Search for the message
        let search_query = SearchQuery {
            query: "Test".to_string(),
            ..Default::default()
        };
        
        let results = index.search(&search_query).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].message_id, message.id);
    }

    #[test]
    fn test_remove_message() {
        let mut index = SimpleSearchIndex::new();

        let account_id = Uuid::new_v4();
        let mailbox_id = Uuid::new_v4();
        
        let headers = MessageHeaders {
            message_id: None,
            in_reply_to: None,
            references: None,
            subject: "Test Subject".to_string(),
            from: vec![EmailAddress {
                name: None,
                email: "sender@example.com".to_string(),
            }],
            to: vec![EmailAddress {
                name: None,
                email: "recipient@example.com".to_string(),
            }],
            cc: vec![],
            bcc: vec![],
            reply_to: vec![],
            date: None,
            received_date: None,
            importance: MessageImportance::Normal,
            custom: HashMap::new(),
        };
        
        let message = Message::new(account_id, mailbox_id, headers);
        
        // Add message to index
        index.add_message(&message).unwrap();
        
        // Search for the message
        let search_query = SearchQuery {
            query: "Test".to_string(),
            ..Default::default()
        };
        
        let results = index.search(&search_query).unwrap();
        assert_eq!(results.len(), 1);
        
        // Remove message from index
        index.remove_message(message.id).unwrap();
        
        // Search again - should find nothing
        let results = index.search(&search_query).unwrap();
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_search_stats() {
        let index = SimpleSearchIndex::new();
        
        let stats = index.get_stats().unwrap();
        assert_eq!(stats.indexed_documents, 0);
    }
}
