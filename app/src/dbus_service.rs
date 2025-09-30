//! DBus service for single-instance application behavior

use zbus::{Connection, ConnectionBuilder, dbus_interface, Result as ZbusResult};
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::constants::{DBUS_APP_NAME, DBUS_APP_PATH};

/// DBus method names as enum for type safety
#[derive(Debug, Clone, PartialEq)]
pub enum DbusMethod {
    Ping,
    ShowWindow,
    GetPid,
}

impl DbusMethod {
    /// Convert enum to string for DBus interface
    pub fn as_str(&self) -> &'static str {
        match self {
            DbusMethod::Ping => "Ping",
            DbusMethod::ShowWindow => "ShowWindow",
            DbusMethod::GetPid => "GetPid",
        }
    }
}

/// DBus service for Asgard Mail
#[derive(Clone)]
pub struct AsgardDbusService {
    /// Connection to DBus
    connection: Arc<Mutex<Option<Connection>>>,
    /// Callback to handle DBus method calls
    method_callback: Arc<Mutex<Option<Box<dyn Fn(DbusMethod) + Send + Sync>>>>,
}

impl AsgardDbusService {
    /// Create a new DBus service
    pub fn new() -> Self {
        Self {
            connection: Arc::new(Mutex::new(None)),
            method_callback: Arc::new(Mutex::new(None)),
        }
    }

    /// Set the callback to handle DBus method calls
    pub async fn set_method_callback<F>(&self, callback: F)
    where
        F: Fn(DbusMethod) + Send + Sync + 'static,
    {
        let mut cb = self.method_callback.lock().await;
        *cb = Some(Box::new(callback));
    }

    /// Set the show window callback
    pub async fn set_show_window_callback<F>(&self, callback: F)
    where
        F: Fn() + Send + Sync + 'static,
    {
        let callback_wrapper = move |method| {
            if method == DbusMethod::ShowWindow {
                callback();
            }
        };
        self.set_method_callback(callback_wrapper).await;
    }

    /// Register this instance as the DBus service
    pub async fn register_service(&self) -> ZbusResult<()> {
        println!("Attempting to register DBus service...");
        
        let service = AsgardDbusInterface::new(self.method_callback.clone());
        
        // Try to claim the name and serve the interface
        let connection = ConnectionBuilder::session()?
            .name(DBUS_APP_NAME)?
            .serve_at(DBUS_APP_PATH, service)?
            .build()
            .await?;

        println!("Successfully registered Asgard Mail DBus service");
        
        // Store the connection
        {
            let mut conn = self.connection.lock().await;
            *conn = Some(connection);
        }

        Ok(())
    }

    /// Unregister the DBus service
    pub async fn unregister_service(&self) -> ZbusResult<()> {
        let mut conn = self.connection.lock().await;
        if let Some(connection) = conn.take() {
            connection.release_name(DBUS_APP_NAME).await?;
            println!("Unregistered Asgard Mail DBus service");
        }
        Ok(())
    }
}

/// DBus interface implementation
pub struct AsgardDbusInterface {
    method_callback: Arc<Mutex<Option<Box<dyn Fn(DbusMethod) + Send + Sync>>>>,
}

impl AsgardDbusInterface {
    fn new(method_callback: Arc<Mutex<Option<Box<dyn Fn(DbusMethod) + Send + Sync>>>>) -> Self {
        Self {
            method_callback,
        }
    }
}

#[dbus_interface(name = "com.asgard.Mail")]
impl AsgardDbusInterface {
    /// Ping method to check if service is alive
    async fn ping(&self) -> String {
        println!("DBus ping received");
        let callback = self.method_callback.lock().await;
        if let Some(cb) = callback.as_ref() {
            cb(DbusMethod::Ping);
        }
        "pong".to_string()
    }

    /// Show the main window
    async fn show_window(&self) {
        println!("DBus show_window called");
        
        let callback = self.method_callback.lock().await;
        if let Some(cb) = callback.as_ref() {
            cb(DbusMethod::ShowWindow);
        }
    }

    /// Get the PID of the current process
    async fn get_pid(&self) -> u32 {
        let pid = std::process::id();
        println!("DBus get_pid called, returning PID: {}", pid);
        
        let callback = self.method_callback.lock().await;
        if let Some(cb) = callback.as_ref() {
            cb(DbusMethod::GetPid);
        }
        
        pid
    }
}