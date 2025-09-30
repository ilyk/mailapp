//! Example: How to find PID by asking DBus
//! 
//! This example demonstrates multiple ways to get the process ID (PID) 
//! of a DBus service using zbus.

use simple_gtk_app::dbus_service::*;
use simple_gtk_app::constants::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== DBus PID Query Examples ===\n");

    // Method 1: Using a custom get_pid method on your service
    println!("Method 1: Custom get_pid method");
    match get_asgard_mail_pid().await {
        Ok(pid) => println!("✅ Asgard Mail PID (custom method): {}", pid),
        Err(e) => println!("❌ Failed to get PID via custom method: {}", e),
    }

    // Method 2: Using the standard DBus GetConnectionUnixProcessID method
    println!("\nMethod 2: Standard DBus GetConnectionUnixProcessID");
    match get_asgard_mail_owner_pid().await {
        Ok(pid) => println!("✅ Asgard Mail PID (standard method): {}", pid),
        Err(e) => println!("❌ Failed to get PID via standard method: {}", e),
    }

    // Method 3: Generic function for any DBus service
    println!("\nMethod 3: Generic service PID query");
    match get_service_owner_pid("org.freedesktop.Notifications").await {
        Ok(pid) => println!("✅ Notification service PID: {}", pid),
        Err(e) => println!("❌ Failed to get notification service PID: {}", e),
    }

    // Method 4: Command-line equivalent
    println!("\n=== Command Line Equivalents ===");
    println!("To get PID via command line, use:");
    println!("```bash");
    println!("# Method 1: Custom get_pid method");
    println!("dbus-send --session --print-reply \\");
    println!("  --dest={} \\", DBUS_APP_NAME);
    println!("  {} \\", DBUS_APP_PATH);
    println!("  {}.get_pid", DBUS_INTERFACE_NAME);
    println!();
    println!("# Method 2: Standard DBus method");
    println!("dbus-send --session --print-reply \\");
    println!("  --dest=org.freedesktop.DBus \\");
    println!("  /org/freedesktop/DBus \\");
    println!("  org.freedesktop.DBus.GetConnectionUnixProcessID \\");
    println!("  string:{}", DBUS_APP_NAME);
    println!("```");

    Ok(())
}
