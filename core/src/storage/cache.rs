//! Cache layer for Asgard Mail

use crate::error::{AsgardError, AsgardResult};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use time::OffsetDateTime;
use uuid::Uuid;
use sha2::{Sha256, Digest};

/// Cache entry metadata
#[derive(Debug, Clone)]
pub struct CacheEntry {
    /// Cache key
    pub key: String,
    /// File path
    pub file_path: PathBuf,
    /// Content type
    pub content_type: String,
    /// Size in bytes
    pub size: usize,
    /// Creation time
    pub created_at: OffsetDateTime,
    /// Last access time
    pub accessed_at: OffsetDateTime,
    /// Expiration time
    pub expires_at: Option<OffsetDateTime>,
    /// Content hash for deduplication
    pub content_hash: String,
}

/// Cache statistics
#[derive(Debug, Clone, Default)]
pub struct CacheStats {
    /// Total number of entries
    pub total_entries: usize,
    /// Total size in bytes
    pub total_size: usize,
    /// Number of hits
    pub hits: usize,
    /// Number of misses
    pub misses: usize,
    /// Cache hit ratio
    pub hit_ratio: f64,
}

/// File-based cache for storing message content and attachments
pub struct Cache {
    /// Cache directory
    cache_dir: PathBuf,
    /// Maximum cache size in bytes
    max_size: usize,
    /// Cache entries metadata
    entries: Arc<RwLock<HashMap<String, CacheEntry>>>,
    /// Cache statistics
    stats: Arc<RwLock<CacheStats>>,
}

impl Cache {
    /// Create a new cache instance
    pub async fn new(cache_dir: PathBuf) -> AsgardResult<Self> {
        // Ensure cache directory exists
        std::fs::create_dir_all(&cache_dir)?;
        
        // Create subdirectories
        std::fs::create_dir_all(cache_dir.join("messages"))?;
        std::fs::create_dir_all(cache_dir.join("attachments"))?;
        std::fs::create_dir_all(cache_dir.join("temp"))?;
        
        let cache = Self {
            cache_dir,
            max_size: 500 * 1024 * 1024, // 500MB default
            entries: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(CacheStats::default())),
        };
        
        // Load existing cache entries
        cache.load_cache_entries().await?;
        
