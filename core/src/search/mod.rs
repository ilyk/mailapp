//! Search functionality for Asgard Mail

// pub mod tantivy_index;  // Temporarily disabled due to zstd-safe conflicts
pub mod simple_index;

// pub use tantivy_index::TantivySearchIndex;  // Temporarily disabled
pub use simple_index::SimpleSearchIndex;

/// Search query
#[derive(Debug, Clone)]
pub struct SearchQuery {
    /// Query text
    pub query: String,
    /// Account ID to search in (None for all accounts)
    pub account_id: Option<uuid::Uuid>,
    /// Mailbox ID to search in (None for all mailboxes)
    pub mailbox_id: Option<uuid::Uuid>,
    /// Maximum number of results
    pub limit: Option<usize>,
    /// Offset for pagination
    pub offset: Option<usize>,
    /// Search in subject
    pub search_subject: bool,
    /// Search in from address
    pub search_from: bool,
    /// Search in to address
    pub search_to: bool,
    /// Search in message body
    pub search_body: bool,
    /// Date range filter
    pub date_range: Option<DateRange>,
    /// Has attachments filter
    pub has_attachments: Option<bool>,
    /// Is read filter
    pub is_read: Option<bool>,
    /// Is flagged filter
    pub is_flagged: Option<bool>,
}

/// Date range for search filtering
#[derive(Debug, Clone)]
pub struct DateRange {
    /// Start date
    pub start: time::OffsetDateTime,
    /// End date
    pub end: time::OffsetDateTime,
}

/// Search result
#[derive(Debug, Clone)]
pub struct SearchResult {
    /// Message ID
    pub message_id: uuid::Uuid,
    /// Account ID
    pub account_id: uuid::Uuid,
    /// Mailbox ID
    pub mailbox_id: uuid::Uuid,
    /// Relevance score
    pub score: f32,
    /// Highlighted text snippets
    pub snippets: Vec<String>,
}

/// Search statistics
#[derive(Debug, Clone, Default)]
pub struct SearchStats {
    /// Total number of results
    pub total_results: usize,
    /// Search time in milliseconds
    pub search_time_ms: u64,
    /// Index size in bytes
    pub index_size_bytes: u64,
    /// Number of indexed documents
    pub indexed_documents: usize,
}

impl Default for SearchQuery {
    fn default() -> Self {
        Self {
            query: String::new(),
            account_id: None,
            mailbox_id: None,
            limit: Some(100),
            offset: Some(0),
            search_subject: true,
            search_from: true,
            search_to: true,
            search_body: true,
            date_range: None,
            has_attachments: None,
            is_read: None,
            is_flagged: None,
        }
    }
}
