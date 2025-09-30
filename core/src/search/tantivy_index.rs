//! Tantivy-based full-text search index

use crate::error::{AsgardError, AsgardResult};
use crate::search::{SearchQuery, SearchResult, SearchStats};
use crate::message::Message;
use std::path::PathBuf;
use std::sync::Arc;
use tantivy::{
    collector::TopDocs,
    directory::MmapDirectory,
    doc,
    query::{Query, TermQuery},
    schema::{Field, Schema, TextFieldIndexing, TextOptions, STORED, STRING, TEXT},
    Index, IndexReader, IndexWriter, ReloadPolicy, Term,
};
use time::OffsetDateTime;
use uuid::Uuid;

/// Tantivy-based search index
pub struct TantivySearchIndex {
    /// Tantivy index
    index: Index,
    /// Index reader
    reader: IndexReader,
    /// Schema fields
    fields: SearchFields,
}

/// Search index fields
#[derive(Debug, Clone)]
struct SearchFields {
    /// Message ID field
    message_id: Field,
    /// Account ID field
    account_id: Field,
    /// Mailbox ID field
    mailbox_id: Field,
    /// Subject field
    subject: Field,
    /// From address field
    from_address: Field,
    /// To addresses field
    to_addresses: Field,
    /// Body text field
    body_text: Field,
    /// Body HTML field
    body_html: Field,
    /// Date field
    date: Field,
    /// Has attachments field
    has_attachments: Field,
    /// Is read field
    is_read: Field,
    /// Is flagged field
    is_flagged: Field,
}

