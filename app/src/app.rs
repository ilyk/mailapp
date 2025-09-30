//! Main application structure for Asgard Mail

use crate::windows::MainWindow;
use crate::notifications::NotificationManager;
use crate::dbus_service::AsgardDbusService;
use crate::theming;
use asgard_core::error::AsgardResult;
use asgard_core::config::Config;
use asgard_core::storage::StorageManager;
// use asgard_core::search::TantivySearchIndex;
use asgard_core::sync::SyncManager;
// use asgard_oauth::TokenManager;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{info, error};
// use gtk4::glib;

/// Main application structure
pub struct AsgardApp {
    /// Application configuration
    config: Config,
    /// Storage manager
    storage: Arc<Mutex<StorageManager>>,
    /// Search index
    // search_index: Arc<Mutex<TantivySearchIndex>>,
    /// Sync manager
    sync_manager: Arc<Mutex<SyncManager>>,
    /// Token manager
    // token_manager: TokenManager,
    /// Notification manager
    notification_manager: NotificationManager,
    /// Main window
    main_window: MainWindow,
    /// Demo mode flag
    demo_mode: bool,
    /// DBus service
    dbus_service: AsgardDbusService,
}

impl AsgardApp {
    /// Create a new Asgard Mail application
    pub async fn new(demo_mode: bool, dbus_service: AsgardDbusService) -> AsgardResult<Self> {
        info!("Initializing Asgard Mail application");

        // Load configuration
        let config = Config::load_from_env();
        config.validate()?;

        // Load CSS theming early
        theming::load_css();

        // Initialize storage
        let storage = Arc::new(Mutex::new(
            StorageManager::new(
                config.database_file_path(),
                config.cache_dir(),
            ).await?
        ));
        storage.lock().await.initialize().await?;

        // Initialize search index
        // let search_index = Arc::new(Mutex::new(
        //     TantivySearchIndex::new(config.search_index_dir())?
        // ));

        // Initialize sync manager
        let sync_manager = Arc::new(Mutex::new(
            SyncManager::new(
                storage.clone(),
                // TODO: Add search index when available - using dummy for now
                Arc::new(Mutex::new(asgard_core::search::SimpleSearchIndex::new())),
                std::time::Duration::from_secs(config.sync.default_sync_interval),
            )
        ));

        // Initialize token manager
        // let token_manager = TokenManager::new()?;

        // Initialize notification manager
        let notification_manager = NotificationManager::new(&config)?;

        // Initialize main window
        let main_window = MainWindow::new(
            config.clone(),
            storage.clone(),
            // search_index.clone(),
            sync_manager.clone(),
            // token_manager.clone(),
            notification_manager.clone(),
            demo_mode,
        ).await?;

        Ok(Self {
            config,
            storage,
            // search_index,
            sync_manager,
            // token_manager,
            notification_manager,
            main_window,
            demo_mode,
            dbus_service,
        })
    }

    /// Run the application
    pub async fn run(mut self) -> AsgardResult<()> {
        info!("Starting Asgard Mail application");

        // Set up DBus callback to show window
        // Note: GTK widgets are not Send/Sync, so we disable this for now
        // let main_window = self.main_window.clone();
        // self.dbus_service.set_show_window_callback(move || {
        //     let main_window = main_window.clone();
        //     glib::MainContext::default().spawn_local(async move {
        //         main_window.show();
        //     });
        // }).await;

        // Set up quit callback on main window
        // We'll use a different approach - exit the process directly
        let quit_callback = move || {
            std::process::exit(0);
        };
        self.main_window.set_quit_callback(quit_callback).await;

        // Start background sync
        {
            let mut sync_manager = self.sync_manager.lock().await;
            sync_manager.start_background_sync().await?;
        }

        // Show main window
        self.main_window.show();

        // System tray functionality can be added here if needed

        // Run the main event loop
        if self.demo_mode {
            info!("Running in demo mode");
            self.run_demo_mode().await?;
        } else {
            self.run_normal_mode().await?;
        }

        // Cleanup
        info!("Shutting down Asgard Mail");
        self.cleanup().await?;

        // Unregister DBus service
        if let Err(e) = self.dbus_service.unregister_service().await {
            error!("Failed to unregister DBus service: {}", e);
        }

        Ok(())
    }

    /// Run in normal mode
    async fn run_normal_mode(&mut self) -> AsgardResult<()> {
        // Start GTK main loop
        info!("Starting GTK main loop...");
        
        // Run GTK main loop in a separate task
        let gtk_handle = tokio::task::spawn_blocking(|| {
            // gtk4::main(); // Handled by AdwApplication::run()
        });
        
        // Wait for either GTK to exit or Ctrl+C
        tokio::select! {
            _ = gtk_handle => {
                info!("GTK main loop exited");
            }
            _ = tokio::signal::ctrl_c() => {
                info!("Received Ctrl+C, shutting down...");
                // gtk4::main_quit(); // Handled by AdwApplication::run()
            }
        }
        
        Ok(())
    }

