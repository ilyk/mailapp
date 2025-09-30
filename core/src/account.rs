//! Account management for Asgard Mail

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::error::{AsgardError, AsgardResult};

/// Account types supported by Asgard Mail
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AccountType {
    /// Gmail account with OAuth2
    Gmail,
    /// Generic IMAP/SMTP account
    ImapSmtp,
    /// POP3 account
    Pop3,
}

impl std::fmt::Display for AccountType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AccountType::Gmail => write!(f, "Gmail"),
            AccountType::ImapSmtp => write!(f, "IMAP/SMTP"),
            AccountType::Pop3 => write!(f, "POP3"),
        }
    }
}

/// Account status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AccountStatus {
    /// Account is active and syncing
    Active,
    /// Account is paused (not syncing)
    Paused,
    /// Account has authentication issues
    AuthError,
    /// Account has connection issues
    ConnectionError,
    /// Account is disabled
    Disabled,
}

impl std::fmt::Display for AccountStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AccountStatus::Active => write!(f, "Active"),
            AccountStatus::Paused => write!(f, "Paused"),
            AccountStatus::AuthError => write!(f, "Authentication Error"),
            AccountStatus::ConnectionError => write!(f, "Connection Error"),
            AccountStatus::Disabled => write!(f, "Disabled"),
        }
    }
}

/// IMAP/SMTP server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Server hostname
    pub host: String,
    /// Server port
    pub port: u16,
    /// Use TLS/SSL
    pub use_tls: bool,
    /// Use STARTTLS
    pub use_starttls: bool,
    /// Authentication method
    pub auth_method: AuthMethod,
}

/// Authentication methods
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AuthMethod {
    /// Username and password
    Password,
    /// OAuth2 with XOAUTH2
    OAuth2,
    /// App-specific password (for Gmail)
    AppPassword,
}

/// Gmail OAuth2 configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GmailOAuthConfig {
    /// OAuth2 client ID
    pub client_id: String,
    /// OAuth2 client secret
    pub client_secret: String,
    /// OAuth2 access token (encrypted)
    pub access_token: Option<String>,
    /// OAuth2 refresh token (encrypted)
    pub refresh_token: Option<String>,
    /// Token expiration time
    pub token_expires_at: Option<OffsetDateTime>,
    /// OAuth2 scopes
    pub scopes: Vec<String>,
}

/// Account configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountConfig {
    /// Account type
    pub account_type: AccountType,
    /// Display name for the account
    pub display_name: String,
    /// Email address
    pub email: String,
    /// IMAP server configuration
    pub imap: Option<ServerConfig>,
    /// SMTP server configuration
    pub smtp: Option<ServerConfig>,
    /// POP3 server configuration
    pub pop3: Option<ServerConfig>,
    /// Gmail OAuth2 configuration
    pub gmail_oauth: Option<GmailOAuthConfig>,
    /// Sync settings
    pub sync_settings: SyncSettings,
    /// Account-specific settings
    pub settings: HashMap<String, serde_json::Value>,
}

/// Sync settings for an account
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncSettings {
    /// Sync interval in seconds
    pub sync_interval: u64,
    /// Enable IDLE for IMAP
    pub enable_idle: bool,
    /// Maximum number of messages to sync
    pub max_messages: Option<usize>,
    /// Sync only recent messages (days)
    pub sync_recent_days: Option<u32>,
    /// Delete messages from server after sync (POP3)
    pub delete_after_sync: bool,
    /// Sync folders/labels
    pub sync_folders: bool,
    /// Sync attachments
    pub sync_attachments: bool,
    /// Maximum attachment size in bytes
    pub max_attachment_size: usize,
}

impl Default for SyncSettings {
    fn default() -> Self {
        Self {
            sync_interval: 300, // 5 minutes
            enable_idle: true,
            max_messages: None,
            sync_recent_days: Some(30),
            delete_after_sync: false,
            sync_folders: true,
            sync_attachments: true,
            max_attachment_size: 25 * 1024 * 1024, // 25MB
        }
    }
}

/// Account represents an email account in Asgard Mail
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    /// Unique account ID
    pub id: Uuid,
    /// Account configuration
    pub config: AccountConfig,
    /// Account status
    pub status: AccountStatus,
    /// Last sync time
    pub last_sync: Option<OffsetDateTime>,
    /// Last error message
    pub last_error: Option<String>,
    /// Account creation time
    pub created_at: OffsetDateTime,
    /// Last modification time
    pub updated_at: OffsetDateTime,
    /// Account statistics
    pub stats: AccountStats,
}

/// Account statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AccountStats {
    /// Total number of messages
    pub total_messages: u64,
    /// Number of unread messages
    pub unread_messages: u64,
    /// Number of folders/labels
    pub total_folders: u32,
    /// Last message received time
    pub last_message_received: Option<OffsetDateTime>,
    /// Total storage used in bytes
    pub storage_used: u64,
    /// Number of sync operations
    pub sync_count: u64,
    /// Number of failed syncs
    pub failed_sync_count: u64,
}

