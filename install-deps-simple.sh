#!/bin/bash

# Simple dependency installer for Asgard Mail
# Run with: sudo ./install-deps-simple.sh

echo "Installing essential dependencies for Asgard Mail..."

# Update package list
apt update

# Install basic build tools
echo "Installing build tools..."
apt install -y build-essential pkg-config libssl-dev

# Install GTK4 and related libraries
echo "Installing GTK4 libraries..."
apt install -y libgtk-4-dev libgdk-pixbuf-2.0-dev libcairo2-dev libpango1.0-dev libatk1.0-dev

# Install libadwaita (already installed according to your output)
echo "libadwaita-1-dev already installed"

# Install SQLite (already installed according to your output)
echo "libsqlite3-dev already installed"

# Install notification dependencies
echo "Installing notification libraries..."
apt install -y libnotify-dev

# Try to install WebKit (optional for minimal version)
echo "Attempting to install WebKit..."
apt install -y libwebkit2gtk-4.1-dev 2>/dev/null || \
apt install -y libwebkit2gtk-5.0-dev 2>/dev/null || \
apt install -y libwebkit2gtk-6.0-dev 2>/dev/null || \
echo "WebKit not available - minimal version will work without it"

echo ""
echo "✅ Essential dependencies installed!"
echo ""
echo "You can now build Asgard Mail:"
echo "  ./minimal-build.sh    # Minimal version (always works)"
echo "  ./working-build.sh    # Full version (if WebKit available)"
echo ""
echo "Note: The minimal version works without WebKit and demonstrates"
echo "      all core functionality. WebKit is only needed for HTML"
echo "      email rendering in the full version."
