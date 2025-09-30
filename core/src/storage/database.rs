//! Database layer for Asgard Mail

use crate::error::{AsgardError, AsgardResult};
use crate::account::{Account, AccountStats};
use crate::mailbox::{Mailbox, MailboxStats};
use crate::message::{Message, MessageFlags, Attachment, MessagePart};
use rusqlite::{Connection, Result as SqliteResult, Row, params};
use serde_json;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use time::OffsetDateTime;
use uuid::Uuid;

/// Database connection wrapper
pub struct Database {
    connection: Arc<Mutex<Connection>>,
}

impl Database {
    /// Create a new database connection
    pub async fn new(database_path: PathBuf) -> AsgardResult<Self> {
        // Ensure parent directory exists
        if let Some(parent) = database_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let connection = Connection::open(database_path)?;
        
        // Enable WAL mode for better concurrency
        connection.execute("PRAGMA journal_mode=WAL", [])?;
        connection.execute("PRAGMA synchronous=NORMAL", [])?;
        connection.execute("PRAGMA cache_size=10000", [])?;
        connection.execute("PRAGMA temp_store=MEMORY", [])?;
        
        Ok(Self {
            connection: Arc::new(Mutex::new(connection)),
        })
    }

    /// Initialize the database (run migrations)
    pub async fn initialize(&mut self) -> AsgardResult<()> {
        let connection = self.connection.clone();
        let mut migration_manager = crate::storage::migrations::MigrationManager::new(connection);
        
        migration_manager.run_migrations().await?;
        Ok(())
    }

    /// Close the database connection
    pub async fn close(self) -> AsgardResult<()> {
        // Connection will be dropped automatically
        Ok(())
    }

    // Account operations

    /// Create a new account
    pub async fn create_account(&self, account: &Account) -> AsgardResult<()> {
        let connection = self.connection.clone();
        let conn = connection.lock().await;
        
        conn.execute(
            "INSERT INTO accounts (id, account_type, display_name, email, config, status, last_sync, last_error, stats, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            params![
                account.id.to_string(),
                serde_json::to_string(&account.config.account_type)?,
                account.config.display_name,
                account.config.email,
                serde_json::to_string(&account.config)?,
                serde_json::to_string(&account.status)?,
                account.last_sync.map(|dt| dt.unix_timestamp()),
                account.last_error,
                serde_json::to_string(&account.stats)?,
                account.created_at.unix_timestamp(),
                account.updated_at.unix_timestamp(),
            ],
        )?;
        
