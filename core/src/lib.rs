//! Asgard Mail Core Library
//!
//! This crate contains the core business logic for Asgard Mail, including:
//! - Domain models (Account, Mailbox, Message)
//! - Storage layer (SQLite database and caching)
//! - Sync engines (IMAP, SMTP, POP3)
//! - Search functionality (Tantivy full-text search)
//! - Gmail-specific features (labels, XOAUTH2)

pub mod account;
pub mod error;
pub mod mailbox;
pub mod message;
pub mod storage;
pub mod sync;
pub mod search;
pub mod gmail;
pub mod config;
pub mod crypto;
pub mod threads;
pub mod types;
pub mod threading;

// Re-export commonly used types
pub use account::{Account, AccountType, AccountStatus};
pub use error::{AsgardError, AsgardResult};
pub use mailbox::{Mailbox, MailboxType, MailboxFlags};
pub use message::{Message, MessageFlags, MessagePart, Attachment};
pub use storage::{Database, Cache};
pub use config::Config;
pub use types::{MsgMeta, Thread};
pub use threading::{group_into_threads, normalize_subject};

/// Application version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Application name
pub const APP_NAME: &str = "Asgard Mail";

/// Default configuration directory name
pub const CONFIG_DIR_NAME: &str = "asgard-mail";

/// Default cache directory name
pub const CACHE_DIR_NAME: &str = "asgard-mail";

/// Default database filename
pub const DB_FILENAME: &str = "asgard-mail.db";

/// Default search index directory name
pub const SEARCH_INDEX_DIR: &str = "search-index";

/// Maximum message size (25MB)
pub const MAX_MESSAGE_SIZE: usize = 25 * 1024 * 1024;

/// Maximum attachment size (25MB)
pub const MAX_ATTACHMENT_SIZE: usize = 25 * 1024 * 1024;

/// Default sync interval in seconds (5 minutes)
pub const DEFAULT_SYNC_INTERVAL: u64 = 300;

/// Default cache size in MB (500MB)
pub const DEFAULT_CACHE_SIZE_MB: usize = 500;

/// Gmail IMAP server
pub const GMAIL_IMAP_HOST: &str = "imap.gmail.com";
pub const GMAIL_IMAP_PORT: u16 = 993;

/// Gmail SMTP server
pub const GMAIL_SMTP_HOST: &str = "smtp.gmail.com";
pub const GMAIL_SMTP_PORT: u16 = 587;

/// Gmail OAuth scopes
pub const GMAIL_OAUTH_SCOPES: &[&str] = &[
    "https://www.googleapis.com/auth/gmail.readonly",
    "https://www.googleapis.com/auth/gmail.send",
    "https://www.googleapis.com/auth/gmail.modify",
    "https://www.googleapis.com/auth/gmail.labels",
];

/// Gmail OAuth redirect URI
pub const GMAIL_OAUTH_REDIRECT_URI: &str = "http://127.0.0.1:8080";

/// Initialize the core library
pub fn init() -> AsgardResult<()> {
    tracing::info!("Initializing Asgard Mail Core v{}", VERSION);
    
    // Initialize sodiumoxide for crypto operations
    sodiumoxide::init().map_err(|_| AsgardError::CryptoInitFailed)?;
    
    Ok(())
}

/// Get the default configuration directory
pub fn get_config_dir() -> AsgardResult<std::path::PathBuf> {
    let config_dir = std::env::var("ASGARD_MAIL_CONFIG_DIR")
        .map(std::path::PathBuf::from)
        .or_else(|_| {
            directories::ProjectDirs::from("", "", CONFIG_DIR_NAME)
                .map(|dirs| dirs.config_dir().to_path_buf())
                .ok_or_else(|| std::env::VarError::NotPresent)
        })
        .unwrap_or_else(|_| std::path::PathBuf::from("~/.config/asgard-mail"));
    
    std::fs::create_dir_all(&config_dir)
        .map_err(|_| AsgardError::ConfigDirCreateFailed(config_dir.clone()))?;
    
    Ok(config_dir)
}

/// Get the default cache directory
pub fn get_cache_dir() -> AsgardResult<std::path::PathBuf> {
    let cache_dir = std::env::var("ASGARD_MAIL_CACHE_DIR")
        .map(std::path::PathBuf::from)
        .or_else(|_| {
            directories::ProjectDirs::from("", "", CACHE_DIR_NAME)
                .map(|dirs| dirs.cache_dir().to_path_buf())
                .ok_or_else(|| std::env::VarError::NotPresent)
        })
        .unwrap_or_else(|_| std::path::PathBuf::from("~/.cache/asgard-mail"));
    
    std::fs::create_dir_all(&cache_dir)
        .map_err(|_| AsgardError::CacheDirCreateFailed(cache_dir.clone()))?;
    
    Ok(cache_dir)
}

/// Get the default data directory
pub fn get_data_dir() -> AsgardResult<std::path::PathBuf> {
    let data_dir = std::env::var("ASGARD_MAIL_DATA_DIR")
        .map(std::path::PathBuf::from)
        .or_else(|_| {
            directories::ProjectDirs::from("", "", CACHE_DIR_NAME)
                .map(|dirs| dirs.data_dir().to_path_buf())
                .ok_or_else(|| std::env::VarError::NotPresent)
        })
        .unwrap_or_else(|_| std::path::PathBuf::from("~/.local/share/asgard-mail"));
    
    std::fs::create_dir_all(&data_dir)
        .map_err(|_| AsgardError::DataDirCreateFailed(data_dir.clone()))?;
    
    Ok(data_dir)
}
