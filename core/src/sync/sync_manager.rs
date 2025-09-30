//! Sync manager for coordinating all sync operations

use crate::error::{AsgardError, AsgardResult};
use crate::account::Account;
use crate::mailbox::Mailbox;
use crate::message::Message;
use crate::storage::StorageManager;
use crate::search::SimpleSearchIndex;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use tokio::time::{interval, Duration};
use tracing::{info, warn, error};
use uuid::Uuid;

use super::{SmtpSend, Pop3Sync, SyncStatus, SyncResult, SyncStats};
// use super::{ImapSync, SmtpSend, Pop3Sync, SyncStatus, SyncResult, SyncStats};  // ImapSync temporarily disabled

/// Sync manager for coordinating all sync operations
pub struct SyncManager {
    /// Storage manager
    storage: Arc<Mutex<StorageManager>>,
    /// Search index
    search_index: Arc<Mutex<SimpleSearchIndex>>,
    /// Active sync engines
    sync_engines: Arc<RwLock<HashMap<Uuid, Box<dyn SyncEngine + Send + Sync>>>>,
    /// Sync statistics
    stats: Arc<RwLock<SyncStats>>,
    /// Sync interval
    sync_interval: Duration,
    /// Background sync task handle
    background_task: Option<tokio::task::JoinHandle<()>>,
}

/// Trait for sync engines
#[async_trait::async_trait]
pub trait SyncEngine {
    /// Get the account ID
    fn account_id(&self) -> Uuid;
    
    /// Get sync status
    fn status(&self) -> SyncStatus;
    
    /// Get last sync result
    fn last_sync_result(&self) -> Option<&SyncResult>;
    
    /// Connect to the server
    async fn connect(&mut self) -> AsgardResult<()>;
    
    /// Disconnect from the server
    async fn disconnect(&mut self) -> AsgardResult<()>;
    
    /// Sync mailboxes
    async fn sync_mailboxes(&mut self) -> AsgardResult<Vec<Mailbox>>;
    
    /// Sync messages in a mailbox
    async fn sync_mailbox_messages(&mut self, mailbox: &Mailbox) -> AsgardResult<Vec<Message>>;
}

/// Wrapper for IMAP sync engine
// Temporarily disabled due to async trait conflicts
// pub struct ImapSyncEngine {
//     engine: ImapSync,
// }

// Temporarily disabled due to async trait conflicts
// #[async_trait::async_trait]
// impl SyncEngine for ImapSyncEngine {
//     fn account_id(&self) -> Uuid {
//         self.engine.account_id()
//     }
//     
//     fn status(&self) -> SyncStatus {
//         self.engine.status()
//     }
//     
//     fn last_sync_result(&self) -> Option<&SyncResult> {
//         self.engine.last_sync_result()
//     }
//     
//     async fn connect(&mut self) -> AsgardResult<()> {
//         self.engine.connect().await
//     }
//     
//     async fn disconnect(&mut self) -> AsgardResult<()> {
//         self.engine.disconnect().await
//     }
//     
//     async fn sync_mailboxes(&mut self) -> AsgardResult<Vec<Mailbox>> {
//         self.engine.sync_mailboxes().await
//     }
//     
//     async fn sync_mailbox_messages(&mut self, mailbox: &Mailbox) -> AsgardResult<Vec<Message>> {
//         self.engine.sync_mailbox_messages(mailbox).await
//     }
// }

