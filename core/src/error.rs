//! Error types for Asgard Mail Core

use std::fmt;
use std::path::PathBuf;

/// Result type alias for Asgard Mail operations
pub type AsgardResult<T> = Result<T, AsgardError>;

/// Main error type for Asgard Mail
#[derive(Debug, thiserror::Error)]
pub enum AsgardError {
    /// Configuration errors
    #[error("Configuration error: {0}")]
    Config(String),
    
    /// Configuration directory not found
    #[error("Configuration directory not found")]
    ConfigDirNotFound,
    
    /// Failed to create configuration directory
    #[error("Failed to create configuration directory: {0}")]
    ConfigDirCreateFailed(PathBuf),
    
    /// Cache directory not found
    #[error("Cache directory not found")]
    CacheDirNotFound,
    
    /// Failed to create cache directory
    #[error("Failed to create cache directory: {0}")]
    CacheDirCreateFailed(PathBuf),
    
    /// Data directory not found
    #[error("Data directory not found")]
    DataDirNotFound,
    
    /// Failed to create data directory
    #[error("Failed to create data directory: {0}")]
    DataDirCreateFailed(PathBuf),
    
    /// Database errors
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),
    
    /// Database migration error
    #[error("Database migration error: {0}")]
    DatabaseMigration(String),
    
    /// IO errors
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    /// Serialization errors
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    /// TOML parsing errors
    #[error("TOML parsing error: {0}")]
    Toml(#[from] toml::de::Error),
    
    /// TOML serialization errors
    #[error("TOML serialization error: {0}")]
    TomlSer(#[from] toml::ser::Error),
    
    /// IMAP errors
    #[error("IMAP error: {0}")]
    Imap(#[from] async_imap::error::Error),
    
    /// SMTP errors
    #[error("SMTP error: {0}")]
    Smtp(#[from] lettre::error::Error),
    
    /// SMTP transport errors
    #[error("SMTP transport error: {0}")]
    SmtpTransport(#[from] lettre::transport::smtp::Error),
    
    /// Address parsing errors
    #[error("Address parsing error: {0}")]
    AddressParsing(#[from] lettre::address::AddressError),
    
    /// OAuth errors  
    #[error("OAuth error: {0}")]
    OAuth(String),
    
    /// OAuth token error
    #[error("OAuth token error: {0}")]
    OAuthToken(String),
    
    /// HTTP errors
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    
    /// URL parsing errors
    #[error("URL parsing error: {0}")]
    Url(#[from] url::ParseError),
    
    /// Email address parsing errors
    #[error("Email address parsing error: {0}")]
    EmailAddress(#[from] addr::Error),
    
    /// MIME parsing errors
    #[error("MIME parsing error: {0}")]
    Mime(#[from] mailparse::MailParseError),
    
    /// TLS errors
    #[error("TLS error: {0}")]
    Tls(String),
    
    /// Authentication errors
    #[error("Authentication failed: {0}")]
    Authentication(String),
    
    /// Authorization errors
    #[error("Authorization failed: {0}")]
    Authorization(String),
    
    /// Network errors
    #[error("Network error: {0}")]
    Network(String),
    
    /// Timeout errors
    #[error("Operation timed out: {0}")]
    Timeout(String),
    
    /// Search index errors
    #[error("Search index error: {0}")]
    SearchIndex(String),
    
    /// Keyring errors
    #[error("Keyring error: {0}")]
    Keyring(#[from] keyring::Error),
    
    /// Crypto errors
    #[error("Crypto error: {0}")]
    Crypto(String),
    
    /// Crypto initialization failed
    #[error("Crypto initialization failed")]
    CryptoInitFailed,
    
    /// Encryption errors
    #[error("Encryption error: {0}")]
    Encryption(String),
    
    /// Decryption errors
    #[error("Decryption error: {0}")]
    Decryption(String),
    
    /// Account errors
    #[error("Account error: {0}")]
    Account(String),
    
    /// Mailbox errors
    #[error("Mailbox error: {0}")]
    Mailbox(String),
    
    /// Message errors
    #[error("Message error: {0}")]
    Message(String),
    
    /// Sync errors
    #[error("Sync error: {0}")]
    Sync(String),
    
    /// Cache errors
    #[error("Cache error: {0}")]
    Cache(String),
    
    /// Validation errors
    #[error("Validation error: {0}")]
    Validation(String),
    
    /// Not found errors
    #[error("Not found: {0}")]
    NotFound(String),
    
    /// Already exists errors
    #[error("Already exists: {0}")]
    AlreadyExists(String),
    
    /// Invalid state errors
    #[error("Invalid state: {0}")]
    InvalidState(String),
    
    /// Unsupported operation errors
    #[error("Unsupported operation: {0}")]
    Unsupported(String),
    
    /// Generic errors
    #[error("Error: {0}")]
    Generic(String),
}

impl AsgardError {
    /// Create a new configuration error
    pub fn config(msg: impl Into<String>) -> Self {
        Self::Config(msg.into())
    }
    
    /// Create a new authentication error
    pub fn auth(msg: impl Into<String>) -> Self {
        Self::Authentication(msg.into())
    }
    
    /// Create a new network error
    pub fn network(msg: impl Into<String>) -> Self {
        Self::Network(msg.into())
    }
    
    /// Create a new timeout error
    pub fn timeout(msg: impl Into<String>) -> Self {
        Self::Timeout(msg.into())
    }
    
    /// Create a new validation error
    pub fn validation(msg: impl Into<String>) -> Self {
        Self::Validation(msg.into())
    }
    
    /// Create a new not found error
    pub fn not_found(msg: impl Into<String>) -> Self {
        Self::NotFound(msg.into())
    }
    
    /// Create a new already exists error
    pub fn already_exists(msg: impl Into<String>) -> Self {
        Self::AlreadyExists(msg.into())
    }
    
    /// Create a new invalid state error
    pub fn invalid_state(msg: impl Into<String>) -> Self {
        Self::InvalidState(msg.into())
    }
    
    /// Create a new unsupported operation error
    pub fn unsupported(msg: impl Into<String>) -> Self {
        Self::Unsupported(msg.into())
    }
    
    /// Create a new generic error
    pub fn generic(msg: impl Into<String>) -> Self {
        Self::Generic(msg.into())
    }
    
    /// Create a new crypto error
    pub fn crypto(msg: impl Into<String>) -> Self {
        Self::Crypto(msg.into())
    }
    
    /// Create a new account error
    pub fn account(msg: impl Into<String>) -> Self {
        Self::Account(msg.into())
    }
    
    /// Check if this is a network-related error
    pub fn is_network_error(&self) -> bool {
        matches!(self, 
            Self::Network(_) | 
            Self::Timeout(_) | 
            Self::Http(_) | 
            Self::Tls(_)
        )
    }
    
    /// Check if this is an authentication error
    pub fn is_auth_error(&self) -> bool {
        matches!(self, 
            Self::Authentication(_) | 
            Self::Authorization(_) | 
            Self::OAuth(_) | 
            Self::OAuthToken(_)
        )
    }
    
    /// Check if this is a recoverable error
    pub fn is_recoverable(&self) -> bool {
        matches!(self, 
            Self::Network(_) | 
            Self::Timeout(_) | 
            Self::Http(_) | 
            Self::Tls(_) |
            Self::Io(_)
        )
    }
}

