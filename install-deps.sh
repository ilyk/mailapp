#!/bin/bash

# Install missing dependencies for Asgard Mail
# Run this script with sudo: sudo ./install-deps.sh

echo "Installing dependencies for Asgard Mail..."

# Update package list
apt update

# Install basic build tools
apt install -y build-essential pkg-config libssl-dev

# Install GTK4 and related libraries
apt install -y libgtk-4-dev libgdk-pixbuf-2.0-dev libcairo2-dev libpango1.0-dev libatk1.0-dev

# Install WebKit dependencies
echo "Installing WebKit dependencies..."

# Try different WebKit package versions
if apt search libwebkit2gtk-6.0-dev 2>/dev/null | grep -q "libwebkit2gtk-6.0-dev"; then
    echo "Installing libwebkit2gtk-6.0-dev..."
    apt install -y libwebkit2gtk-6.0-dev
elif apt search libwebkit2gtk-5.0-dev 2>/dev/null | grep -q "libwebkit2gtk-5.0-dev"; then
    echo "Installing libwebkit2gtk-5.0-dev..."
    apt install -y libwebkit2gtk-5.0-dev
elif apt search libwebkit2gtk-4.1-dev 2>/dev/null | grep -q "libwebkit2gtk-4.1-dev"; then
    echo "Installing libwebkit2gtk-4.1-dev..."
    apt install -y libwebkit2gtk-4.1-dev
else
    echo "No suitable WebKit package found, trying libwebkit2gtk-dev..."
    apt install -y libwebkit2gtk-dev || echo "WebKit development files not available"
fi

# Install libsoup
apt install -y libsoup2.4-dev || apt install -y libsoup-2.4-dev || echo "libsoup not available"

# Skip JavaScriptCore as it's usually included with WebKit
echo "JavaScriptCore is typically included with WebKit, skipping separate installation"

# Install libadwaita (try different package names)
if apt list --installed | grep -q "libadwaita-1-dev"; then
    echo "libadwaita-1-dev already installed"
elif apt search libadwaita-1-dev 2>/dev/null | grep -q "libadwaita-1-dev"; then
    apt install -y libadwaita-1-dev
else
    echo "libadwaita-1-dev not available, trying alternative..."
    apt install -y gir1.2-adw-1 || echo "libadwaita development files not found"
fi

# Install SQLite
apt install -y libsqlite3-dev

# Install notification dependencies
apt install -y libnotify-dev

echo ""
echo "✅ Dependencies installed!"
echo ""
echo "Now you can build Asgard Mail with:"
echo "  ./working-build.sh"
