//! Configuration management for Asgard Mail

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

use crate::error::{AsgardError, AsgardResult};

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Application settings
    pub app: AppConfig,
    /// UI settings
    pub ui: UiConfig,
    /// Sync settings
    pub sync: SyncConfig,
    /// Security settings
    pub security: SecurityConfig,
    /// Notification settings
    pub notifications: NotificationConfig,
    /// Search settings
    pub search: SearchConfig,
    /// Cache settings
    pub cache: CacheConfig,
}

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// Application name
    pub name: String,
    /// Application version
    pub version: String,
    /// Debug mode
    pub debug: bool,
    /// Log level
    pub log_level: String,
    /// Configuration directory
    pub config_dir: PathBuf,
    /// Cache directory
    pub cache_dir: PathBuf,
    /// Data directory
    pub data_dir: PathBuf,
}

/// UI configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    /// Theme preference
    pub theme: ThemePreference,
    /// Window geometry
    pub window_geometry: WindowGeometry,
    /// Pane sizes
    pub pane_sizes: PaneSizes,
    /// Show system tray
    pub show_tray: bool,
    /// Minimize to tray
    pub minimize_to_tray: bool,
    /// Auto-mark as read delay (seconds)
    pub auto_mark_read_delay: u32,
    /// Show preview pane
    pub show_preview_pane: bool,
    /// Message list density
    pub message_list_density: MessageListDensity,
}

/// Theme preference
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ThemePreference {
    /// Light theme
    Light,
    /// Dark theme
    Dark,
    /// System theme
    System,
}

impl std::fmt::Display for ThemePreference {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ThemePreference::Light => write!(f, "light"),
            ThemePreference::Dark => write!(f, "dark"),
            ThemePreference::System => write!(f, "system"),
        }
    }
}

/// Window geometry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowGeometry {
    /// Window width
    pub width: i32,
    /// Window height
    pub height: i32,
    /// Window X position
    pub x: i32,
    /// Window Y position
    pub y: i32,
    /// Window maximized
    pub maximized: bool,
}

/// Pane sizes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaneSizes {
    /// Left pane width (mailbox list)
    pub left_pane_width: i32,
    /// Right pane width (message view)
    pub right_pane_width: i32,
}

/// Message list density
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageListDensity {
    /// Compact density
    Compact,
    /// Normal density
    Normal,
    /// Comfortable density
    Comfortable,
}

impl std::fmt::Display for MessageListDensity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MessageListDensity::Compact => write!(f, "compact"),
            MessageListDensity::Normal => write!(f, "normal"),
            MessageListDensity::Comfortable => write!(f, "comfortable"),
        }
    }
}

/// Sync configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConfig {
    /// Default sync interval (seconds)
    pub default_sync_interval: u64,
    /// Enable IDLE for IMAP
    pub enable_idle: bool,
    /// Maximum concurrent syncs
    pub max_concurrent_syncs: usize,
    /// Sync timeout (seconds)
    pub sync_timeout: u64,
    /// Retry attempts
    pub retry_attempts: u32,
    /// Retry delay (seconds)
    pub retry_delay: u64,
    /// Enable background sync
    pub enable_background_sync: bool,
}

/// Security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Block remote images by default
    pub block_remote_images: bool,
    /// Strip tracking pixels
    pub strip_tracking_pixels: bool,
    /// Encrypt cached content
    pub encrypt_cache: bool,
    /// Allow unsafe HTML
    pub allow_unsafe_html: bool,
    /// Sanitize HTML content
    pub sanitize_html: bool,
    /// Trusted domains for images
    pub trusted_domains: Vec<String>,
    /// Blocked domains
    pub blocked_domains: Vec<String>,
}

/// Notification configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationConfig {
    /// Enable desktop notifications
    pub enable_notifications: bool,
    /// Show notification for new messages
    pub show_new_message_notifications: bool,
    /// Show notification for sync errors
    pub show_sync_error_notifications: bool,
    /// Notification timeout (seconds)
    pub notification_timeout: u32,
    /// Play notification sound
    pub play_sound: bool,
    /// Notification sound file
    pub sound_file: Option<PathBuf>,
}

/// Search configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchConfig {
    /// Enable full-text search
    pub enable_full_text_search: bool,
    /// Search index directory
    pub index_dir: PathBuf,
    /// Maximum search results
    pub max_search_results: usize,
    /// Search timeout (seconds)
    pub search_timeout: u64,
    /// Index update interval (seconds)
    pub index_update_interval: u64,
    /// Enable search suggestions
    pub enable_suggestions: bool,
}

