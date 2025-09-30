# Asgard Mail

A modern, production-ready email client for GNOME 43+ written in Rust. Asgard Mail provides a clean, Apple Mail-inspired interface with full Gmail OAuth support and traditional IMAP/SMTP/POP3 compatibility.

## Features

- **Gmail OAuth2 Integration**: Secure OAuth2 with PKCE flow, no password storage
- **Multi-Protocol Support**: IMAP, SMTP, and POP3 with TLS encryption
- **Apple Mail UI**: Clean 3-pane layout with smooth animations
- **System Tray**: Minimize to tray with background sync
- **Full-Text Search**: Local Tantivy indexing for fast message search
- **Dark Mode**: First-class dark mode support via libadwaita
- **Desktop Notifications**: Real-time new mail notifications
- **HTML Rendering**: Secure HTML message viewing with WebKit
- **Threaded Conversations**: Gmail-style conversation threading
- **Offline Support**: Local caching with SQLite storage

## Screenshots

*Coming soon - the app will feature a clean, modern interface matching GNOME design guidelines*

## Installation

### Dependencies

#### Ubuntu/Debian
```bash
sudo apt install libgtk-4-dev libadwaita-1-dev libwebkit2gtk-5.0-dev \
                 libayatana-appindicator3-dev libsqlite3-dev \
                 libssl-dev pkg-config build-essential
```

#### Fedora/RHEL
```bash
sudo dnf install gtk4-devel libadwaita-devel webkit2gtk5-devel \
                 libayatana-appindicator3-devel sqlite-devel \
                 openssl-devel pkg-config gcc
```

#### Arch Linux
```bash
sudo pacman -S gtk4 libadwaita webkit2gtk libayatana-appindicator3 \
               sqlite openssl pkg-config base-devel
```

### Building from Source

```bash
git clone https://github.com/asgard/asgard-mail.git
cd asgard-mail
make setup
make build
make run
```

## Gmail OAuth Setup

1. **Create Google Cloud Project**:
   - Go to [Google Cloud Console](https://console.cloud.google.com/)
   - Create a new project or select existing one
   - Enable Gmail API

2. **Configure OAuth Consent Screen**:
   - Go to "APIs & Services" > "OAuth consent screen"
   - Choose "External" user type
   - Fill in app information (name: "Asgard Mail")
   - Add your email to test users

3. **Create OAuth Credentials**:
   - Go to "APIs & Services" > "Credentials"
   - Click "Create Credentials" > "OAuth 2.0 Client IDs"
   - Choose "Desktop application"
   - Add redirect URI: `http://127.0.0.1:8080`

4. **Configure Asgard Mail**:
   ```bash
   mkdir -p ~/.config/asgard-mail
   cat > ~/.config/asgard-mail/google_oauth.json << EOF
   {
     "client_id": "your-client-id.googleusercontent.com",
     "client_secret": "your-client-secret"
   }
   EOF
   ```

## Usage

### Adding Accounts

1. **Gmail Account**:
   - Click "Add Account" > "Gmail"
   - Complete OAuth flow in browser
   - Account will sync automatically

2. **Generic IMAP/SMTP**:
   - Click "Add Account" > "Other"
   - Enter server details and credentials
   - Test connection before saving

3. **POP3 Account**:
   - Choose POP3 option in account wizard
   - Configure server settings and sync preferences

### Keyboard Shortcuts

- `Ctrl+N` - Compose new message
- `Ctrl+R` - Reply
- `Ctrl+Shift+R` - Reply All
- `Ctrl+F` - Forward
- `Delete` - Delete message
- `Ctrl+A` - Archive
- `Ctrl+M` - Mark as read/unread
- `Ctrl+L` - Move to folder/Label
- `Ctrl+K` - Quick search
- `Ctrl+Q` - Quit application

### System Tray

- Click tray icon to restore window
- Right-click for menu: Open, Compose, Pause Sync, Quit
- Unread count badge shows total unread messages
- Background sync continues when minimized

## Configuration

Configuration files are stored in `~/.config/asgard-mail/`:

- `accounts.toml` - Account settings (encrypted)
- `app.toml` - Application preferences
- `google_oauth.json` - Gmail OAuth credentials

### Environment Variables

- `ASGARD_MAIL_DEBUG=1` - Enable debug logging
- `ASGARD_MAIL_CONFIG_DIR` - Override config directory
- `ASGARD_MAIL_CACHE_DIR` - Override cache directory

## Development

### Prerequisites

- Rust 1.70+
- GTK4 development libraries
- libadwaita development libraries
- WebKit2GTK development libraries

### Building

```bash
make setup    # Install dependencies
make dev      # Development build
make run      # Run application
make test     # Run tests
make lint     # Run linter
make fmt      # Format code
```

### Project Structure

```
asgard-mail/
├── app/                 # GTK application and UI
├── core/               # Core business logic
│   ├── account/        # Account management
│   ├── storage/        # SQLite and caching
│   ├── sync/           # IMAP/SMTP/POP3 sync
│   └── search/         # Tantivy full-text search
├── oauth/              # Gmail OAuth implementation
├── ui-components/      # Reusable UI widgets
├── theming/            # Styles and themes
└── crates/pop3/        # Minimal POP3 client
```

### Testing

```bash
make test              # Run all tests
make test-unit         # Unit tests only
make test-integration  # Integration tests
make test-coverage     # Generate coverage report
```

## Security & Privacy

- **Encrypted Storage**: Account credentials stored in system keyring
- **TLS Everywhere**: All connections use TLS 1.2+
- **HTML Sanitization**: Messages rendered in sandboxed WebKit
- **No Telemetry**: No analytics or data collection
- **Local Search**: Full-text search runs locally
- **Image Blocking**: Remote images blocked by default

## Troubleshooting

### Common Issues

1. **OAuth Flow Fails**:
   - Check redirect URI matches exactly: `http://127.0.0.1:8080`
   - Ensure Gmail API is enabled in Google Cloud Console
   - Verify client credentials in `google_oauth.json`

2. **IMAP Connection Issues**:
   - Check server settings and port numbers
   - Verify TLS/SSL configuration
   - Test with `openssl s_client -connect imap.gmail.com:993`

3. **System Tray Not Working**:
   - Install `libayatana-appindicator3-dev`
   - Check if your desktop environment supports AppIndicator
   - Try running with `XDG_CURRENT_DESKTOP=GNOME`

4. **Build Errors**:
   - Ensure all development libraries are installed
   - Check Rust version: `rustc --version` (should be 1.70+)
   - Clean build: `cargo clean && make build`

### Debug Mode

Enable debug logging:
```bash
ASGARD_MAIL_DEBUG=1 asgard-mail
```

Logs are written to `~/.local/share/asgard-mail/logs/`

## Contributing

1. Fork the repository
2. Create a feature branch: `git checkout -b feature-name`
3. Make changes and add tests
4. Run tests: `make test`
5. Submit a pull request

## License

MIT License - see [LICENSE](LICENSE) file for details.

## Acknowledgments

- GNOME Foundation for GTK4 and libadwaita
- Rust community for excellent crates
- Apple for Mail.app design inspiration
- Gmail team for OAuth2 implementation guidance
