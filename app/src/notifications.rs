//! Notification system for Asgard Mail

use asgard_core::config::Config;
use asgard_core::error::AsgardResult;
use notify_rust::Notification;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Notification manager for the application
pub struct NotificationManager {
    /// Application configuration
    config: Config,
    /// Notification state
    state: Arc<Mutex<NotificationState>>,
}

/// Internal notification state
struct NotificationState {
    /// Whether notifications are enabled
    enabled: bool,
    /// Last notification time
    last_notification: Option<std::time::Instant>,
    /// Notification cooldown
    cooldown: std::time::Duration,
}

impl NotificationManager {
    /// Create a new notification manager
    pub fn new(config: &Config) -> AsgardResult<Self> {
        let state = Arc::new(Mutex::new(NotificationState {
            enabled: config.notifications.enable_notifications,
            last_notification: None,
            cooldown: std::time::Duration::from_secs(config.notifications.notification_timeout.into()),
        }));

        Ok(Self {
            config: config.clone(),
            state,
        })
    }

    /// Show a notification
    pub async fn show_notification(&self, title: &str, body: &str) -> AsgardResult<()> {
        let mut state = self.state.lock().await;
        
        if !state.enabled {
            return Ok(());
        }

        // Check cooldown
        if let Some(last_time) = state.last_notification {
            if last_time.elapsed() < state.cooldown {
                return Ok(());
            }
        }

        // Create and show notification
        let notification = Notification::new()
            .summary(title)
            .body(body)
            .icon("mail-message-new")
            .timeout(5000) // 5 seconds
            .show();

        match notification {
            Ok(_) => {
                state.last_notification = Some(std::time::Instant::now());
                tracing::info!("Notification shown: {}", title);
            }
            Err(e) => {
                tracing::warn!("Failed to show notification: {}", e);
            }
        }

        Ok(())
    }

    /// Show new email notification
    pub async fn show_new_email_notification(&self, sender: &str, subject: &str) -> AsgardResult<()> {
        let title = "New Email";
        let body = format!("From: {}\nSubject: {}", sender, subject);
        self.show_notification(title, &body).await
    }

    /// Show sync notification
    pub async fn show_sync_notification(&self, message: &str) -> AsgardResult<()> {
        let title = "Email Sync";
        let body = message;
        self.show_notification(title, body).await
    }

    /// Show error notification
    pub async fn show_error_notification(&self, error: &str) -> AsgardResult<()> {
        let title = "Error";
        let body = error;
        self.show_notification(title, body).await
    }

    /// Enable notifications
    pub async fn enable(&self) {
        let mut state = self.state.lock().await;
        state.enabled = true;
    }

    /// Disable notifications
    pub async fn disable(&self) {
        let mut state = self.state.lock().await;
        state.enabled = false;
    }

    /// Check if notifications are enabled
    pub async fn is_enabled(&self) -> bool {
        let state = self.state.lock().await;
        state.enabled
    }

    /// Set notification cooldown
    pub async fn set_cooldown(&self, duration: std::time::Duration) {
        let mut state = self.state.lock().await;
        state.cooldown = duration;
    }
}

impl Clone for NotificationManager {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            state: self.state.clone(),
        }
    }
}