/// Cache configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// Maximum cache size (MB)
    pub max_cache_size_mb: usize,
    /// Cache cleanup interval (seconds)
    pub cleanup_interval: u64,
    /// Enable cache compression
    pub enable_compression: bool,
    /// Cache retention days
    pub retention_days: u32,
    /// Enable cache encryption
    pub enable_encryption: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            app: AppConfig::default(),
            ui: UiConfig::default(),
            sync: SyncConfig::default(),
            security: SecurityConfig::default(),
            notifications: NotificationConfig::default(),
            search: SearchConfig::default(),
            cache: CacheConfig::default(),
        }
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            name: "Asgard Mail".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            debug: false,
            log_level: "info".to_string(),
            config_dir: crate::get_config_dir().unwrap_or_else(|_| PathBuf::from("~/.config/asgard-mail")),
            cache_dir: crate::get_cache_dir().unwrap_or_else(|_| PathBuf::from("~/.cache/asgard-mail")),
            data_dir: crate::get_data_dir().unwrap_or_else(|_| PathBuf::from("~/.local/share/asgard-mail")),
        }
    }
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            theme: ThemePreference::System,
            window_geometry: WindowGeometry {
                width: 1200,
                height: 800,
                x: 100,
                y: 100,
                maximized: false,
            },
            pane_sizes: PaneSizes {
                left_pane_width: 250,
                right_pane_width: 400,
            },
            show_tray: true,
            minimize_to_tray: true,
            auto_mark_read_delay: 3,
            show_preview_pane: true,
            message_list_density: MessageListDensity::Normal,
        }
    }
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            default_sync_interval: 300, // 5 minutes
            enable_idle: true,
            max_concurrent_syncs: 3,
            sync_timeout: 30,
            retry_attempts: 3,
            retry_delay: 5,
            enable_background_sync: true,
        }
    }
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            block_remote_images: true,
            strip_tracking_pixels: true,
            encrypt_cache: false,
            allow_unsafe_html: false,
            sanitize_html: true,
            trusted_domains: vec![],
            blocked_domains: vec![],
        }
    }
}

impl Default for NotificationConfig {
    fn default() -> Self {
        Self {
            enable_notifications: true,
            show_new_message_notifications: true,
            show_sync_error_notifications: true,
            notification_timeout: 5,
            play_sound: false,
            sound_file: None,
        }
    }
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self {
            enable_full_text_search: true,
            index_dir: crate::get_data_dir()
                .unwrap_or_else(|_| PathBuf::from("~/.local/share/asgard-mail"))
                .join("search-index"),
            max_search_results: 1000,
            search_timeout: 10,
            index_update_interval: 60,
            enable_suggestions: true,
        }
    }
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_cache_size_mb: 500,
            cleanup_interval: 3600, // 1 hour
            enable_compression: true,
            retention_days: 30,
            enable_encryption: false,
        }
    }
}

impl Config {
    /// Load configuration from file
    pub fn load(config_path: &PathBuf) -> AsgardResult<Self> {
        if config_path.exists() {
            let content = std::fs::read_to_string(config_path)?;
            let config: Config = toml::from_str(&content)?;
            Ok(config)
        } else {
            // Return default configuration
            Ok(Config::default())
        }
    }

    /// Save configuration to file
    pub fn save(&self, config_path: &PathBuf) -> AsgardResult<()> {
        // Ensure directory exists
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let content = toml::to_string_pretty(self)?;
        std::fs::write(config_path, content)?;
        Ok(())
    }

    /// Load configuration from environment variables
    pub fn load_from_env() -> Self {
        let mut config = Config::default();

        // Load from environment variables
        if let Ok(debug) = std::env::var("ASGARD_MAIL_DEBUG") {
            config.app.debug = debug == "1" || debug.to_lowercase() == "true";
        }

        if let Ok(log_level) = std::env::var("ASGARD_MAIL_LOG_LEVEL") {
            config.app.log_level = log_level;
        }

        if let Ok(config_dir) = std::env::var("ASGARD_MAIL_CONFIG_DIR") {
            config.app.config_dir = PathBuf::from(config_dir);
        }

        if let Ok(cache_dir) = std::env::var("ASGARD_MAIL_CACHE_DIR") {
            config.app.cache_dir = PathBuf::from(cache_dir);
        }

        if let Ok(data_dir) = std::env::var("ASGARD_MAIL_DATA_DIR") {
            config.app.data_dir = PathBuf::from(data_dir);
        }

        if let Ok(theme) = std::env::var("ASGARD_MAIL_THEME") {
            config.ui.theme = match theme.to_lowercase().as_str() {
                "light" => ThemePreference::Light,
                "dark" => ThemePreference::Dark,
                "system" => ThemePreference::System,
                _ => ThemePreference::System,
            };
        }

        if let Ok(sync_interval) = std::env::var("ASGARD_MAIL_SYNC_INTERVAL_SECONDS") {
            if let Ok(interval) = sync_interval.parse() {
                config.sync.default_sync_interval = interval;
            }
        }

        if let Ok(idle_enabled) = std::env::var("ASGARD_MAIL_IDLE_ENABLED") {
            config.sync.enable_idle = idle_enabled == "1" || idle_enabled.to_lowercase() == "true";
        }

        if let Ok(max_syncs) = std::env::var("ASGARD_MAIL_MAX_CONCURRENT_SYNCS") {
            if let Ok(syncs) = max_syncs.parse() {
                config.sync.max_concurrent_syncs = syncs;
            }
        }

        if let Ok(encrypt_cache) = std::env::var("ASGARD_MAIL_ENCRYPT_CACHE") {
            config.security.encrypt_cache = encrypt_cache == "1" || encrypt_cache.to_lowercase() == "true";
        }

        if let Ok(block_images) = std::env::var("ASGARD_MAIL_BLOCK_REMOTE_IMAGES") {
            config.security.block_remote_images = block_images == "1" || block_images.to_lowercase() == "true";
        }

        if let Ok(strip_tracking) = std::env::var("ASGARD_MAIL_STRIP_TRACKING_PIXELS") {
            config.security.strip_tracking_pixels = strip_tracking == "1" || strip_tracking.to_lowercase() == "true";
        }

        if let Ok(notifications) = std::env::var("ASGARD_MAIL_SHOW_TRAY_NOTIFICATIONS") {
            config.notifications.enable_notifications = notifications == "1" || notifications.to_lowercase() == "true";
        }

        if let Ok(auto_mark_read) = std::env::var("ASGARD_MAIL_AUTO_MARK_READ_SECONDS") {
            if let Ok(seconds) = auto_mark_read.parse() {
                config.ui.auto_mark_read_delay = seconds;
            }
        }

        if let Ok(cache_size) = std::env::var("ASGARD_MAIL_CACHE_SIZE_MB") {
            if let Ok(size) = cache_size.parse() {
                config.cache.max_cache_size_mb = size;
            }
        }

        config
    }

