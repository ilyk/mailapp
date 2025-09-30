//! Database migrations for Asgard Mail

use crate::error::{AsgardError, AsgardResult};
use rusqlite::{Connection, Result as SqliteResult};
use std::sync::Arc;
use tokio::sync::Mutex;

/// Migration manager for database schema updates
pub struct MigrationManager {
    connection: Arc<Mutex<Connection>>,
}

impl MigrationManager {
    /// Create a new migration manager
    pub fn new(connection: Arc<Mutex<Connection>>) -> Self {
        Self { connection }
    }

    /// Run all pending migrations
    pub async fn run_migrations(&mut self) -> AsgardResult<()> {
        self.create_migrations_table().await?;
        
        let migrations = self.get_migrations();
        for migration in migrations {
            if !self.is_migration_applied(migration.name()).await? {
                tracing::info!("Applying migration: {}", migration.name());
                let mut conn = self.connection.lock().await;
                migration.apply(&mut *conn)?;
                drop(conn);
                self.mark_migration_applied(migration.name()).await?;
                tracing::info!("Migration applied successfully: {}", migration.name());
            }
        }
        
        Ok(())
    }

    /// Create the migrations tracking table
    async fn create_migrations_table(&mut self) -> SqliteResult<()> {
        let conn = self.connection.lock().await;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS migrations (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                applied_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;
        Ok(())
    }

    /// Check if a migration has been applied
    async fn is_migration_applied(&self, migration_name: &str) -> SqliteResult<bool> {
        let conn = self.connection.lock().await;
        let mut stmt = conn.prepare("SELECT COUNT(*) FROM migrations WHERE name = ?")?;
        let count: i64 = stmt.query_row([migration_name], |row| row.get(0))?;
        Ok(count > 0)
    }

    /// Mark a migration as applied
    async fn mark_migration_applied(&mut self, migration_name: &str) -> SqliteResult<()> {
        let conn = self.connection.lock().await;
        conn.execute(
            "INSERT INTO migrations (name) VALUES (?)",
            [migration_name],
        )?;
        Ok(())
    }

    /// Get all available migrations
    fn get_migrations(&self) -> Vec<Box<dyn Migration>> {
        vec![
            Box::new(CreateAccountsTable),
            Box::new(CreateMailboxesTable),
            Box::new(CreateMessagesTable),
            Box::new(CreateAttachmentsTable),
            Box::new(CreateMessagePartsTable),
            Box::new(CreateMessageThreadsTable),
            Box::new(CreateMessageFlagsTable),
            Box::new(CreateMessageLabelsTable),
            Box::new(CreateSearchIndexTable),
            Box::new(CreateCacheTable),
            Box::new(AddIndexes),
        ]
    }
}

/// Trait for database migrations
trait Migration {
    fn name(&self) -> &str;
    fn apply(&self, connection: &mut Connection) -> SqliteResult<()>;
}

/// Migration: Create accounts table
struct CreateAccountsTable;

impl Migration for CreateAccountsTable {
    fn name(&self) -> &str {
        "create_accounts_table"
    }

    fn apply(&self, connection: &mut Connection) -> SqliteResult<()> {
        connection.execute(
            "CREATE TABLE accounts (
                id TEXT PRIMARY KEY,
                account_type TEXT NOT NULL,
                display_name TEXT NOT NULL,
                email TEXT NOT NULL,
                config TEXT NOT NULL,
                status TEXT NOT NULL,
                last_sync DATETIME,
                last_error TEXT,
                stats TEXT NOT NULL,
                created_at DATETIME NOT NULL,
                updated_at DATETIME NOT NULL
            )",
            [],
        )?;
        Ok(())
    }
}

/// Migration: Create mailboxes table
struct CreateMailboxesTable;

impl Migration for CreateMailboxesTable {
    fn name(&self) -> &str {
        "create_mailboxes_table"
    }

