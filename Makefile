# Asgard Mail Makefile

.PHONY: setup build dev run test lint fmt clean install uninstall flatpak

# Default target
all: build

# Setup development environment
setup:
	@echo "Setting up Asgard Mail development environment..."
	@if ! command -v cargo >/dev/null 2>&1; then \
		echo "Installing Rust..."; \
		curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y; \
		. ~/.cargo/env; \
	fi
	@echo "Installing system dependencies..."
	@if command -v apt >/dev/null 2>&1; then \
		echo "Installing dependencies for Ubuntu/Debian/Kali..."; \
		sudo apt update; \
		sudo apt install -y libgtk-4-dev libsqlite3-dev libssl-dev pkg-config build-essential; \
		if apt search libadwaita-1-dev 2>/dev/null | grep -q "libadwaita-1-dev"; then \
			sudo apt install -y libadwaita-1-dev; \
		else \
			echo "libadwaita-1-dev not available, trying alternative..."; \
			sudo apt install -y gir1.2-adw-1 || echo "libadwaita development files not found"; \
		fi; \
		if apt search libayatana-appindicator3-dev 2>/dev/null | grep -q "libayatana-appindicator3-dev"; then \
			sudo apt install -y libayatana-appindicator3-dev; \
		else \
			echo "libayatana-appindicator3-dev not available, trying alternative..."; \
			sudo apt install -y libappindicator3-dev || echo "AppIndicator not available"; \
		fi; \
		if apt search libwebkit2gtk-6.0-dev 2>/dev/null | grep -q "libwebkit2gtk-6.0-dev"; then \
			sudo apt install -y libwebkit2gtk-6.0-dev; \
		elif apt search libwebkit2gtk-5.0-dev 2>/dev/null | grep -q "libwebkit2gtk-5.0-dev"; then \
			sudo apt install -y libwebkit2gtk-5.0-dev; \
		else \
			sudo apt install -y libwebkit2gtk-4.1-dev; \
		fi; \
	elif command -v dnf >/dev/null 2>&1; then \
		echo "Installing dependencies for Fedora/RHEL..."; \
		sudo dnf install -y gtk4-devel libadwaita-devel webkit2gtk6-devel libayatana-appindicator3-devel sqlite-devel openssl-devel pkg-config gcc; \
	elif command -v pacman >/dev/null 2>&1; then \
		echo "Installing dependencies for Arch Linux..."; \
		sudo pacman -S --noconfirm gtk4 libadwaita webkit2gtk libayatana-appindicator3 sqlite openssl pkg-config base-devel; \
	else \
		echo "Please install dependencies manually for your distribution"; \
		echo "Required packages: gtk4, libadwaita, webkit2gtk, libayatana-appindicator3, sqlite, openssl"; \
	fi
	@echo "Installing Rust dependencies..."
	cargo install cargo-make cargo-watch cargo-tarpaulin
	@echo "Setup complete!"

# Development build
dev:
	cargo build --workspace

# Release build
build:
	cargo build --release --workspace

# Run the application
run: build
	./target/release/asgard-mail

# Run in development mode
dev-run:
	cargo run --bin asgard-mail

# Run with demo data
demo: build
	./target/release/asgard-mail --demo

# Run tests
test:
	cargo test --workspace

# Run unit tests only
test-unit:
	cargo test --workspace --lib

# Run integration tests
test-integration:
	cargo test --workspace --test '*'

# Generate test coverage
test-coverage:
	cargo tarpaulin --workspace --out Html --output-dir coverage

# Run linter
lint:
	cargo clippy --workspace -- -D warnings

# Format code
fmt:
	cargo fmt --all

# Clean build artifacts
clean:
	cargo clean

# Install to system
install: build
	@echo "Installing Asgard Mail..."
	sudo cp target/release/asgard-mail /usr/local/bin/
	sudo cp assets/asgard-mail.desktop /usr/share/applications/
	sudo cp assets/icons/asgard-mail.svg /usr/share/icons/hicolor/scalable/apps/
	sudo gtk-update-icon-cache /usr/share/icons/hicolor/ || true
	@echo "Installation complete!"

# Uninstall from system
uninstall:
	@echo "Uninstalling Asgard Mail..."
	sudo rm -f /usr/local/bin/asgard-mail
	sudo rm -f /usr/share/applications/asgard-mail.desktop
	sudo rm -f /usr/share/icons/hicolor/scalable/apps/asgard-mail.svg
	sudo gtk-update-icon-cache /usr/share/icons/hicolor/ || true
	@echo "Uninstallation complete!"

# Build Flatpak package
flatpak:
	@echo "Building Flatpak package..."
	flatpak-builder --repo=asgard-mail-repo build-dir com.asgard.Mail.json
	flatpak build-bundle asgard-mail-repo asgard-mail.flatpak com.asgard.Mail

# Install Flatpak package
flatpak-install: flatpak
	flatpak install --user asgard-mail.flatpak

# Development with file watching
watch:
	cargo watch -x run

# Check for updates
check:
	cargo outdated

# Update dependencies
update:
	cargo update

# Security audit
audit:
	cargo audit

# Generate documentation
docs:
	cargo doc --workspace --open

# Benchmark
bench:
	cargo bench

# Help
help:
	@echo "Asgard Mail Makefile"
	@echo ""
	@echo "Available targets:"
	@echo "  setup          - Set up development environment"
	@echo "  build          - Build release version"
	@echo "  dev            - Build development version"
	@echo "  run            - Run release version"
	@echo "  dev-run        - Run development version"
	@echo "  demo           - Run with demo data"
	@echo "  test           - Run all tests"
	@echo "  test-unit      - Run unit tests only"
	@echo "  test-integration - Run integration tests"
	@echo "  test-coverage  - Generate test coverage report"
	@echo "  lint           - Run clippy linter"
	@echo "  fmt            - Format code with rustfmt"
	@echo "  clean          - Clean build artifacts"
	@echo "  install        - Install to system"
	@echo "  uninstall      - Uninstall from system"
	@echo "  flatpak        - Build Flatpak package"
	@echo "  flatpak-install - Install Flatpak package"
	@echo "  watch          - Run with file watching"
	@echo "  check          - Check for dependency updates"
	@echo "  update         - Update dependencies"
	@echo "  audit          - Security audit"
	@echo "  docs           - Generate and open documentation"
	@echo "  bench          - Run benchmarks"
	@echo "  help           - Show this help"