impl SyncManager {
    /// Create a new sync manager
    pub fn new(
        storage: Arc<Mutex<StorageManager>>,
        search_index: Arc<Mutex<SimpleSearchIndex>>,
        sync_interval: Duration,
    ) -> Self {
        Self {
            storage,
            search_index,
            sync_engines: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(SyncStats::default())),
            sync_interval,
            background_task: None,
        }
    }

    /// Add an account for syncing
    pub async fn add_account(&self, account: Account) -> AsgardResult<()> {
        let account_id = account.id;
        
        // Create appropriate sync engine based on account type
        let sync_engine: Box<dyn SyncEngine + Send + Sync> = match account.account_type() {
            crate::account::AccountType::Gmail | crate::account::AccountType::ImapSmtp => {
                // ImapSyncEngine temporarily disabled due to async trait conflicts
                // For now, use POP3 as fallback
                Box::new(Pop3Sync::new(account.clone()))
            }
            crate::account::AccountType::Pop3 => {
                Box::new(Pop3Sync::new(account.clone()))
            }
        };
        
        // Store account in database
        {
            let mut storage = self.storage.lock().await;
            storage.database_mut().create_account(&account).await?;
        }
        
        // Add sync engine
        {
            let mut engines = self.sync_engines.write().await;
            engines.insert(account_id, sync_engine);
        }
        
        info!("Added account for syncing: {}", account_id);
        Ok(())
    }

    /// Remove an account from syncing
    pub async fn remove_account(&self, account_id: Uuid) -> AsgardResult<()> {
        // Disconnect and remove sync engine
        {
            let mut engines = self.sync_engines.write().await;
            if let Some(mut engine) = engines.remove(&account_id) {
                if let Err(e) = engine.disconnect().await {
                    warn!("Failed to disconnect sync engine for account {}: {}", account_id, e);
                }
            }
        }
        
        // Remove account from database
        {
            let mut storage = self.storage.lock().await;
            storage.database_mut().delete_account(account_id).await?;
        }
        
        info!("Removed account from syncing: {}", account_id);
        Ok(())
    }

    /// Start background sync
    pub async fn start_background_sync(&mut self) -> AsgardResult<()> {
        if self.background_task.is_some() {
            return Err(AsgardError::invalid_state("Background sync already running"));
        }
        
        let sync_engines = self.sync_engines.clone();
        let storage = self.storage.clone();
        let search_index = self.search_index.clone();
        let stats = self.stats.clone();
        let sync_interval = self.sync_interval;
        
        let task = tokio::spawn(async move {
            let mut interval = interval(sync_interval);
            
            loop {
                interval.tick().await;
                
                // Get account IDs to sync
                let account_ids: Vec<Uuid> = {
                    let engines = sync_engines.read().await;
                    engines.keys().cloned().collect()
                };
                
                // Sync each account individually to avoid holding mutable references across await
                for account_id in account_ids {
                    let result = {
                        let mut engines = sync_engines.write().await;
                        if let Some(engine) = engines.get_mut(&account_id) {
                            Self::sync_account_engine(&mut **engine, &storage, &search_index).await
                        } else {
                            continue; // Account was removed
                        }
                    };
                    
                    match result {
                        Err(e) => {
                            error!("Failed to sync account {}: {}", account_id, e);
                            
                            // Update stats
                            let mut stats = stats.write().await;
                            stats.failed_syncs += 1;
                        }
                        Ok(_result) => {
                            // Update stats
                            let mut stats = stats.write().await;
                            stats.successful_syncs += 1;
                        }
                    }
                }
            }
        });
        
        self.background_task = Some(task);
        info!("Started background sync with interval: {:?}", sync_interval);
        Ok(())
    }

    /// Stop background sync
    pub async fn stop_background_sync(&mut self) -> AsgardResult<()> {
        if let Some(task) = self.background_task.take() {
            task.abort();
            info!("Stopped background sync");
        }
        Ok(())
    }

    /// Sync a specific account
    pub async fn sync_account(&self, account_id: Uuid) -> AsgardResult<SyncResult> {
        let mut engines = self.sync_engines.write().await;
        let engine = engines.get_mut(&account_id)
            .ok_or_else(|| AsgardError::not_found(format!("Account not found: {}", account_id)))?;
        
        Self::sync_account_engine(&mut **engine, &self.storage, &self.search_index).await
    }

    /// Get sync statistics
    pub async fn get_stats(&self) -> SyncStats {
        self.stats.read().await.clone()
    }

    /// Get sync status for all accounts
    pub async fn get_all_status(&self) -> HashMap<Uuid, SyncStatus> {
        let engines = self.sync_engines.read().await;
        engines.iter()
            .map(|(id, engine)| (*id, engine.status()))
            .collect()
    }

    // Helper methods

    async fn sync_account_engine(
        engine: &mut (dyn SyncEngine + Send),
        storage: &Arc<Mutex<StorageManager>>,
        search_index: &Arc<Mutex<SimpleSearchIndex>>,
    ) -> AsgardResult<SyncResult> {
        let start_time = std::time::Instant::now();
        let mut messages_synced = 0;
        let mut new_messages = 0;
        let mut updated_messages = 0;
        
        // Connect to server
        engine.connect().await?;
        
        // Sync mailboxes
        let mailboxes = engine.sync_mailboxes().await?;
        
        // Store mailboxes in database
        {
            let mut storage = storage.lock().await;
            for mailbox in &mailboxes {
                storage.database_mut().create_mailbox(mailbox).await?;
            }
        }
        
        // Sync messages in each mailbox
        for mailbox in &mailboxes {
            let messages = engine.sync_mailbox_messages(mailbox).await?;
            
            // Store messages in database and search index
            {
                let mut storage = storage.lock().await;
                let mut search_index = search_index.lock().await;
                
                for message in messages {
                    // Check if message already exists
                    let existing_message = storage.database().get_message(message.id).await?;
                    
                    if existing_message.is_some() {
                        storage.database_mut().update_message(&message).await?;
                        search_index.update_message(&message)?;
                        updated_messages += 1;
                    } else {
                        storage.database_mut().create_message(&message).await?;
                        search_index.add_message(&message)?;
                        new_messages += 1;
                    }
                    
                    messages_synced += 1;
                }
            }
        }
        
        // Disconnect from server
        engine.disconnect().await?;
        
        let duration = start_time.elapsed();
        
        let result = SyncResult {
            messages_synced,
            new_messages,
            updated_messages,
            deleted_messages: 0, // TODO: Implement message deletion detection
            duration,
            error: None,
        };
        
        Ok(result)
    }
}

impl Drop for SyncManager {
    fn drop(&mut self) {
        if let Some(task) = self.background_task.take() {
            task.abort();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_sync_manager_creation() {
        let temp_dir = TempDir::new().unwrap();
        let storage = Arc::new(Mutex::new(
            StorageManager::new(
                temp_dir.path().join("test.db"),
                temp_dir.path().join("cache"),
            ).await.unwrap()
        ));
        
        let search_index = Arc::new(Mutex::new(
            SimpleSearchIndex::new(temp_dir.path().join("search")).unwrap()
        ));
        
        let sync_manager = SyncManager::new(
            storage,
            search_index,
            Duration::from_secs(300),
        );
        
        let stats = sync_manager.get_stats().await;
        assert_eq!(stats.total_syncs, 0);
    }
}