        Ok(cache)
    }

    /// Initialize the cache
    pub async fn initialize(&mut self) -> AsgardResult<()> {
        // Clean up expired entries
        self.cleanup_expired().await?;
        
        // Ensure cache size is within limits
        self.enforce_size_limit().await?;
        
        Ok(())
    }

    /// Close the cache
    pub async fn close(self) -> AsgardResult<()> {
        // Save cache metadata
        self.save_cache_metadata().await?;
        Ok(())
    }

    /// Set the maximum cache size
    pub fn set_max_size(&mut self, max_size: usize) {
        self.max_size = max_size;
    }

    /// Get cache statistics
    pub async fn get_stats(&self) -> CacheStats {
        self.stats.read().await.clone()
    }

    /// Store data in the cache
    pub async fn store(&self, key: &str, data: &[u8], content_type: &str, ttl: Option<time::Duration>) -> AsgardResult<()> {
        let content_hash = self.calculate_hash(data);
        
        // Check if we already have this content
        if let Some(existing_entry) = self.find_by_hash(&content_hash).await? {
            // Create a hard link to the existing file
            let new_path = self.get_file_path(key);
            std::fs::hard_link(&existing_entry.file_path, &new_path)?;
            
            // Update metadata
            let entry = CacheEntry {
                key: key.to_string(),
                file_path: new_path,
                content_type: content_type.to_string(),
                size: data.len(),
                created_at: OffsetDateTime::now_utc(),
                accessed_at: OffsetDateTime::now_utc(),
                expires_at: ttl.map(|duration| OffsetDateTime::now_utc() + duration),
                content_hash,
            };
            
            self.entries.write().await.insert(key.to_string(), entry);
            return Ok(());
        }
        
        // Store new content
        let file_path = self.get_file_path(key);
        std::fs::write(&file_path, data)?;
        
        let entry = CacheEntry {
            key: key.to_string(),
            file_path,
            content_type: content_type.to_string(),
            size: data.len(),
            created_at: OffsetDateTime::now_utc(),
            accessed_at: OffsetDateTime::now_utc(),
            expires_at: ttl.map(|duration| OffsetDateTime::now_utc() + duration),
            content_hash,
        };
        
        self.entries.write().await.insert(key.to_string(), entry);
        
        // Update statistics
        self.update_stats_hit().await;
        
        // Check if we need to enforce size limit
        self.enforce_size_limit().await?;
        
        Ok(())
    }

    /// Retrieve data from the cache
    pub async fn retrieve(&self, key: &str) -> AsgardResult<Option<Vec<u8>>> {
        let mut entries = self.entries.write().await;
        
        if let Some(entry) = entries.get_mut(key) {
            // Check if entry has expired
            if let Some(expires_at) = entry.expires_at {
                if OffsetDateTime::now_utc() > expires_at {
                    // Remove expired entry
                    self.remove_entry(&entry.file_path).await?;
                    entries.remove(key);
                    self.update_stats_miss().await;
                    return Ok(None);
                }
            }
            
            // Update access time
            entry.accessed_at = OffsetDateTime::now_utc();
            
            // Read file content
            match std::fs::read(&entry.file_path) {
                Ok(data) => {
                    self.update_stats_hit().await;
                    Ok(Some(data))
                }
                Err(_) => {
                    // File doesn't exist, remove entry
                    entries.remove(key);
                    self.update_stats_miss().await;
                    Ok(None)
                }
            }
        } else {
            self.update_stats_miss().await;
            Ok(None)
        }
    }

    /// Remove data from the cache
    pub async fn remove(&self, key: &str) -> AsgardResult<bool> {
        let mut entries = self.entries.write().await;
        
        if let Some(entry) = entries.remove(key) {
            self.remove_entry(&entry.file_path).await?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Check if a key exists in the cache
    pub async fn contains(&self, key: &str) -> bool {
        self.entries.read().await.contains_key(key)
    }

    /// Get cache entry metadata
    pub async fn get_entry(&self, key: &str) -> Option<CacheEntry> {
        self.entries.read().await.get(key).cloned()
    }

    /// Clear all cache entries
    pub async fn clear(&self) -> AsgardResult<()> {
        let mut entries = self.entries.write().await;
        
        for entry in entries.values() {
            self.remove_entry(&entry.file_path).await?;
        }
        
        entries.clear();
        
        // Reset statistics
        *self.stats.write().await = CacheStats::default();
        
        Ok(())
    }

    /// Clean up expired entries
    pub async fn cleanup_expired(&self) -> AsgardResult<()> {
        let now = OffsetDateTime::now_utc();
        let mut entries = self.entries.write().await;
        let mut to_remove = Vec::new();
        
        for (key, entry) in entries.iter() {
            if let Some(expires_at) = entry.expires_at {
                if now > expires_at {
                    to_remove.push(key.clone());
                }
            }
        }
        
        for key in to_remove {
            if let Some(entry) = entries.remove(&key) {
                self.remove_entry(&entry.file_path).await?;
            }
        }
        
        Ok(())
    }

    /// Enforce cache size limit by removing least recently used entries
    pub async fn enforce_size_limit(&self) -> AsgardResult<()> {
        let mut entries = self.entries.write().await;
        let mut total_size: usize = entries.values().map(|e| e.size).sum();
        
        if total_size <= self.max_size {
            return Ok(());
        }
        
        // Sort entries by access time (oldest first)
        let mut sorted_entries: Vec<_> = entries.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
        sorted_entries.sort_by_key(|(_, entry)| entry.accessed_at);
        
        // Remove entries until we're under the limit
        for (key, entry) in sorted_entries {
            if total_size <= self.max_size {
                break;
            }
            
            self.remove_entry(&entry.file_path).await?;
            entries.remove(&key);
            total_size -= entry.size;
        }
        
        Ok(())
    }

    /// Store message content
    pub async fn store_message(&self, message_id: Uuid, content: &[u8], content_type: &str) -> AsgardResult<()> {
        let key = format!("message:{}", message_id);
        self.store(&key, content, content_type, None).await
    }

    /// Retrieve message content
    pub async fn retrieve_message(&self, message_id: Uuid) -> AsgardResult<Option<Vec<u8>>> {
        let key = format!("message:{}", message_id);
        self.retrieve(&key).await
    }

    /// Store attachment content
    pub async fn store_attachment(&self, attachment_id: Uuid, content: &[u8], content_type: &str) -> AsgardResult<()> {
        let key = format!("attachment:{}", attachment_id);
        self.store(&key, content, content_type, None).await
    }

    /// Retrieve attachment content
    pub async fn retrieve_attachment(&self, attachment_id: Uuid) -> AsgardResult<Option<Vec<u8>>> {
        let key = format!("attachment:{}", attachment_id);
        self.retrieve(&key).await
    }

    /// Store temporary data
    pub async fn store_temp(&self, key: &str, data: &[u8], ttl: time::Duration) -> AsgardResult<()> {
        let temp_key = format!("temp:{}", key);
        self.store(&temp_key, data, "application/octet-stream", Some(ttl)).await
    }

    /// Retrieve temporary data
    pub async fn retrieve_temp(&self, key: &str) -> AsgardResult<Option<Vec<u8>>> {
        let temp_key = format!("temp:{}", key);
        self.retrieve(&temp_key).await
    }

    // Helper methods

    fn get_file_path(&self, key: &str) -> PathBuf {
        let hash = self.calculate_hash(key.as_bytes());
        let subdir = &hash[0..2];
        
        if key.starts_with("message:") {
            self.cache_dir.join("messages").join(subdir).join(&hash)
        } else if key.starts_with("attachment:") {
            self.cache_dir.join("attachments").join(subdir).join(&hash)
        } else if key.starts_with("temp:") {
            self.cache_dir.join("temp").join(&hash)
        } else {
            self.cache_dir.join(&hash)
        }
    }

    fn calculate_hash(&self, data: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data);
        hex::encode(hasher.finalize())
    }

    async fn find_by_hash(&self, content_hash: &str) -> AsgardResult<Option<CacheEntry>> {
        let entries = self.entries.read().await;
        
        for entry in entries.values() {
            if entry.content_hash == content_hash {
                return Ok(Some(entry.clone()));
            }
        }
        
        Ok(None)
    }

    async fn remove_entry(&self, file_path: &Path) -> AsgardResult<()> {
        if file_path.exists() {
            std::fs::remove_file(file_path)?;
        }
        Ok(())
    }

    async fn update_stats_hit(&self) {
        let mut stats = self.stats.write().await;
        stats.hits += 1;
        stats.hit_ratio = stats.hits as f64 / (stats.hits + stats.misses) as f64;
    }

    async fn update_stats_miss(&self) {
        let mut stats = self.stats.write().await;
        stats.misses += 1;
        stats.hit_ratio = stats.hits as f64 / (stats.hits + stats.misses) as f64;
    }

    async fn load_cache_entries(&self) -> AsgardResult<()> {
        // This would scan the cache directory and rebuild the entries map
        // For now, we'll start with an empty cache
        Ok(())
    }

    async fn save_cache_metadata(&self) -> AsgardResult<()> {
        // This would save cache metadata to disk for persistence
        // For now, we'll skip this
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_cache_creation() {
        let temp_dir = TempDir::new().unwrap();
        let cache_dir = temp_dir.path().to_path_buf();
        let cache = Cache::new(cache_dir).await.unwrap();
        cache.initialize().await.unwrap();
    }

    #[tokio::test]
    async fn test_cache_store_retrieve() {
        let temp_dir = TempDir::new().unwrap();
        let cache_dir = temp_dir.path().to_path_buf();
        let cache = Cache::new(cache_dir).await.unwrap();
        cache.initialize().await.unwrap();

        let data = b"Hello, World!";
        let key = "test_key";
        
        // Store data
        cache.store(key, data, "text/plain", None).await.unwrap();
        
        // Retrieve data
        let retrieved = cache.retrieve(key).await.unwrap().unwrap();
        assert_eq!(data, retrieved.as_slice());
        
        // Check that key exists
        assert!(cache.contains(key).await);
    }

    #[tokio::test]
    async fn test_cache_expiration() {
        let temp_dir = TempDir::new().unwrap();
        let cache_dir = temp_dir.path().to_path_buf();
        let cache = Cache::new(cache_dir).await.unwrap();
        cache.initialize().await.unwrap();

        let data = b"Hello, World!";
        let key = "test_key";
        let ttl = time::Duration::seconds(1);
        
        // Store data with TTL
        cache.store(key, data, "text/plain", Some(ttl)).await.unwrap();
        
        // Should be available immediately
        assert!(cache.retrieve(key).await.unwrap().is_some());
        
        // Wait for expiration
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        
        // Should be expired
        assert!(cache.retrieve(key).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_cache_deduplication() {
        let temp_dir = TempDir::new().unwrap();
        let cache_dir = temp_dir.path().to_path_buf();
        let cache = Cache::new(cache_dir).await.unwrap();
        cache.initialize().await.unwrap();

        let data = b"Hello, World!";
        
        // Store same data with different keys
        cache.store("key1", data, "text/plain", None).await.unwrap();
        cache.store("key2", data, "text/plain", None).await.unwrap();
        
        // Both should be retrievable
        assert!(cache.retrieve("key1").await.unwrap().is_some());
        assert!(cache.retrieve("key2").await.unwrap().is_some());
        
        // But should share the same file (deduplication)
        let entry1 = cache.get_entry("key1").await.unwrap();
        let entry2 = cache.get_entry("key2").await.unwrap();
        assert_eq!(entry1.content_hash, entry2.content_hash);
    }

    #[tokio::test]
    async fn test_message_cache() {
        let temp_dir = TempDir::new().unwrap();
        let cache_dir = temp_dir.path().to_path_buf();
        let cache = Cache::new(cache_dir).await.unwrap();
        cache.initialize().await.unwrap();

        let message_id = Uuid::new_v4();
        let content = b"Message content";
        
        // Store message
        cache.store_message(message_id, content, "text/plain").await.unwrap();
        
        // Retrieve message
        let retrieved = cache.retrieve_message(message_id).await.unwrap().unwrap();
        assert_eq!(content, retrieved.as_slice());
    }

    #[tokio::test]
    async fn test_attachment_cache() {
        let temp_dir = TempDir::new().unwrap();
        let cache_dir = temp_dir.path().to_path_buf();
        let cache = Cache::new(cache_dir).await.unwrap();
        cache.initialize().await.unwrap();

        let attachment_id = Uuid::new_v4();
        let content = b"Attachment content";
        
        // Store attachment
        cache.store_attachment(attachment_id, content, "application/pdf").await.unwrap();
        
        // Retrieve attachment
        let retrieved = cache.retrieve_attachment(attachment_id).await.unwrap().unwrap();
        assert_eq!(content, retrieved.as_slice());
    }

    #[tokio::test]
    async fn test_cache_stats() {
        let temp_dir = TempDir::new().unwrap();
        let cache_dir = temp_dir.path().to_path_buf();
        let cache = Cache::new(cache_dir).await.unwrap();
        cache.initialize().await.unwrap();

        let data = b"Hello, World!";
        let key = "test_key";
        
        // Store and retrieve data
        cache.store(key, data, "text/plain", None).await.unwrap();
        cache.retrieve(key).await.unwrap();
        cache.retrieve("nonexistent").await.unwrap();
        
        let stats = cache.get_stats().await;
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 1);
        assert_eq!(stats.hit_ratio, 0.5);
    }
}
