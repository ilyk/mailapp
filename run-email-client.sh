#!/bin/bash

# Run the functional Asgard Mail email client

echo "🚀 Starting Asgard Mail - Functional Email Client..."

# Build the email client
echo "Building email client..."
~/.rustup/toolchains/1.85-x86_64-unknown-linux-gnu/bin/cargo build --bin simple-gtk-app

if [ $? -eq 0 ]; then
    echo "✅ Build successful!"
    echo ""
    echo "📧 Starting Asgard Mail with real email data..."
    echo "Features:"
    echo "  ✅ Apple Mail-inspired 3-pane interface"
    echo "  ✅ Real email backend with sample messages"
    echo "  ✅ Mailbox navigation (Inbox, Sent, Drafts, etc.)"
    echo "  ✅ Message list with real email data"
    echo "  ✅ Message viewer with full content"
    echo "  ✅ Unread message counts"
    echo "  ✅ Time formatting (2m, 1h, Dec 15)"
    echo ""
    
    # Run the email client
    DISPLAY=:0 ~/.rustup/toolchains/1.85-x86_64-unknown-linux-gnu/bin/cargo run --bin simple-gtk-app
else
    echo "❌ Build failed!"
    exit 1
fi
