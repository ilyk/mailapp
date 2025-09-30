//! Asgard Mail - Main application entry point

use clap::Parser;
// use gtk4::prelude::*;
// use libadwaita::prelude::*;
use tracing::{info, error};
use asgard_core::init;

mod app;
mod windows;
mod widgets;
mod tray;
mod notifications;
mod dbus_service;
mod constants;
mod theming;
mod thread_helpers;
mod email_backend;

use app::AsgardApp;
use dbus_service::AsgardDbusService;
use constants::{DBUS_APP_NAME, DBUS_INTERFACE_PATH, DBUS_INTERFACE_NAME};

/// Command line arguments
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Enable debug mode
    #[arg(short, long)]
    debug: bool,

    /// Run in demo mode with mock data
    #[arg(long)]
    demo: bool,

    /// Configuration directory
    #[arg(long)]
    config_dir: Option<String>,

    /// Cache directory
    #[arg(long)]
    cache_dir: Option<String>,

    /// Data directory
    #[arg(long)]
    data_dir: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse command line arguments
    let args = Args::parse();

    // Initialize logging
    let log_level = if args.debug {
        "debug"
    } else {
        &std::env::var("ASGARD_MAIL_LOG_LEVEL").unwrap_or_else(|_| "info".to_string())
    };

    tracing_subscriber::fmt()
        .with_max_level(match log_level {
            "debug" => tracing::Level::DEBUG,
            "info" => tracing::Level::INFO,
            "warn" => tracing::Level::WARN,
            "error" => tracing::Level::ERROR,
            _ => tracing::Level::INFO,
        })
        .init();

    info!("Starting Asgard Mail v{}", env!("CARGO_PKG_VERSION"));

    // Set environment variables if provided
    if let Some(config_dir) = args.config_dir {
        std::env::set_var("ASGARD_MAIL_CONFIG_DIR", config_dir);
    }
    if let Some(cache_dir) = args.cache_dir {
        std::env::set_var("ASGARD_MAIL_CACHE_DIR", cache_dir);
    }
    if let Some(data_dir) = args.data_dir {
        std::env::set_var("ASGARD_MAIL_DATA_DIR", data_dir);
    }

    // Initialize core library
    if let Err(e) = init() {
        error!("Failed to initialize core library: {}", e);
        return Err(e.into());
    }

    // Initialize GTK
    gtk4::init()?;
    
    // Initialize libadwaita
    libadwaita::init()?;

    // Try to register this instance as the DBus service
    let dbus_service = AsgardDbusService::new();
    
    // Try to register the service - if it fails, another instance is already running
    match dbus_service.register_service().await {
        Ok(_) => {
            info!("Successfully registered as the primary instance");
        }
        Err(e) => {
            error!("Failed to register DBus service (error: {}), assuming another instance is running", e);
            info!("Notifying existing instance to show and exiting");
            // Try to notify the existing instance to show
            if let Ok(connection) = zbus::Connection::session().await {
                let _ = connection.call_method(
                    Some(DBUS_APP_NAME),
                    DBUS_INTERFACE_PATH,
                    Some(DBUS_INTERFACE_NAME),
                    "ShowWindow",
                    &(),
                ).await;
            }
            return Ok(());
        }
    }

    // Create and run the application
    let app = AsgardApp::new(args.demo, dbus_service).await?;
    app.run().await?;

    Ok(())
}
