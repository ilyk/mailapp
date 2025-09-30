#!/bin/bash

# Asgard Mail Setup Script for Kali Linux

set -e

echo "Setting up Asgard Mail development environment..."

# Install Rust if not present
if ! command -v cargo >/dev/null 2>&1; then
    echo "Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source ~/.cargo/env
fi

# Install system dependencies
echo "Installing system dependencies..."

# Update package list
sudo apt update

# Install basic build tools
sudo apt install -y build-essential pkg-config libssl-dev

# Install GTK4
sudo apt install -y libgtk-4-dev

# Install SQLite
sudo apt install -y libsqlite3-dev

# Try to install libadwaita (may not be available on older systems)
if apt list --installed | grep -q "libadwaita-1-dev"; then
    echo "libadwaita-1-dev already installed"
elif apt search libadwaita-1-dev 2>/dev/null | grep -q "libadwaita-1-dev"; then
    sudo apt install -y libadwaita-1-dev
else
    echo "libadwaita-1-dev not available, will use GTK4 only"
fi

# Install WebKit (try different versions)
if apt search libwebkit2gtk-4.1-dev 2>/dev/null | grep -q "libwebkit2gtk-4.1-dev"; then
    sudo apt install -y libwebkit2gtk-4.1-dev
    echo "Installed WebKit 4.1"
elif apt search libwebkit2gtk-5.0-dev 2>/dev/null | grep -q "libwebkit2gtk-5.0-dev"; then
    sudo apt install -y libwebkit2gtk-5.0-dev
    echo "Installed WebKit 5.0"
else
    echo "WebKit development files not found"
fi

# Try to install AppIndicator
if apt search libayatana-appindicator3-dev 2>/dev/null | grep -q "libayatana-appindicator3-dev"; then
    sudo apt install -y libayatana-appindicator3-dev
elif apt search libappindicator3-dev 2>/dev/null | grep -q "libappindicator3-dev"; then
    sudo apt install -y libappindicator3-dev
else
    echo "AppIndicator not available, system tray will be disabled"
fi

# Install Rust development tools
echo "Installing Rust development tools..."
cargo install cargo-make || echo "cargo-make installation failed, continuing..."
cargo install cargo-watch || echo "cargo-watch installation failed, continuing..."

echo "Setup complete!"
echo ""
echo "To build Asgard Mail, run:"
echo "  cargo build --workspace"
echo ""
echo "To run in demo mode:"
echo "  cargo run --bin asgard-mail -- --demo"