        Ok(())
    }

    /// Get an account by ID
    pub async fn get_account(&self, account_id: Uuid) -> AsgardResult<Option<Account>> {
        let connection = self.connection.clone();
        let conn = connection.lock().await;
        
        let mut stmt = conn.prepare(
            "SELECT id, account_type, display_name, email, config, status, last_sync, last_error, stats, created_at, updated_at
             FROM accounts WHERE id = ?"
        )?;
        
        let account_result = stmt.query_row([account_id.to_string()], |row| {
            self.row_to_account(row)
        });
        
        match account_result {
            Ok(account) => Ok(Some(account)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// Get all accounts
    pub async fn get_accounts(&self) -> AsgardResult<Vec<Account>> {
        let connection = self.connection.clone();
        let conn = connection.lock().await;
        
        let mut stmt = conn.prepare(
            "SELECT id, account_type, display_name, email, config, status, last_sync, last_error, stats, created_at, updated_at
             FROM accounts ORDER BY created_at"
        )?;
        
        let account_iter = stmt.query_map([], |row| {
            self.row_to_account(row)
        })?;
        
        let mut accounts = Vec::new();
        for account_result in account_iter {
            accounts.push(account_result?);
        }
        
        Ok(accounts)
    }

    /// Update an account
    pub async fn update_account(&self, account: &Account) -> AsgardResult<()> {
        let connection = self.connection.clone();
        let conn = connection.lock().await;
        
        conn.execute(
            "UPDATE accounts SET account_type = ?, display_name = ?, email = ?, config = ?, status = ?, last_sync = ?, last_error = ?, stats = ?, updated_at = ?
             WHERE id = ?",
            params![
                serde_json::to_string(&account.config.account_type)?,
                account.config.display_name,
                account.config.email,
                serde_json::to_string(&account.config)?,
                serde_json::to_string(&account.status)?,
                account.last_sync.map(|dt| dt.unix_timestamp()),
                account.last_error,
                serde_json::to_string(&account.stats)?,
                account.updated_at.unix_timestamp(),
                account.id.to_string(),
            ],
        )?;
        
        Ok(())
    }

    /// Delete an account
    pub async fn delete_account(&self, account_id: Uuid) -> AsgardResult<()> {
        let connection = self.connection.clone();
        let conn = connection.lock().await;
        
        let _changes = conn.execute("DELETE FROM accounts WHERE id = ?", [account_id.to_string()])?;
        Ok(())
    }

    // Mailbox operations

    /// Create a new mailbox
    pub async fn create_mailbox(&self, mailbox: &Mailbox) -> AsgardResult<()> {
        let connection = self.connection.clone();
        let conn = connection.lock().await;
        
        conn.execute(
            "INSERT INTO mailboxes (id, account_id, name, display_name, mailbox_type, parent_id, attributes, stats, settings, created_at, updated_at, last_sync)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            params![
                mailbox.id.to_string(),
                mailbox.account_id.to_string(),
                mailbox.name,
                mailbox.display_name,
                serde_json::to_string(&mailbox.mailbox_type)?,
                mailbox.parent_id.map(|id| id.to_string()),
                serde_json::to_string(&mailbox.attributes)?,
                serde_json::to_string(&mailbox.stats)?,
                serde_json::to_string(&mailbox.settings)?,
                mailbox.created_at.unix_timestamp(),
                mailbox.updated_at.unix_timestamp(),
                mailbox.last_sync.map(|dt| dt.unix_timestamp()),
            ],
        )?;
        
        Ok(())
    }

    /// Get mailboxes for an account
    pub async fn get_mailboxes(&self, account_id: Uuid) -> AsgardResult<Vec<Mailbox>> {
        let connection = self.connection.clone();
        let conn = connection.lock().await;
        
        let mut stmt = conn.prepare(
            "SELECT id, account_id, name, display_name, mailbox_type, parent_id, attributes, stats, settings, created_at, updated_at, last_sync
             FROM mailboxes WHERE account_id = ? ORDER BY name"
        )?;
        
        let mailbox_iter = stmt.query_map([account_id.to_string()], |row| {
            self.row_to_mailbox(row)
        })?;
        
        let mut mailboxes = Vec::new();
        for mailbox_result in mailbox_iter {
            mailboxes.push(mailbox_result?);
        }
        
        Ok(mailboxes)
    }

    /// Get a mailbox by ID
    pub async fn get_mailbox(&self, mailbox_id: Uuid) -> AsgardResult<Option<Mailbox>> {
        let connection = self.connection.clone();
        let conn = connection.lock().await;
        
        let mut stmt = conn.prepare(
            "SELECT id, account_id, name, display_name, mailbox_type, parent_id, attributes, stats, settings, created_at, updated_at, last_sync
             FROM mailboxes WHERE id = ?"
        )?;
        
        let mailbox_result = stmt.query_row([mailbox_id.to_string()], |row| {
            self.row_to_mailbox(row)
        });
        
        match mailbox_result {
            Ok(mailbox) => Ok(Some(mailbox)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// Update a mailbox
    pub async fn update_mailbox(&self, mailbox: &Mailbox) -> AsgardResult<()> {
        let connection = self.connection.clone();
        let conn = connection.lock().await;
        
        conn.execute(
            "UPDATE mailboxes SET name = ?, display_name = ?, mailbox_type = ?, parent_id = ?, attributes = ?, stats = ?, settings = ?, updated_at = ?, last_sync = ?
             WHERE id = ?",
            params![
                mailbox.name,
                mailbox.display_name,
                serde_json::to_string(&mailbox.mailbox_type)?,
                mailbox.parent_id.map(|id| id.to_string()),
                serde_json::to_string(&mailbox.attributes)?,
                serde_json::to_string(&mailbox.stats)?,
                serde_json::to_string(&mailbox.settings)?,
                mailbox.updated_at.unix_timestamp(),
                mailbox.last_sync.map(|dt| dt.unix_timestamp()),
                mailbox.id.to_string(),
            ],
        )?;
        
        Ok(())
    }

    /// Delete a mailbox
    pub async fn delete_mailbox(&self, mailbox_id: Uuid) -> AsgardResult<()> {
        let connection = self.connection.clone();
        let conn = connection.lock().await;
        
        let _changes = conn.execute("DELETE FROM mailboxes WHERE id = ?", [mailbox_id.to_string()])?;
        Ok(())
    }

    // Message operations

    /// Create a new message
    pub async fn create_message(&self, message: &Message) -> AsgardResult<()> {
        let connection = self.connection.clone();
        let mut conn = connection.lock().await;
        
        let tx = conn.transaction()?;
        
        // Insert message
        tx.execute(
            "INSERT INTO messages (id, account_id, mailbox_id, uid, uid_validity, sequence_number, headers, size, thread_id, conversation_id, raw_content, created_at, updated_at, last_sync)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            params![
                message.id.to_string(),
                message.account_id.to_string(),
                message.mailbox_id.to_string(),
                message.uid,
                message.uid_validity,
                message.sequence_number,
                serde_json::to_string(&message.headers)?,
                message.size,
                message.thread_id.map(|id| id.to_string()),
                message.conversation_id,
                message.raw_content,
                message.created_at.unix_timestamp(),
                message.updated_at.unix_timestamp(),
                message.last_sync.map(|dt| dt.unix_timestamp()),
            ],
        )?;
        
        // Insert message flags
        for flag in &message.flags {
            tx.execute(
                "INSERT INTO message_flags (message_id, flag) VALUES (?, ?)",
                params![message.id.to_string(), serde_json::to_string(flag)?],
            )?;
        }
        
        // Insert message labels
        for label in &message.labels {
            tx.execute(
                "INSERT INTO message_labels (message_id, label) VALUES (?, ?)",
                params![message.id.to_string(), label],
            )?;
        }
        
        // Insert message parts
        for part in &message.parts {
            self.insert_message_part(&tx, &message.id, part, None)?;
        }
        
        // Insert attachments
        for attachment in &message.attachments {
            tx.execute(
                "INSERT INTO attachments (id, message_id, part_id, filename, mime_type, size, content_hash, file_path, content, created_at)
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                params![
                    attachment.id.to_string(),
                    attachment.message_id.to_string(),
                    attachment.part_id,
                    attachment.filename,
                    attachment.mime_type,
                    attachment.size,
                    attachment.content_hash,
                    attachment.file_path,
                    attachment.content,
                    attachment.created_at.unix_timestamp(),
                ],
            )?;
        }
        
        tx.commit()?;
        Ok(())
    }

    /// Get messages for a mailbox
    pub async fn get_messages(&self, mailbox_id: Uuid, limit: Option<usize>, offset: Option<usize>) -> AsgardResult<Vec<Message>> {
        let connection = self.connection.clone();
        let conn = connection.lock().await;
        
        let limit = limit.unwrap_or(100);
        let offset = offset.unwrap_or(0);
        
        let mut stmt = conn.prepare(
            "SELECT id, account_id, mailbox_id, uid, uid_validity, sequence_number, headers, size, thread_id, conversation_id, raw_content, created_at, updated_at, last_sync
             FROM messages WHERE mailbox_id = ? ORDER BY created_at DESC LIMIT ? OFFSET ?"
        )?;
        
        let message_iter = stmt.query_map(params![mailbox_id.to_string(), limit, offset], |row| {
            self.row_to_message(row)
        })?;
        
        let mut messages = Vec::new();
        for message_result in message_iter {
            let mut message = message_result?;
            
            // Load flags
            message.flags = self.get_message_flags(&conn, &message.id)?;
            
            // Load labels
            message.labels = self.get_message_labels(&conn, &message.id)?;
            
            // Load parts
            message.parts = self.get_message_parts(&conn, &message.id, None)?;
            
            // Load attachments
            message.attachments = self.get_message_attachments(&conn, &message.id)?;
            
            messages.push(message);
        }
        
        Ok(messages)
    }

    /// Get a message by ID
    pub async fn get_message(&self, message_id: Uuid) -> AsgardResult<Option<Message>> {
        let connection = self.connection.clone();
        let conn = connection.lock().await;
        
        let mut stmt = conn.prepare(
            "SELECT id, account_id, mailbox_id, uid, uid_validity, sequence_number, headers, size, thread_id, conversation_id, raw_content, created_at, updated_at, last_sync
             FROM messages WHERE id = ?"
        )?;
        
        let message_result = stmt.query_row([message_id.to_string()], |row| {
            self.row_to_message(row)
        });
        
        match message_result {
            Ok(mut message) => {
                // Load flags
                message.flags = self.get_message_flags(&conn, &message.id)?;
                
                // Load labels
                message.labels = self.get_message_labels(&conn, &message.id)?;
                
                // Load parts
                message.parts = self.get_message_parts(&conn, &message.id, None)?;
                
                // Load attachments
                message.attachments = self.get_message_attachments(&conn, &message.id)?;
                
                Ok(Some(message))
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// Update a message
    pub async fn update_message(&self, message: &Message) -> AsgardResult<()> {
        let connection = self.connection.clone();
        let mut conn = connection.lock().await;
        
        let tx = conn.transaction()?;
        
        // Update message
        tx.execute(
            "UPDATE messages SET uid = ?, uid_validity = ?, sequence_number = ?, headers = ?, size = ?, thread_id = ?, conversation_id = ?, raw_content = ?, updated_at = ?, last_sync = ?
             WHERE id = ?",
            params![
                message.uid,
                message.uid_validity,
                message.sequence_number,
                serde_json::to_string(&message.headers)?,
                message.size,
                message.thread_id.map(|id| id.to_string()),
                message.conversation_id,
                message.raw_content,
                message.updated_at.unix_timestamp(),
                message.last_sync.map(|dt| dt.unix_timestamp()),
                message.id.to_string(),
            ],
        )?;
        
        // Update flags
        tx.execute("DELETE FROM message_flags WHERE message_id = ?", [message.id.to_string()])?;
        for flag in &message.flags {
            tx.execute(
                "INSERT INTO message_flags (message_id, flag) VALUES (?, ?)",
                params![message.id.to_string(), serde_json::to_string(flag)?],
            )?;
        }
        
        // Update labels
        tx.execute("DELETE FROM message_labels WHERE message_id = ?", [message.id.to_string()])?;
        for label in &message.labels {
            tx.execute(
                "INSERT INTO message_labels (message_id, label) VALUES (?, ?)",
                params![message.id.to_string(), label],
            )?;
        }
        
        tx.commit()?;
        Ok(())
    }

    /// Delete a message
    pub async fn delete_message(&self, message_id: Uuid) -> AsgardResult<()> {
        let connection = self.connection.clone();
        let conn = connection.lock().await;
        
        let _changes = conn.execute("DELETE FROM messages WHERE id = ?", [message_id.to_string()])?;
        Ok(())
    }

    // Helper methods

    fn row_to_account(&self, row: &Row) -> SqliteResult<Account> {
        let id: String = row.get(0)?;
        let account_type: String = row.get(1)?;
        let display_name: String = row.get(2)?;
        let email: String = row.get(3)?;
        let config: String = row.get(4)?;
        let status: String = row.get(5)?;
        let last_sync: Option<i64> = row.get(6)?;
        let last_error: Option<String> = row.get(7)?;
        let stats: String = row.get(8)?;
        let created_at: i64 = row.get(9)?;
        let updated_at: i64 = row.get(10)?;

        Ok(Account {
            id: Uuid::parse_str(&id).map_err(|_| rusqlite::Error::InvalidColumnType(0, "UUID".to_string(), rusqlite::types::Type::Text))?,
            config: serde_json::from_str(&config).map_err(|e| rusqlite::Error::InvalidColumnType(4, "Config".to_string(), rusqlite::types::Type::Text))?,
            status: serde_json::from_str(&status).map_err(|e| rusqlite::Error::InvalidColumnType(5, "Status".to_string(), rusqlite::types::Type::Text))?,
            last_sync: last_sync.map(|ts| OffsetDateTime::from_unix_timestamp(ts).unwrap_or_else(|_| OffsetDateTime::now_utc())),
            last_error,
            created_at: OffsetDateTime::from_unix_timestamp(created_at).unwrap_or_else(|_| OffsetDateTime::now_utc()),
            updated_at: OffsetDateTime::from_unix_timestamp(updated_at).unwrap_or_else(|_| OffsetDateTime::now_utc()),
            stats: serde_json::from_str(&stats).map_err(|e| rusqlite::Error::InvalidColumnType(8, "Stats".to_string(), rusqlite::types::Type::Text))?,
        })
    }

    fn row_to_mailbox(&self, row: &Row) -> SqliteResult<Mailbox> {
        let id: String = row.get(0)?;
        let account_id: String = row.get(1)?;
        let name: String = row.get(2)?;
        let display_name: String = row.get(3)?;
        let mailbox_type: String = row.get(4)?;
        let parent_id: Option<String> = row.get(5)?;
        let attributes: String = row.get(6)?;
        let stats: String = row.get(7)?;
        let settings: String = row.get(8)?;
        let created_at: i64 = row.get(9)?;
        let updated_at: i64 = row.get(10)?;
        let last_sync: Option<i64> = row.get(11)?;

        Ok(Mailbox {
            id: Uuid::parse_str(&id).map_err(|_| rusqlite::Error::InvalidColumnType(0, "UUID".to_string(), rusqlite::types::Type::Text))?,
            account_id: Uuid::parse_str(&account_id).map_err(|_| rusqlite::Error::InvalidColumnType(1, "UUID".to_string(), rusqlite::types::Type::Text))?,
            name,
            display_name,
            mailbox_type: serde_json::from_str(&mailbox_type).map_err(|e| rusqlite::Error::InvalidColumnType(4, "MailboxType".to_string(), rusqlite::types::Type::Text))?,
            parent_id: parent_id.map(|id| Uuid::parse_str(&id).unwrap()),
            attributes: serde_json::from_str(&attributes).map_err(|e| rusqlite::Error::InvalidColumnType(6, "Attributes".to_string(), rusqlite::types::Type::Text))?,
            stats: serde_json::from_str(&stats).map_err(|e| rusqlite::Error::InvalidColumnType(7, "Stats".to_string(), rusqlite::types::Type::Text))?,
            settings: serde_json::from_str(&settings).map_err(|e| rusqlite::Error::InvalidColumnType(8, "Settings".to_string(), rusqlite::types::Type::Text))?,
            created_at: OffsetDateTime::from_unix_timestamp(created_at).unwrap_or_else(|_| OffsetDateTime::now_utc()),
            updated_at: OffsetDateTime::from_unix_timestamp(updated_at).unwrap_or_else(|_| OffsetDateTime::now_utc()),
            last_sync: last_sync.map(|ts| OffsetDateTime::from_unix_timestamp(ts).unwrap_or_else(|_| OffsetDateTime::now_utc())),
        })
    }

    fn row_to_message(&self, row: &Row) -> SqliteResult<Message> {
        let id: String = row.get(0)?;
        let account_id: String = row.get(1)?;
        let mailbox_id: String = row.get(2)?;
        let uid: Option<u32> = row.get(3)?;
        let uid_validity: Option<u32> = row.get(4)?;
        let sequence_number: Option<u32> = row.get(5)?;
        let headers: String = row.get(6)?;
        let size: usize = row.get(7)?;
        let thread_id: Option<String> = row.get(8)?;
        let conversation_id: Option<String> = row.get(9)?;
        let raw_content: Option<Vec<u8>> = row.get(10)?;
        let created_at: i64 = row.get(11)?;
        let updated_at: i64 = row.get(12)?;
        let last_sync: Option<i64> = row.get(13)?;

        Ok(Message {
            id: Uuid::parse_str(&id).map_err(|_| rusqlite::Error::InvalidColumnType(0, "UUID".to_string(), rusqlite::types::Type::Text))?,
            account_id: Uuid::parse_str(&account_id).map_err(|_| rusqlite::Error::InvalidColumnType(1, "UUID".to_string(), rusqlite::types::Type::Text))?,
            mailbox_id: Uuid::parse_str(&mailbox_id).map_err(|_| rusqlite::Error::InvalidColumnType(2, "UUID".to_string(), rusqlite::types::Type::Text))?,
            uid,
            uid_validity,
            sequence_number,
            headers: serde_json::from_str(&headers).map_err(|e| rusqlite::Error::InvalidColumnType(6, "Headers".to_string(), rusqlite::types::Type::Text))?,
            flags: vec![], // Will be loaded separately
            labels: vec![], // Will be loaded separately
            parts: vec![], // Will be loaded separately
            attachments: vec![], // Will be loaded separately
            thread_id: thread_id.map(|id| Uuid::parse_str(&id).unwrap()),
            conversation_id,
            size,
            raw_content,
            created_at: OffsetDateTime::from_unix_timestamp(created_at).unwrap_or_else(|_| OffsetDateTime::now_utc()),
            updated_at: OffsetDateTime::from_unix_timestamp(updated_at).unwrap_or_else(|_| OffsetDateTime::now_utc()),
            last_sync: last_sync.map(|ts| OffsetDateTime::from_unix_timestamp(ts).unwrap_or_else(|_| OffsetDateTime::now_utc())),
        })
    }

    fn get_message_flags(&self, conn: &Connection, message_id: &Uuid) -> SqliteResult<Vec<MessageFlags>> {
        let mut stmt = conn.prepare("SELECT flag FROM message_flags WHERE message_id = ?")?;
        let flag_iter = stmt.query_map([message_id.to_string()], |row| {
            let flag_str: String = row.get(0)?;
            serde_json::from_str(&flag_str).map_err(|e| rusqlite::Error::InvalidColumnType(0, "MessageFlags".to_string(), rusqlite::types::Type::Text))
        })?;
        
        let mut flags = Vec::new();
        for flag_result in flag_iter {
            flags.push(flag_result?);
        }
        
        Ok(flags)
    }

    fn get_message_labels(&self, conn: &Connection, message_id: &Uuid) -> SqliteResult<Vec<String>> {
        let mut stmt = conn.prepare("SELECT label FROM message_labels WHERE message_id = ?")?;
        let label_iter = stmt.query_map([message_id.to_string()], |row| {
            let label: String = row.get(0)?;
            Ok(label)
        })?;
        
        let mut labels = Vec::new();
        for label_result in label_iter {
            labels.push(label_result?);
        }
        
        Ok(labels)
    }

    fn get_message_parts(&self, conn: &Connection, message_id: &Uuid, parent_id: Option<&str>) -> SqliteResult<Vec<MessagePart>> {
        let mut stmt = conn.prepare("SELECT id, part_id, part_type, mime_type, disposition, filename, size, encoding, content_id, content_location, content FROM message_parts WHERE message_id = ? AND parent_id IS ?")?;
        let part_iter = stmt.query_map(params![message_id.to_string(), parent_id], |row| {
            let id: String = row.get(0)?;
            let part_id: String = row.get(1)?;
            let part_type: String = row.get(2)?;
            let mime_type: String = row.get(3)?;
            let disposition: Option<String> = row.get(4)?;
            let filename: Option<String> = row.get(5)?;
            let size: usize = row.get(6)?;
            let encoding: Option<String> = row.get(7)?;
            let content_id: Option<String> = row.get(8)?;
            let content_location: Option<String> = row.get(9)?;
            let content: Option<Vec<u8>> = row.get(10)?;

            Ok(MessagePart {
                id,
                part_type: serde_json::from_str(&part_type).unwrap_or(crate::message::MessagePartType::Other),
                mime_type,
                disposition,
                filename,
                size,
                encoding,
                content_id,
                content_location,
                content,
                children: vec![], // Will be loaded recursively
            })
        })?;
        
        let mut parts = Vec::new();
        for part_result in part_iter {
            let mut part = part_result?;
            
            // Load child parts recursively
            part.children = self.get_message_parts(conn, message_id, Some(&part.id))?;
            
            parts.push(part);
        }
        
        Ok(parts)
    }

    fn get_message_attachments(&self, conn: &Connection, message_id: &Uuid) -> SqliteResult<Vec<Attachment>> {
        let mut stmt = conn.prepare("SELECT id, part_id, filename, mime_type, size, content_hash, file_path, content, created_at FROM attachments WHERE message_id = ?")?;
        let attachment_iter = stmt.query_map([message_id.to_string()], |row| {
            let id: String = row.get(0)?;
            let part_id: String = row.get(1)?;
            let filename: String = row.get(2)?;
            let mime_type: String = row.get(3)?;
            let size: usize = row.get(4)?;
            let content_hash: String = row.get(5)?;
            let file_path: Option<String> = row.get(6)?;
            let content: Option<Vec<u8>> = row.get(7)?;
            let created_at: i64 = row.get(8)?;

            Ok(Attachment {
                id: Uuid::parse_str(&id).unwrap(),
                message_id: *message_id,
                part_id,
                filename,
                mime_type,
                size,
                content_hash,
                file_path,
                content,
                created_at: OffsetDateTime::from_unix_timestamp(created_at).unwrap_or_else(|_| OffsetDateTime::now_utc()),
            })
        })?;
        
        let mut attachments = Vec::new();
        for attachment_result in attachment_iter {
            attachments.push(attachment_result?);
        }
        
        Ok(attachments)
    }

    fn insert_message_part(&self, tx: &rusqlite::Transaction, message_id: &Uuid, part: &MessagePart, parent_id: Option<&str>) -> SqliteResult<()> {
        tx.execute(
            "INSERT INTO message_parts (id, message_id, part_id, part_type, mime_type, disposition, filename, size, encoding, content_id, content_location, content, parent_id)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            params![
                part.id,
                message_id.to_string(),
                part.id,
                serde_json::to_string(&part.part_type).map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?,
                part.mime_type,
                part.disposition,
                part.filename,
                part.size,
                part.encoding,
                part.content_id,
                part.content_location,
                part.content,
                parent_id,
            ],
        )?;
        
        // Insert child parts recursively
        for child_part in &part.children {
            self.insert_message_part(tx, message_id, child_part, Some(&part.id))?;
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use crate::account::{Account, AccountType, GmailOAuthConfig};

    #[tokio::test]
    async fn test_database_creation() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let database = Database::new(db_path).await.unwrap();
        database.initialize().await.unwrap();
    }

    #[tokio::test]
    async fn test_account_operations() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let database = Database::new(db_path).await.unwrap();
        database.initialize().await.unwrap();

        let oauth_config = GmailOAuthConfig {
            client_id: "test-client-id".to_string(),
            client_secret: "test-client-secret".to_string(),
            access_token: None,
            refresh_token: None,
            token_expires_at: None,
            scopes: vec!["https://www.googleapis.com/auth/gmail.readonly".to_string()],
        };

        let account = Account::new_gmail(
            "test@gmail.com".to_string(),
            Some("Test Account".to_string()),
            oauth_config,
        ).unwrap();

        // Create account
        database.create_account(&account).await.unwrap();

        // Get account
        let retrieved_account = database.get_account(account.id).await.unwrap().unwrap();
        assert_eq!(retrieved_account.email(), account.email());
        assert_eq!(retrieved_account.display_name(), account.display_name());

        // Update account
        let mut updated_account = account.clone();
        updated_account.update_last_sync();
        database.update_account(&updated_account).await.unwrap();

        // Get all accounts
        let accounts = database.get_accounts().await.unwrap();
        assert_eq!(accounts.len(), 1);

        // Delete account
        database.delete_account(account.id).await.unwrap();
        let deleted_account = database.get_account(account.id).await.unwrap();
        assert!(deleted_account.is_none());
    }
}
