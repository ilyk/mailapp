//! Sync engines for Asgard Mail

// pub mod imap_sync;  // Temporarily disabled due to async trait conflicts
pub mod smtp_send;
pub mod pop3_sync;
pub mod sync_manager;

// pub use imap_sync::ImapSync;  // Temporarily disabled
pub use smtp_send::SmtpSend;
pub use pop3_sync::Pop3Sync;
pub use sync_manager::{SyncManager, SyncEngine};

/// Sync status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncStatus {
    /// Sync is idle
    Idle,
    /// Sync is running
    Running,
    /// Sync is paused
    Paused,
    /// Sync has an error
    Error,
}

/// Sync result
#[derive(Debug, Clone)]
pub struct SyncResult {
    /// Number of messages synced
    pub messages_synced: u32,
    /// Number of new messages
    pub new_messages: u32,
    /// Number of updated messages
    pub updated_messages: u32,
    /// Number of deleted messages
    pub deleted_messages: u32,
    /// Sync duration
    pub duration: std::time::Duration,
    /// Error message if any
    pub error: Option<String>,
}

/// Sync statistics
#[derive(Debug, Clone, Default)]
pub struct SyncStats {
    /// Total sync operations
    pub total_syncs: u64,
    /// Successful syncs
    pub successful_syncs: u64,
    /// Failed syncs
    pub failed_syncs: u64,
    /// Total messages synced
    pub total_messages_synced: u64,
    /// Last sync time
    pub last_sync: Option<time::OffsetDateTime>,
    /// Average sync duration
    pub average_sync_duration: std::time::Duration,
}