impl Account {
    /// Create a new Gmail account
    pub fn new_gmail(
        email: String,
        display_name: Option<String>,
        oauth_config: GmailOAuthConfig,
    ) -> AsgardResult<Self> {
        // Basic email validation - replace with proper validation later
        if !email.contains('@') {
            return Err(AsgardError::validation("Invalid email address"));
        }

        Ok(Self {
            id: Uuid::new_v4(),
            config: AccountConfig {
                account_type: AccountType::Gmail,
                display_name: display_name.unwrap_or_else(|| email.to_string()),
                email: email.to_string(),
                imap: Some(ServerConfig {
                    host: "imap.gmail.com".to_string(),
                    port: 993,
                    use_tls: true,
                    use_starttls: false,
                    auth_method: AuthMethod::OAuth2,
                }),
                smtp: Some(ServerConfig {
                    host: "smtp.gmail.com".to_string(),
                    port: 587,
                    use_tls: false,
                    use_starttls: true,
                    auth_method: AuthMethod::OAuth2,
                }),
                pop3: None,
                gmail_oauth: Some(oauth_config),
                sync_settings: SyncSettings::default(),
                settings: HashMap::new(),
            },
            status: AccountStatus::Active,
            last_sync: None,
            last_error: None,
            created_at: OffsetDateTime::now_utc(),
            updated_at: OffsetDateTime::now_utc(),
            stats: AccountStats::default(),
        })
    }

    /// Create a new IMAP/SMTP account
    pub fn new_imap_smtp(
        email: String,
        display_name: Option<String>,
        imap_config: ServerConfig,
        smtp_config: ServerConfig,
    ) -> AsgardResult<Self> {
        // Basic email validation - replace with proper validation later
        if !email.contains('@') {
            return Err(AsgardError::validation("Invalid email address"));
        }

        Ok(Self {
            id: Uuid::new_v4(),
            config: AccountConfig {
                account_type: AccountType::ImapSmtp,
                display_name: display_name.unwrap_or_else(|| email.to_string()),
                email: email.to_string(),
                imap: Some(imap_config),
                smtp: Some(smtp_config),
                pop3: None,
                gmail_oauth: None,
                sync_settings: SyncSettings::default(),
                settings: HashMap::new(),
            },
            status: AccountStatus::Active,
            last_sync: None,
            last_error: None,
            created_at: OffsetDateTime::now_utc(),
            updated_at: OffsetDateTime::now_utc(),
            stats: AccountStats::default(),
        })
    }

    /// Create a new POP3 account
    pub fn new_pop3(
        email: String,
        display_name: Option<String>,
        pop3_config: ServerConfig,
    ) -> AsgardResult<Self> {
        // Basic email validation - replace with proper validation later
        if !email.contains('@') {
            return Err(AsgardError::validation("Invalid email address"));
        }

        Ok(Self {
            id: Uuid::new_v4(),
            config: AccountConfig {
                account_type: AccountType::Pop3,
                display_name: display_name.unwrap_or_else(|| email.to_string()),
                email: email.to_string(),
                imap: None,
                smtp: None,
                pop3: Some(pop3_config),
                gmail_oauth: None,
                sync_settings: SyncSettings::default(),
                settings: HashMap::new(),
            },
            status: AccountStatus::Active,
            last_sync: None,
            last_error: None,
            created_at: OffsetDateTime::now_utc(),
            updated_at: OffsetDateTime::now_utc(),
            stats: AccountStats::default(),
        })
    }

    /// Get the email address
    pub fn email(&self) -> &str {
        &self.config.email
    }

    /// Get the display name
    pub fn display_name(&self) -> &str {
        &self.config.display_name
    }

    /// Get the account type
    pub fn account_type(&self) -> AccountType {
        self.config.account_type
    }

    /// Check if the account is active
    pub fn is_active(&self) -> bool {
        self.status == AccountStatus::Active
    }

    /// Check if the account has errors
    pub fn has_errors(&self) -> bool {
        matches!(
            self.status,
            AccountStatus::AuthError | AccountStatus::ConnectionError
        )
    }

    /// Update the account status
    pub fn update_status(&mut self, status: AccountStatus) {
        self.status = status;
        self.updated_at = OffsetDateTime::now_utc();
    }

    /// Update the last sync time
    pub fn update_last_sync(&mut self) {
        self.last_sync = Some(OffsetDateTime::now_utc());
        self.updated_at = OffsetDateTime::now_utc();
    }

    /// Set the last error
    pub fn set_last_error(&mut self, error: Option<String>) {
        self.last_error = error;
        self.updated_at = OffsetDateTime::now_utc();
    }