    fn apply(&self, connection: &mut Connection) -> SqliteResult<()> {
        connection.execute(
            "CREATE TABLE mailboxes (
                id TEXT PRIMARY KEY,
                account_id TEXT NOT NULL,
                name TEXT NOT NULL,
                display_name TEXT NOT NULL,
                mailbox_type TEXT NOT NULL,
                parent_id TEXT,
                attributes TEXT NOT NULL,
                stats TEXT NOT NULL,
                settings TEXT NOT NULL,
                created_at DATETIME NOT NULL,
                updated_at DATETIME NOT NULL,
                last_sync DATETIME,
                FOREIGN KEY (account_id) REFERENCES accounts (id) ON DELETE CASCADE,
                FOREIGN KEY (parent_id) REFERENCES mailboxes (id) ON DELETE CASCADE
            )",
            [],
        )?;
        Ok(())
    }
}

/// Migration: Create messages table
struct CreateMessagesTable;

impl Migration for CreateMessagesTable {
    fn name(&self) -> &str {
        "create_messages_table"
    }

    fn apply(&self, connection: &mut Connection) -> SqliteResult<()> {
        connection.execute(
            "CREATE TABLE messages (
                id TEXT PRIMARY KEY,
                account_id TEXT NOT NULL,
                mailbox_id TEXT NOT NULL,
                uid INTEGER,
                uid_validity INTEGER,
                sequence_number INTEGER,
                headers TEXT NOT NULL,
                size INTEGER NOT NULL,
                thread_id TEXT,
                conversation_id TEXT,
                raw_content BLOB,
                created_at DATETIME NOT NULL,
                updated_at DATETIME NOT NULL,
                last_sync DATETIME,
                FOREIGN KEY (account_id) REFERENCES accounts (id) ON DELETE CASCADE,
                FOREIGN KEY (mailbox_id) REFERENCES mailboxes (id) ON DELETE CASCADE,
                FOREIGN KEY (thread_id) REFERENCES message_threads (id) ON DELETE SET NULL
            )",
            [],
        )?;
        Ok(())
    }
}

/// Migration: Create attachments table
struct CreateAttachmentsTable;

impl Migration for CreateAttachmentsTable {
    fn name(&self) -> &str {
        "create_attachments_table"
    }

    fn apply(&self, connection: &mut Connection) -> SqliteResult<()> {
        connection.execute(
            "CREATE TABLE attachments (
                id TEXT PRIMARY KEY,
                message_id TEXT NOT NULL,
                part_id TEXT NOT NULL,
                filename TEXT NOT NULL,
                mime_type TEXT NOT NULL,
                size INTEGER NOT NULL,
                content_hash TEXT NOT NULL,
                file_path TEXT,
                content BLOB,
                created_at DATETIME NOT NULL,
                FOREIGN KEY (message_id) REFERENCES messages (id) ON DELETE CASCADE
            )",
            [],
        )?;
        Ok(())
    }
}

/// Migration: Create message parts table
struct CreateMessagePartsTable;

impl Migration for CreateMessagePartsTable {
    fn name(&self) -> &str {
        "create_message_parts_table"
    }

    fn apply(&self, connection: &mut Connection) -> SqliteResult<()> {
        connection.execute(
            "CREATE TABLE message_parts (
                id TEXT PRIMARY KEY,
                message_id TEXT NOT NULL,
                part_id TEXT NOT NULL,
                part_type TEXT NOT NULL,
                mime_type TEXT NOT NULL,
                disposition TEXT,
                filename TEXT,
                size INTEGER NOT NULL,
                encoding TEXT,
                content_id TEXT,
                content_location TEXT,
                content BLOB,
                parent_id TEXT,
                FOREIGN KEY (message_id) REFERENCES messages (id) ON DELETE CASCADE,
                FOREIGN KEY (parent_id) REFERENCES message_parts (id) ON DELETE CASCADE
            )",
            [],
        )?;
        Ok(())
    }
}

/// Migration: Create message threads table
struct CreateMessageThreadsTable;

impl Migration for CreateMessageThreadsTable {
    fn name(&self) -> &str {
        "create_message_threads_table"
    }