    /// Run in demo mode
    async fn run_demo_mode(&mut self) -> AsgardResult<()> {
        info!("Demo mode: Creating sample data");

        // Create demo account
        let demo_account = self.create_demo_account().await?;

        // Add demo account to sync manager
        {
            let sync_manager = self.sync_manager.lock().await;
            sync_manager.add_account(demo_account).await?;
        }

        // Create demo mailboxes and messages
        self.create_demo_data().await?;

        // Run GTK main loop for demo
        info!("Starting GTK main loop for demo...");
        
        // Run GTK main loop in a separate task
        let gtk_handle = tokio::task::spawn_blocking(|| {
            // gtk4::main(); // Handled by AdwApplication::run()
        });
        
        // Wait for either GTK to exit or Ctrl+C
        tokio::select! {
            _ = gtk_handle => {
                info!("GTK main loop exited");
            }
            _ = tokio::signal::ctrl_c() => {
                info!("Received Ctrl+C, shutting down...");
                // gtk4::main_quit(); // Handled by AdwApplication::run()
            }
        }

        Ok(())
    }

    /// Create demo account
    async fn create_demo_account(&self) -> AsgardResult<asgard_core::account::Account> {
        use asgard_core::account::{Account, GmailOAuthConfig};

        let oauth_config = GmailOAuthConfig {
            client_id: "demo-client-id".to_string(),
            client_secret: "demo-client-secret".to_string(),
            access_token: Some("demo-access-token".to_string()),
            refresh_token: Some("demo-refresh-token".to_string()),
            token_expires_at: Some(time::OffsetDateTime::now_utc() + time::Duration::hours(1)),
            scopes: vec!["https://www.googleapis.com/auth/gmail.readonly".to_string()],
        };

        let account = Account::new_gmail(
            "demo@gmail.com".to_string(),
            Some("Demo Account".to_string()),
            oauth_config,
        )?;

        // Store account in database
        {
            let mut storage = self.storage.lock().await;
            storage.database_mut().create_account(&account).await?;
        }

        Ok(account)
    }

    /// Create demo data
    async fn create_demo_data(&self) -> AsgardResult<()> {
        use asgard_core::mailbox::Mailbox;
        use asgard_core::message::{Message, MessageHeaders, EmailAddress, MessageImportance};
        use std::collections::HashMap;

        // Create demo mailboxes
        let inbox = Mailbox::new_inbox(uuid::Uuid::new_v4());
        let sent = Mailbox::new_sent(uuid::Uuid::new_v4(), "Sent".to_string());

        // Store mailboxes
        {
            let mut storage = self.storage.lock().await;
            storage.database_mut().create_mailbox(&inbox).await?;
            storage.database_mut().create_mailbox(&sent).await?;
        }

        // Create demo messages
        let demo_messages = vec![
            ("Welcome to Asgard Mail", "Welcome to Asgard Mail! This is a demo message to show you how the application works."),
            ("Getting Started", "Here are some tips to get started with Asgard Mail..."),
            ("Features Overview", "Asgard Mail includes many features like Gmail OAuth, IMAP sync, and more!"),
        ];

        for (subject, body) in &demo_messages {
            let headers = MessageHeaders {
                message_id: None,
                in_reply_to: None,
                references: None,
                subject: subject.to_string(),
                from: vec![EmailAddress {
                    name: Some("Demo Sender".to_string()),
                    email: "demo@example.com".to_string(),
                }],
                to: vec![EmailAddress {
                    name: None,
                    email: "demo@gmail.com".to_string(),
                }],
                cc: vec![],
                bcc: vec![],
                reply_to: vec![],
                date: Some(time::OffsetDateTime::now_utc()),
                received_date: Some(time::OffsetDateTime::now_utc()),
                importance: MessageImportance::Normal,
                custom: HashMap::new(),
            };

            let mut message = Message::new(uuid::Uuid::new_v4(), inbox.id, headers);
            
            // Add text part
            let part = asgard_core::message::MessagePart {
                id: "1".to_string(),
                part_type: asgard_core::message::MessagePartType::Text,
                mime_type: "text/plain".to_string(),
                disposition: None,
                filename: None,
                size: body.len(),
                encoding: None,
                content_id: None,
                content_location: None,
                content: Some(body.as_bytes().to_vec()),
                children: vec![],
            };
            message.add_part(part);

            // Store message
            {
                let mut storage = self.storage.lock().await;
                storage.database_mut().create_message(&message).await?;
            }

            // Add to search index
            {
                // let mut search_index = self.search_index.lock().await;
                // search_index.add_message(&message)?;
            }
        }

        info!("Created demo data: {} mailboxes, {} messages", 2, demo_messages.len());
        Ok(())
    }

    /// Cleanup resources
    async fn cleanup(&mut self) -> AsgardResult<()> {
        info!("Cleaning up application resources");

        // Stop background sync
        {
            let mut sync_manager = self.sync_manager.lock().await;
            sync_manager.stop_background_sync().await?;
        }

        // Close storage
        {
            let _storage = self.storage.lock().await;
            // storage.close().await?; // TODO: Fix this - need to move out of mutex
        }

        Ok(())
    }

    /// Quit the application
    pub async fn quit(&mut self) -> AsgardResult<()> {
        info!("Quitting Asgard Mail application");
        
        // Cleanup resources
        self.cleanup().await?;
        
        // Unregister DBus service
        if let Err(e) = self.dbus_service.unregister_service().await {
            error!("Failed to unregister DBus service: {}", e);
        }
        
        // Exit the application
        std::process::exit(0);
    }
}
