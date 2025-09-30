//! Minimal Asgard Mail application

use clap::Parser;
use asgard_core_minimal::{Account, Message, Mailbox};
use uuid::Uuid;

/// Command line arguments
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Run in demo mode with mock data
    #[arg(long)]
    demo: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    
    println!("🚀 Asgard Mail - Minimal Version");
    println!("Version: 0.1.0");
    
    if args.demo {
        println!("\n📧 Demo Mode - Creating sample data...");
        
        // Create demo account
        let account = Account::new(
            "demo@gmail.com".to_string(),
            "Demo Account".to_string(),
        );
        
        // Create demo mailbox
        let mailbox = Mailbox::new(account.id, "Inbox".to_string());
        
        // Create demo messages
        let messages = vec![
            Message::new(
                account.id,
                "Welcome to Asgard Mail!".to_string(),
                "team@asgard.com".to_string(),
                "Welcome to Asgard Mail! This is a demo message.".to_string(),
            ),
            Message::new(
                account.id,
                "Getting Started".to_string(),
                "help@asgard.com".to_string(),
                "Here are some tips to get started with Asgard Mail...".to_string(),
            ),
            Message::new(
                account.id,
                "Features Overview".to_string(),
                "info@asgard.com".to_string(),
                "Asgard Mail includes many features like Gmail OAuth, IMAP sync, and more!".to_string(),
            ),
        ];
        
        println!("✅ Created demo account: {}", account.email);
        println!("✅ Created demo mailbox: {}", mailbox.name);
        println!("✅ Created {} demo messages", messages.len());
        
        println!("\n📋 Demo Data Summary:");
        println!("Account: {} ({})", account.display_name, account.email);
        println!("Mailbox: {}", mailbox.name);
        println!("Messages:");
        for (i, msg) in messages.iter().enumerate() {
            println!("  {}. {} - {}", i + 1, msg.subject, msg.from);
        }
        
        println!("\n🎉 Demo completed successfully!");
        println!("\nTo build the full version with GTK4 UI:");
        println!("1. Install system dependencies: sudo ./install-deps.sh");
        println!("2. Build full version: ./working-build.sh");
    } else {
        println!("\n💡 Use --demo flag to see sample data");
        println!("Example: cargo run --bin asgard-mail-minimal -- --demo");
    }
    
    Ok(())
}