    fn apply(&self, connection: &mut Connection) -> SqliteResult<()> {
        connection.execute(
            "CREATE TABLE message_threads (
                id TEXT PRIMARY KEY,
                account_id TEXT NOT NULL,
                subject TEXT NOT NULL,
                participants TEXT NOT NULL,
                created_at DATETIME NOT NULL,
                last_message_at DATETIME,
                unread_count INTEGER NOT NULL DEFAULT 0,
                size INTEGER NOT NULL DEFAULT 0,
                FOREIGN KEY (account_id) REFERENCES accounts (id) ON DELETE CASCADE
            )",
            [],
        )?;
        Ok(())
    }
}

/// Migration: Create message flags table
struct CreateMessageFlagsTable;

impl Migration for CreateMessageFlagsTable {
    fn name(&self) -> &str {
        "create_message_flags_table"
    }

    fn apply(&self, connection: &mut Connection) -> SqliteResult<()> {
        connection.execute(
            "CREATE TABLE message_flags (
                message_id TEXT NOT NULL,
                flag TEXT NOT NULL,
                PRIMARY KEY (message_id, flag),
                FOREIGN KEY (message_id) REFERENCES messages (id) ON DELETE CASCADE
            )",
            [],
        )?;
        Ok(())
    }
}

/// Migration: Create message labels table
struct CreateMessageLabelsTable;

impl Migration for CreateMessageLabelsTable {
    fn name(&self) -> &str {
        "create_message_labels_table"
    }

    fn apply(&self, connection: &mut Connection) -> SqliteResult<()> {
        connection.execute(
            "CREATE TABLE message_labels (
                message_id TEXT NOT NULL,
                label TEXT NOT NULL,
                PRIMARY KEY (message_id, label),
                FOREIGN KEY (message_id) REFERENCES messages (id) ON DELETE CASCADE
            )",
            [],
        )?;
        Ok(())
    }
}

/// Migration: Create search index table
struct CreateSearchIndexTable;

impl Migration for CreateSearchIndexTable {
    fn name(&self) -> &str {
        "create_search_index_table"
    }

    fn apply(&self, connection: &mut Connection) -> SqliteResult<()> {
        connection.execute(
            "CREATE TABLE search_index (
                message_id TEXT PRIMARY KEY,
                subject TEXT NOT NULL,
                from_address TEXT NOT NULL,
                to_addresses TEXT NOT NULL,
                body_text TEXT,
                body_html TEXT,
                indexed_at DATETIME NOT NULL,
                FOREIGN KEY (message_id) REFERENCES messages (id) ON DELETE CASCADE
            )",
            [],
        )?;
        Ok(())
    }
}

/// Migration: Create cache table
struct CreateCacheTable;

impl Migration for CreateCacheTable {
    fn name(&self) -> &str {
        "create_cache_table"
    }

    fn apply(&self, connection: &mut Connection) -> SqliteResult<()> {
        connection.execute(
            "CREATE TABLE cache (
                key TEXT PRIMARY KEY,
                value BLOB NOT NULL,
                content_type TEXT,
                size INTEGER NOT NULL,
                created_at DATETIME NOT NULL,
                accessed_at DATETIME NOT NULL,
                expires_at DATETIME
            )",
            [],
        )?;
        Ok(())
    }
}

/// Migration: Add indexes for better performance
struct AddIndexes;

impl Migration for AddIndexes {
    fn name(&self) -> &str {
        "add_indexes"
    }