impl TantivySearchIndex {
    /// Create a new search index
    pub fn new(index_dir: PathBuf) -> AsgardResult<Self> {
        // Ensure index directory exists
        std::fs::create_dir_all(&index_dir)?;

        // Create schema
        let mut schema_builder = Schema::builder();
        
        let message_id = schema_builder.add_text_field("message_id", STRING | STORED);
        let account_id = schema_builder.add_text_field("account_id", STRING | STORED);
        let mailbox_id = schema_builder.add_text_field("mailbox_id", STRING | STORED);
        
        let text_options = TextOptions::default()
            .set_indexing_options(TextFieldIndexing::default().set_tokenizer("default"));
        
        let subject = schema_builder.add_text_field("subject", text_options.clone() | STORED);
        let from_address = schema_builder.add_text_field("from_address", text_options.clone() | STORED);
        let to_addresses = schema_builder.add_text_field("to_addresses", text_options.clone() | STORED);
        let body_text = schema_builder.add_text_field("body_text", text_options.clone());
        let body_html = schema_builder.add_text_field("body_html", text_options.clone());
        
        let date = schema_builder.add_i64_field("date", STORED);
        let has_attachments = schema_builder.add_bool_field("has_attachments", STORED);
        let is_read = schema_builder.add_bool_field("is_read", STORED);
        let is_flagged = schema_builder.add_bool_field("is_flagged", STORED);
        
        let schema = schema_builder.build();
        
        let fields = SearchFields {
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

        // Create or open index
        let index = Index::open_or_create(MmapDirectory::open(&index_dir)?, schema)?;
        
        // Create reader
        let reader = index
            .reader_builder()
            .reload_policy(ReloadPolicy::OnCommit)
            .try_into()?;

        Ok(Self {
            index,
            reader,
            fields,
        })
    }

    /// Add a message to the index
    pub fn add_message(&mut self, message: &Message) -> AsgardResult<()> {
        let mut index_writer = self.index.writer(50_000_000)?; // 50MB buffer
        
        let message_id_str = message.id.to_string();
        let account_id_str = message.account_id.to_string();
        let mailbox_id_str = message.mailbox_id.to_string();
        
        let subject = &message.headers.subject;
        let from_address = message.headers.from.first()
            .map(|addr| addr.email.as_str())
            .unwrap_or("");
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
        
        index_writer.add_document(doc!(
            self.fields.message_id => message_id_str,
            self.fields.account_id => account_id_str,
            self.fields.mailbox_id => mailbox_id_str,
            self.fields.subject => subject,
            self.fields.from_address => from_address,
            self.fields.to_addresses => to_addresses,
            self.fields.body_text => body_text,
            self.fields.body_html => body_html,
            self.fields.date => date,
            self.fields.has_attachments => has_attachments,
            self.fields.is_read => is_read,
            self.fields.is_flagged => is_flagged,
        ))?;
        
        index_writer.commit()?;
        Ok(())
    }

    /// Update a message in the index
    pub fn update_message(&mut self, message: &Message) -> AsgardResult<()> {
        // Remove old document
        self.remove_message(message.id)?;
        
        // Add updated document
        self.add_message(message)?;
        
        Ok(())
    }

    /// Remove a message from the index
    pub fn remove_message(&mut self, message_id: Uuid) -> AsgardResult<()> {
        let mut index_writer = self.index.writer(50_000_000)?;
        
        let term = Term::from_field_text(self.fields.message_id, &message_id.to_string());
        index_writer.delete_term(term);
        
        index_writer.commit()?;
        Ok(())
    }

    /// Search the index
    pub fn search(&self, query: &SearchQuery) -> AsgardResult<Vec<SearchResult>> {
        let searcher = self.reader.searcher();
        
        // Build search query
        let tantivy_query = self.build_query(query)?;
        
        // Execute search
        let limit = query.limit.unwrap_or(100);
        let top_docs = searcher.search(&tantivy_query, &TopDocs::with_limit(limit))?;
        
        let mut results = Vec::new();
        for (score, doc_address) in top_docs {
            let retrieved_doc = searcher.doc(doc_address)?;
            
            let message_id = retrieved_doc
                .get_first(self.fields.message_id)
                .and_then(|v| v.as_text())
                .and_then(|s| Uuid::parse_str(s).ok())
                .ok_or_else(|| AsgardError::search_index("Invalid message ID"))?;
            
            let account_id = retrieved_doc
                .get_first(self.fields.account_id)
                .and_then(|v| v.as_text())
                .and_then(|s| Uuid::parse_str(s).ok())
                .ok_or_else(|| AsgardError::search_index("Invalid account ID"))?;
            
            let mailbox_id = retrieved_doc
                .get_first(self.fields.mailbox_id)
                .and_then(|v| v.as_text())
                .and_then(|s| Uuid::parse_str(s).ok())
                .ok_or_else(|| AsgardError::search_index("Invalid mailbox ID"))?;
            
            let subject = retrieved_doc
                .get_first(self.fields.subject)
                .and_then(|v| v.as_text())
                .unwrap_or("")
                .to_string();
            
            let from_address = retrieved_doc
                .get_first(self.fields.from_address)
                .and_then(|v| v.as_text())
                .unwrap_or("")
                .to_string();
            
            // Create snippets (simplified)
            let snippets = vec![
                format!("Subject: {}", subject),
                format!("From: {}", from_address),
            ];
            
            results.push(SearchResult {
                message_id,
                account_id,
                mailbox_id,
                score,
                snippets,
            });
        }
        
        Ok(results)
    }

    /// Get search statistics
    pub fn get_stats(&self) -> AsgardResult<SearchStats> {
        let searcher = self.reader.searcher();
        let num_docs = searcher.num_docs();
        
        // Get index size (simplified)
        let index_size_bytes = 0; // Would need to calculate actual size
        
        Ok(SearchStats {
            total_results: 0, // Would be set during search
            search_time_ms: 0, // Would be measured during search
            index_size_bytes,
            indexed_documents: num_docs as usize,
        })
    }

    /// Clear the entire index
    pub fn clear(&mut self) -> AsgardResult<()> {
        let mut index_writer = self.index.writer(50_000_000)?;
        index_writer.delete_all_documents()?;
        index_writer.commit()?;
        Ok(())
    }

    /// Optimize the index
    pub fn optimize(&mut self) -> AsgardResult<()> {
        let mut index_writer = self.index.writer(50_000_000)?;
        index_writer.merge_segments()?;
        index_writer.commit()?;
        Ok(())
    }

    // Helper methods

    fn build_query(&self, query: &SearchQuery) -> AsgardResult<Box<dyn Query>> {
        use tantivy::query::{BooleanQuery, Occur, TermQuery as TQ};
        
        let mut subqueries = Vec::new();
        
        // Text search query
        if !query.query.is_empty() {
            let text_query = self.build_text_query(&query.query, query)?;
            subqueries.push((Occur::Must, text_query));
        }
        
        // Account filter
        if let Some(account_id) = query.account_id {
            let term = Term::from_field_text(self.fields.account_id, &account_id.to_string());
            subqueries.push((Occur::Must, Box::new(TQ::new(term, tantivy::schema::IndexRecordOption::Basic))));
        }
        
        // Mailbox filter
        if let Some(mailbox_id) = query.mailbox_id {
            let term = Term::from_field_text(self.fields.mailbox_id, &mailbox_id.to_string());
            subqueries.push((Occur::Must, Box::new(TQ::new(term, tantivy::schema::IndexRecordOption::Basic))));
        }
        
        // Has attachments filter
        if let Some(has_attachments) = query.has_attachments {
            let term = Term::from_field_bool(self.fields.has_attachments, has_attachments);
            subqueries.push((Occur::Must, Box::new(TQ::new(term, tantivy::schema::IndexRecordOption::Basic))));
        }
        
        // Is read filter
        if let Some(is_read) = query.is_read {
            let term = Term::from_field_bool(self.fields.is_read, is_read);
            subqueries.push((Occur::Must, Box::new(TQ::new(term, tantivy::schema::IndexRecordOption::Basic))));
        }
        
        // Is flagged filter
        if let Some(is_flagged) = query.is_flagged {
            let term = Term::from_field_bool(self.fields.is_flagged, is_flagged);
            subqueries.push((Occur::Must, Box::new(TQ::new(term, tantivy::schema::IndexRecordOption::Basic))));
        }
        
        if subqueries.is_empty() {
            // Return a match-all query
            use tantivy::query::AllQuery;
            Ok(Box::new(AllQuery))
        } else {
            Ok(Box::new(BooleanQuery::new(subqueries)))
        }
    }

    fn build_text_query(&self, query_text: &str, search_query: &SearchQuery) -> AsgardResult<Box<dyn Query>> {
        use tantivy::query::{BooleanQuery, Occur, TermQuery as TQ};
        
        let mut subqueries = Vec::new();
        
        // Simple term-based search (in a real implementation, you'd use a proper query parser)
        let terms: Vec<&str> = query_text.split_whitespace().collect();
        
        for term in terms {
            if search_query.search_subject {
                let term_query = self.create_text_field_query(self.fields.subject, term)?;
                subqueries.push((Occur::Should, term_query));
            }
            
            if search_query.search_from {
                let term_query = self.create_text_field_query(self.fields.from_address, term)?;
                subqueries.push((Occur::Should, term_query));
            }
            
            if search_query.search_to {
                let term_query = self.create_text_field_query(self.fields.to_addresses, term)?;
                subqueries.push((Occur::Should, term_query));
            }
            
            if search_query.search_body {
                let term_query = self.create_text_field_query(self.fields.body_text, term)?;
                subqueries.push((Occur::Should, term_query));
            }
        }
        
        if subqueries.is_empty() {
            use tantivy::query::AllQuery;
            Ok(Box::new(AllQuery))
        } else {
            Ok(Box::new(BooleanQuery::new(subqueries)))
        }
    }

    fn create_text_field_query(&self, field: Field, term: &str) -> AsgardResult<Box<dyn Query>> {
        use tantivy::query::TermQuery as TQ;
        let term = Term::from_field_text(field, term);
        Ok(Box::new(TQ::new(term, tantivy::schema::IndexRecordOption::Basic)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use crate::message::{Message, MessageHeaders, EmailAddress, MessageImportance};
    use std::collections::HashMap;

    #[test]
    fn test_search_index_creation() {
        let temp_dir = TempDir::new().unwrap();
        let index_dir = temp_dir.path().to_path_buf();
        let _index = TantivySearchIndex::new(index_dir).unwrap();
    }

    #[test]
    fn test_add_and_search_message() {
        let temp_dir = TempDir::new().unwrap();
        let index_dir = temp_dir.path().to_path_buf();
        let mut index = TantivySearchIndex::new(index_dir).unwrap();

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
            date: Some(OffsetDateTime::now_utc()),
            received_date: Some(OffsetDateTime::now_utc()),
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
        let temp_dir = TempDir::new().unwrap();
        let index_dir = temp_dir.path().to_path_buf();
        let mut index = TantivySearchIndex::new(index_dir).unwrap();

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
            date: Some(OffsetDateTime::now_utc()),
            received_date: Some(OffsetDateTime::now_utc()),
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
        let temp_dir = TempDir::new().unwrap();
        let index_dir = temp_dir.path().to_path_buf();
        let index = TantivySearchIndex::new(index_dir).unwrap();
        
        let stats = index.get_stats().unwrap();
        assert_eq!(stats.indexed_documents, 0);
    }
}
