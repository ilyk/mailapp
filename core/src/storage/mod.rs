//! Storage layer for Asgard Mail

pub mod database;
pub mod cache;
pub mod migrations;

pub use database::Database;
pub use cache::Cache;
pub use migrations::MigrationManager;

/// Storage manager that coordinates database and cache operations
pub struct StorageManager {
    database: Database,
    cache: Cache,
}

impl StorageManager {
    /// Create a new storage manager
    pub async fn new(database_path: std::path::PathBuf, cache_dir: std::path::PathBuf) -> crate::error::AsgardResult<Self> {
        let database = Database::new(database_path).await?;
        let cache = Cache::new(cache_dir).await?;
        
        Ok(Self { database, cache })
    }

    /// Get the database instance
    pub fn database(&self) -> &Database {
        &self.database
    }

    /// Get the cache instance
    pub fn cache(&self) -> &Cache {
        &self.cache
    }

    /// Get mutable database instance
    pub fn database_mut(&mut self) -> &mut Database {
        &mut self.database
    }

    /// Get mutable cache instance
    pub fn cache_mut(&mut self) -> &mut Cache {
        &mut self.cache
    }

    /// Initialize storage (run migrations, etc.)
    pub async fn initialize(&mut self) -> crate::error::AsgardResult<()> {
        self.database.initialize().await?;
        self.cache.initialize().await?;
        Ok(())
    }

    /// Close storage connections
    pub async fn close(self) -> crate::error::AsgardResult<()> {
        self.database.close().await?;
        self.cache.close().await?;
        Ok(())
    }
}