    fn apply(&self, connection: &mut Connection) -> SqliteResult<()> {
        // Account indexes
        connection.execute("CREATE INDEX IF NOT EXISTS idx_accounts_email ON accounts (email)", [])?;
        connection.execute("CREATE INDEX IF NOT EXISTS idx_accounts_status ON accounts (status)", [])?;
        
        // Mailbox indexes
        connection.execute("CREATE INDEX IF NOT EXISTS idx_mailboxes_account_id ON mailboxes (account_id)", [])?;
        connection.execute("CREATE INDEX IF NOT EXISTS idx_mailboxes_parent_id ON mailboxes (parent_id)", [])?;
        connection.execute("CREATE INDEX IF NOT EXISTS idx_mailboxes_type ON mailboxes (mailbox_type)", [])?;
        
        // Message indexes
        connection.execute("CREATE INDEX IF NOT EXISTS idx_messages_account_id ON messages (account_id)", [])?;
        connection.execute("CREATE INDEX IF NOT EXISTS idx_messages_mailbox_id ON messages (mailbox_id)", [])?;
        connection.execute("CREATE INDEX IF NOT EXISTS idx_messages_thread_id ON messages (thread_id)", [])?;
        connection.execute("CREATE INDEX IF NOT EXISTS idx_messages_conversation_id ON messages (conversation_id)", [])?;
        connection.execute("CREATE INDEX IF NOT EXISTS idx_messages_created_at ON messages (created_at)", [])?;
        
        // Attachment indexes
        connection.execute("CREATE INDEX IF NOT EXISTS idx_attachments_message_id ON attachments (message_id)", [])?;
        connection.execute("CREATE INDEX IF NOT EXISTS idx_attachments_content_hash ON attachments (content_hash)", [])?;
        
        // Message part indexes
        connection.execute("CREATE INDEX IF NOT EXISTS idx_message_parts_message_id ON message_parts (message_id)", [])?;
        connection.execute("CREATE INDEX IF NOT EXISTS idx_message_parts_parent_id ON message_parts (parent_id)", [])?;
        
        // Thread indexes
        connection.execute("CREATE INDEX IF NOT EXISTS idx_message_threads_account_id ON message_threads (account_id)", [])?;
        connection.execute("CREATE INDEX IF NOT EXISTS idx_message_threads_last_message_at ON message_threads (last_message_at)", [])?;
        
        // Flag indexes
        connection.execute("CREATE INDEX IF NOT EXISTS idx_message_flags_message_id ON message_flags (message_id)", [])?;
        connection.execute("CREATE INDEX IF NOT EXISTS idx_message_flags_flag ON message_flags (flag)", [])?;
        
        // Label indexes
        connection.execute("CREATE INDEX IF NOT EXISTS idx_message_labels_message_id ON message_labels (message_id)", [])?;
        connection.execute("CREATE INDEX IF NOT EXISTS idx_message_labels_label ON message_labels (label)", [])?;
        
        // Search index
        connection.execute("CREATE INDEX IF NOT EXISTS idx_search_index_subject ON search_index (subject)", [])?;
        connection.execute("CREATE INDEX IF NOT EXISTS idx_search_index_from_address ON search_index (from_address)", [])?;
        
        // Cache indexes
        connection.execute("CREATE INDEX IF NOT EXISTS idx_cache_accessed_at ON cache (accessed_at)", [])?;
        connection.execute("CREATE INDEX IF NOT EXISTS idx_cache_expires_at ON cache (expires_at)", [])?;
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_migration_manager() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let connection = Connection::open(db_path).unwrap();
        let mut migration_manager = MigrationManager::new(connection);
        
        // Run migrations
        migration_manager.run_migrations().unwrap();
        
        // Check that migrations table was created
        let mut stmt = migration_manager.connection.prepare("SELECT COUNT(*) FROM migrations").unwrap();
        let count: i64 = stmt.query_row([], |row| row.get(0)).unwrap();
        assert!(count > 0);
    }

    #[test]
    fn test_migration_idempotency() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let connection = Connection::open(db_path).unwrap();
        let mut migration_manager = MigrationManager::new(connection);
        
        // Run migrations twice
        migration_manager.run_migrations().unwrap();
        migration_manager.run_migrations().unwrap();
        
        // Should not fail on second run
        let mut stmt = migration_manager.connection.prepare("SELECT COUNT(*) FROM migrations").unwrap();
        let count: i64 = stmt.query_row([], |row| row.get(0)).unwrap();
        assert!(count > 0);
    }
}