    /// Update account statistics
    pub fn update_stats(&mut self, stats: AccountStats) {
        self.stats = stats;
        self.updated_at = OffsetDateTime::now_utc();
    }

    /// Get the IMAP server configuration
    pub fn imap_config(&self) -> Option<&ServerConfig> {
        self.config.imap.as_ref()
    }

    /// Get the SMTP server configuration
    pub fn smtp_config(&self) -> Option<&ServerConfig> {
        self.config.smtp.as_ref()
    }

    /// Get the POP3 server configuration
    pub fn pop3_config(&self) -> Option<&ServerConfig> {
        self.config.pop3.as_ref()
    }

    /// Get the Gmail OAuth configuration
    pub fn gmail_oauth_config(&self) -> Option<&GmailOAuthConfig> {
        self.config.gmail_oauth.as_ref()
    }

    /// Check if this is a Gmail account
    pub fn is_gmail(&self) -> bool {
        self.config.account_type == AccountType::Gmail
    }

    /// Check if this is an IMAP/SMTP account
    pub fn is_imap_smtp(&self) -> bool {
        self.config.account_type == AccountType::ImapSmtp
    }

    /// Check if this is a POP3 account
    pub fn is_pop3(&self) -> bool {
        self.config.account_type == AccountType::Pop3
    }

    /// Validate the account configuration
    pub fn validate(&self) -> AsgardResult<()> {
        // Validate email address
        // Basic email validation - replace with proper validation later
        if !self.config.email.contains('@') {
            return Err(AsgardError::validation("Invalid email address"));
        }
        // Email validation passed

        // Validate account type specific configuration
        match self.config.account_type {
            AccountType::Gmail => {
                if self.config.gmail_oauth.is_none() {
                    return Err(AsgardError::validation("Gmail account requires OAuth configuration"));
                }
                if self.config.imap.is_none() || self.config.smtp.is_none() {
                    return Err(AsgardError::validation("Gmail account requires IMAP and SMTP configuration"));
                }
            }
            AccountType::ImapSmtp => {
                if self.config.imap.is_none() || self.config.smtp.is_none() {
                    return Err(AsgardError::validation("IMAP/SMTP account requires both IMAP and SMTP configuration"));
                }
            }
            AccountType::Pop3 => {
                if self.config.pop3.is_none() {
                    return Err(AsgardError::validation("POP3 account requires POP3 configuration"));
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gmail_account_creation() {
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

        assert_eq!(account.email(), "test@gmail.com");
        assert_eq!(account.display_name(), "Test Account");
        assert_eq!(account.account_type(), AccountType::Gmail);
        assert!(account.is_gmail());
        assert!(account.is_active());
    }

    #[test]
    fn test_imap_smtp_account_creation() {
        let imap_config = ServerConfig {
            host: "imap.example.com".to_string(),
            port: 993,
            use_tls: true,
            use_starttls: false,
            auth_method: AuthMethod::Password,
        };

        let smtp_config = ServerConfig {
            host: "smtp.example.com".to_string(),
            port: 587,
            use_tls: false,
            use_starttls: true,
            auth_method: AuthMethod::Password,
        };

        let account = Account::new_imap_smtp(
            "test@example.com".to_string(),
            None,
            imap_config,
            smtp_config,
        ).unwrap();

        assert_eq!(account.email(), "test@example.com");
        assert_eq!(account.display_name(), "test@example.com");
        assert_eq!(account.account_type(), AccountType::ImapSmtp);
        assert!(account.is_imap_smtp());
        assert!(account.is_active());
    }

    #[test]
    fn test_pop3_account_creation() {
        let pop3_config = ServerConfig {
            host: "pop3.example.com".to_string(),
            port: 995,
            use_tls: true,
            use_starttls: false,
            auth_method: AuthMethod::Password,
        };

        let account = Account::new_pop3(
            "test@example.com".to_string(),
            Some("POP3 Account".to_string()),
            pop3_config,
        ).unwrap();

        assert_eq!(account.email(), "test@example.com");
        assert_eq!(account.display_name(), "POP3 Account");
        assert_eq!(account.account_type(), AccountType::Pop3);
        assert!(account.is_pop3());
        assert!(account.is_active());
    }

    #[test]
    fn test_invalid_email() {
        let oauth_config = GmailOAuthConfig {
            client_id: "test-client-id".to_string(),
            client_secret: "test-client-secret".to_string(),
            access_token: None,
            refresh_token: None,
            token_expires_at: None,
            scopes: vec!["https://www.googleapis.com/auth/gmail.readonly".to_string()],
        };

        let result = Account::new_gmail(
            "invalid-email".to_string(),
            None,
            oauth_config,
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_account_validation() {
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
            None,
            oauth_config,
        ).unwrap();

        assert!(account.validate().is_ok());
    }
}