    /// Get the configuration file path
    pub fn config_file_path(&self) -> PathBuf {
        self.app.config_dir.join("config.toml")
    }

    /// Get the accounts configuration file path
    pub fn accounts_file_path(&self) -> PathBuf {
        self.app.config_dir.join("accounts.toml")
    }

    /// Get the database file path
    pub fn database_file_path(&self) -> PathBuf {
        self.app.data_dir.join("asgard-mail.db")
    }

    /// Get the search index directory
    pub fn search_index_dir(&self) -> PathBuf {
        self.search.index_dir.clone()
    }

    /// Get the cache directory
    pub fn cache_dir(&self) -> PathBuf {
        self.app.cache_dir.clone()
    }

    /// Get the data directory
    pub fn data_dir(&self) -> PathBuf {
        self.app.data_dir.clone()
    }

    /// Validate the configuration
    pub fn validate(&self) -> AsgardResult<()> {
        // Validate directories
        if !self.app.config_dir.exists() {
            std::fs::create_dir_all(&self.app.config_dir)
                .map_err(|_| AsgardError::config("Failed to create config directory"))?;
        }

        if !self.app.cache_dir.exists() {
            std::fs::create_dir_all(&self.app.cache_dir)
                .map_err(|_| AsgardError::config("Failed to create cache directory"))?;
        }

        if !self.app.data_dir.exists() {
            std::fs::create_dir_all(&self.app.data_dir)
                .map_err(|_| AsgardError::config("Failed to create data directory"))?;
        }

        // Validate sync settings
        if self.sync.default_sync_interval == 0 {
            return Err(AsgardError::config("Sync interval cannot be zero"));
        }

        if self.sync.max_concurrent_syncs == 0 {
            return Err(AsgardError::config("Max concurrent syncs cannot be zero"));
        }

        // Validate cache settings
        if self.cache.max_cache_size_mb == 0 {
            return Err(AsgardError::config("Cache size cannot be zero"));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert_eq!(config.app.name, "Asgard Mail");
        assert_eq!(config.ui.theme, ThemePreference::System);
        assert!(config.sync.enable_idle);
        assert!(config.security.block_remote_images);
    }

    #[test]
    fn test_config_save_load() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");
        
        let mut config = Config::default();
        config.app.debug = true;
        config.ui.theme = ThemePreference::Dark;
        
        config.save(&config_path).unwrap();
        assert!(config_path.exists());
        
        let loaded_config = Config::load(&config_path).unwrap();
        assert_eq!(loaded_config.app.debug, true);
        assert_eq!(loaded_config.ui.theme, ThemePreference::Dark);
    }

    #[test]
    fn test_config_validation() {
        let mut config = Config::default();
        assert!(config.validate().is_ok());
        
        config.sync.default_sync_interval = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_from_env() {
        std::env::set_var("ASGARD_MAIL_DEBUG", "1");
        std::env::set_var("ASGARD_MAIL_THEME", "dark");
        std::env::set_var("ASGARD_MAIL_SYNC_INTERVAL_SECONDS", "600");
        
        let config = Config::load_from_env();
        assert!(config.app.debug);
        assert_eq!(config.ui.theme, ThemePreference::Dark);
        assert_eq!(config.sync.default_sync_interval, 600);
        
        // Clean up
        std::env::remove_var("ASGARD_MAIL_DEBUG");
        std::env::remove_var("ASGARD_MAIL_THEME");
        std::env::remove_var("ASGARD_MAIL_SYNC_INTERVAL_SECONDS");
    }
}
