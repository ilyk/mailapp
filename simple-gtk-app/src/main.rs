//! Apple Mail-inspired GTK4 email client

use gtk4::prelude::*;
use libadwaita::prelude::*;
use libadwaita::Application as AdwApplication;
use gio::Menu;
use email_backend::EmailBackend;
use thread_helpers::Thread;
use std::rc::Rc;
use std::cell::RefCell;
use gtk4::Align;
use uuid::Uuid;
use time::OffsetDateTime;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::fs::File;
use std::io::Write;

mod dbus_service;
use dbus_service::AsgardDbusService;

static PROGRAMMATIC_SHOW: AtomicBool = AtomicBool::new(false);

fn get_lock_file_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("asgardmail")
        .join("app.lock")
}

fn create_lock_file() -> Result<(), Box<dyn std::error::Error>> {
    let lock_path = get_lock_file_path();
    std::fs::create_dir_all(lock_path.parent().unwrap())?;
    let mut file = File::create(&lock_path)?;
    file.write_all(std::process::id().to_string().as_bytes())?;
    Ok(())
}

fn remove_lock_file() -> Result<(), Box<dyn std::error::Error>> {
    let lock_path = get_lock_file_path();
    if lock_path.exists() {
        std::fs::remove_file(&lock_path)?;
    }
    Ok(())
}

fn is_app_running() -> bool {
    let lock_path = get_lock_file_path();
    
    if !lock_path.exists() {
        return false;
    }
    
    // Check if the process ID in the lock file is still running
    if let Ok(contents) = std::fs::read_to_string(&lock_path) {
        if let Ok(pid) = contents.trim().parse::<u32>() {
            // Check if process is still running
            let is_running = std::process::Command::new("kill")
                .args(["-0", &pid.to_string()])
                .status()
                .map(|status| status.success())
                .unwrap_or(false);
            
            if !is_running {
                // Process is not running, clean up stale lock file
                let _ = remove_lock_file();
                return false;
            }
            
            return true;
        }
    }
    
    // If we can't parse the PID or check if it's running, clean up and assume it's not running
    let _ = remove_lock_file();
    false
}

mod theming;
mod widgets;
mod thread_helpers;
mod threading_test;
mod goa_ffi;
mod constants;

use crate::constants::{DBUS_APP_NAME, DBUS_APP_PATH, DBUS_INTERFACE_NAME};
use crate::dbus_service::DbusMethod;

struct AppState {
    backend: Rc<RefCell<EmailBackend>>,
    current_account: Option<Uuid>,
    current_mailbox: Option<String>,
    current_thread_id: Option<String>,
    accordion_states: Rc<RefCell<HashMap<String, bool>>>,
}

#[derive(Clone)]
struct Reader {
    root: gtk4::Box,
    meta_count: gtk4::Label,
    scroller: gtk4::ScrolledWindow,
    cards: gtk4::Box,
}

struct SystemTray {
    window: libadwaita::ApplicationWindow,
    is_visible: bool,
}

impl Reader {
    fn clear(&self) {
        // Clear all message cards
        while let Some(child) = self.cards.first_child() {
            self.cards.remove(&child);
        }
        
        // Reset the meta count
        self.meta_count.set_text("No messages selected");
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct UIState {
    // Window state
    window_width: i32,
    window_height: i32,
    window_x: i32,
    window_y: i32,
    window_maximized: bool,
    
    // Panel sizes
    sidebar_width: i32,
    middle_width: i32,
    right_width: i32,
    
    // Sidebar state
    sidebar_visible: bool,
    
    // Current selection
    current_mailbox: Option<String>,
    current_thread_id: Option<String>,
    
    // Accordion states (mailbox_name -> expanded)
    accordion_states: HashMap<String, bool>,
    
    // Scroll positions
    sidebar_scroll_position: f64,
    message_list_scroll_position: f64,
    reader_scroll_position: f64,
    
    // Category selections (category -> active)
    category_states: HashMap<String, bool>,
}

impl Default for UIState {
    fn default() -> Self {
        Self {
            window_width: 1400,
            window_height: 900,
            window_x: 100,
            window_y: 100,
            window_maximized: false,
            sidebar_width: 260,
            middle_width: 420,
            right_width: 720,
            sidebar_visible: true,
            current_mailbox: Some("ALL_INBOXES".to_string()),
            current_thread_id: None,
            accordion_states: HashMap::new(),
            sidebar_scroll_position: 0.0,
            message_list_scroll_position: 0.0,
            reader_scroll_position: 0.0,
            category_states: {
                let mut states = HashMap::new();
                states.insert("Primary".to_string(), true);
                states
            },
        }
    }
}

fn get_config_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("asgardmail")
        .join("ui_state.json")
}

// GTK4's AdwApplication automatically handles single-instance behavior via D-Bus
// When a second instance is started, the activate signal is called on the existing instance

fn save_ui_state(state: &UIState) -> Result<(), Box<dyn std::error::Error>> {
    let config_path = get_config_path();
    
    // Create parent directory if it doesn't exist
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)?;
    }
    
    let json = serde_json::to_string_pretty(state)?;
    fs::write(&config_path, json)?;
    
    println!("UI state saved to: {:?}", config_path);
    Ok(())
}

fn load_ui_state() -> UIState {
    let config_path = get_config_path();
    
    match fs::read_to_string(&config_path) {
        Ok(content) => {
            match serde_json::from_str::<UIState>(&content) {
                Ok(state) => {
                    println!("UI state loaded from: {:?}", config_path);
                    state
                }
                Err(e) => {
                    println!("Failed to parse UI state: {}, using defaults", e);
                    UIState::default()
                }
            }
        }
        Err(_) => {
            println!("No UI state file found, using defaults");
            UIState::default()
        }
    }
}

fn create_system_tray(window: &libadwaita::ApplicationWindow) {
    println!("Setting up modern GTK4 system integration...");
    
    // Set up desktop notifications for when app is hidden
    let notification = gio::Notification::new("Asgard Mail");
    notification.set_body(Some("Email client is running in the background"));
    notification.set_icon(&gio::ThemedIcon::new("mail-message-new"));
    
    // Connect window close to show notification and hide instead of quit
    let window_clone = window.clone();
    let notification_clone = notification.clone();
    window.connect_close_request(move |window| {
        // Show notification that app is running in background
        if let Some(app) = window.application() {
            app.send_notification(Some("asgard-mail-background"), &notification_clone);
        }
        
        // Hide the window instead of closing
        println!("Hiding window instead of closing #2");
        window.hide();
        gtk4::glib::Propagation::Stop
    });
    
    // Note: Single-instance behavior is handled in the main app.connect_activate handler
    // This function only sets up the close request handler for system tray behavior
    
    println!("✅ Modern GTK4 system integration enabled!");
    println!("");
    println!("Features:");
    println!("• Desktop notifications when app goes to background");
    println!("• Proper application launcher integration");
    println!("• Hide to background instead of closing");
    println!("• System notification area integration");
    println!("");
    println!("When you close the window, you'll get a notification");
    println!("and can show the app again via the application launcher.");
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Fork the process to detach from CLI
    match unsafe { libc::fork() } {
        -1 => {
            eprintln!("Failed to fork process");
            return Err("Failed to fork process".into());
        }
        0 => {
            // Child process - continue with application
            println!("🔄 Forked to background process (PID: {})", std::process::id());
            
            // Create a new session to fully detach from parent
            unsafe { libc::setsid() };
            
            // Ensure we can still access the display
            if std::env::var("DISPLAY").is_err() && std::env::var("WAYLAND_DISPLAY").is_err() {
                eprintln!("Warning: No display environment found after fork");
                // Try to set a default DISPLAY if none exists
                std::env::set_var("DISPLAY", ":0");
            }
        }
        _ => {
            // Parent process - exit immediately
            println!("✅ Parent process exiting, child continues in background");
            std::process::exit(0);
        }
    }
    
    // Initialize GTK4 first
    if let Err(e) = gtk4::init() {
        eprintln!("Failed to initialize GTK4: {}", e);
        return Err(e.into());
    }
    
    // Initialize libadwaita
    if let Err(e) = libadwaita::init() {
        eprintln!("Failed to initialize libadwaita: {}", e);
        return Err(e.into());
    }

    // Try to register DBus service for single-instance behavior
    let dbus_service = AsgardDbusService::new();
    match dbus_service.register_service_sync() {
        Ok(_) => {
            println!("Successfully registered as the primary instance");
        }
        Err(e) => {
            println!("Failed to register DBus service (error: {}), assuming another instance is running", e);
            println!("Notifying existing instance to show and exiting");
            // Try to notify the existing instance to show
            println!("Attempting to notify existing instance...");
            // Use zbus blocking connection for synchronous DBus call
            match zbus::blocking::Connection::session() {
                Ok(connection) => {
                    println!("Connected to DBus, calling ShowWindow...");
                    match connection.call_method(
                        Some(DBUS_APP_NAME),
                        DBUS_APP_PATH,
                        Some(DBUS_INTERFACE_NAME),
                        DbusMethod::ShowWindow.as_str(),
                        &(),
                    ) {
                        Ok(_) => {
                            println!("ShowWindow call completed successfully");
                        }
                        Err(e) => {
                            println!("ShowWindow call failed: {}", e);
                        }
                    }
                }
                Err(e) => {
                    println!("Failed to connect to DBus: {}", e);
                }
            }
            return Ok(());
        }
    }

    let app = AdwApplication::builder()
        .application_id(DBUS_INTERFACE_NAME)
        .build();

    let dbus_service_clone = dbus_service.clone();
    app.connect_activate(move |app| {
        println!("🔄 App activated - PID: {}", std::process::id());
        
        println!("🆕 Creating new instance (PID: {})", std::process::id());
        
        // This is the first instance - proceed with normal initialization
        
        // Load CSS theming early
        theming::load_css();
        
        // Load UI state
        let ui_state = load_ui_state();
        
        // Initialize backend with demo data
        let mut backend = EmailBackend::new();
        
        // Try to get GNOME email accounts, fallback to demo if none found
        let account_id = match goa_ffi::get_gnome_email_accounts() {
            Ok(gnome_accounts) if !gnome_accounts.is_empty() => {
                println!("Found {} GNOME email accounts", gnome_accounts.len());
                
                // Add all GNOME accounts to the backend
                let mut account_ids = Vec::new();
                for gnome_account in gnome_accounts {
                    println!("Adding GNOME account: {} ({})", gnome_account.email, gnome_account.provider);
                    let gnome_account_struct = email_backend::GnomeEmailAccount {
                        id: gnome_account.id,
                        provider: gnome_account.provider,
                        email: gnome_account.email,
                        identity: gnome_account.identity,
                        imap_host: gnome_account.imap_host,
                        imap_port: gnome_account.imap_port,
                        imap_use_ssl: gnome_account.imap_use_ssl,
                        smtp_host: gnome_account.smtp_host,
                        smtp_port: gnome_account.smtp_port,
                        smtp_use_tls: gnome_account.smtp_use_tls,
                    };
                    let account_id = backend.add_gnome_account(gnome_account_struct);
                    account_ids.push(account_id);
                }
                
                // Use the first account for demo messages
                let first_account_id = account_ids[0];
                backend.add_sample_messages(first_account_id);
                first_account_id
            }
            Ok(_) => {
                println!("No GNOME email accounts found, creating demo account");
                // Fallback to demo account
                let account = email_backend::EmailAccount::new(
                    "demo@example.com".to_string(),
                    "Demo User".to_string(),
                    "imap.gmail.com".to_string(),
                    993,
                    "smtp.gmail.com".to_string(),
                    587,
                    "demo@example.com".to_string(),
                    "password".to_string(),
                );
                let account_id = backend.add_account(account);
                backend.add_sample_messages(account_id);
                account_id
            }
            Err(e) => {
                println!("Error accessing GNOME accounts: {}, creating demo account", e);
                // Fallback to demo account
                let account = email_backend::EmailAccount::new(
                    "demo@example.com".to_string(),
                    "Demo User".to_string(),
                    "imap.gmail.com".to_string(),
                    993,
                    "smtp.gmail.com".to_string(),
                    587,
                    "demo@example.com".to_string(),
                    "password".to_string(),
                );
                let account_id = backend.add_account(account);
                backend.add_sample_messages(account_id);
                account_id
            }
        };
        
        let backend = Rc::new(RefCell::new(backend));
        // Use the same account_id that we used for adding sample messages

        let app_state = Rc::new(RefCell::new(AppState {
            backend: backend.clone(),
            current_account: Some(account_id),
            current_mailbox: ui_state.current_mailbox.clone(),
            current_thread_id: ui_state.current_thread_id.clone(),
            accordion_states: Rc::new(RefCell::new(ui_state.accordion_states.clone())),
        }));

        let window = libadwaita::ApplicationWindow::builder()
            .application(app)
            .title("Asgard Mail")
            .default_width(ui_state.window_width)
            .default_height(ui_state.window_height)
            .build();
        
        // Ensure opaque background
        window.add_css_class("background");
        
        // Create header bar
        let header_bar = libadwaita::HeaderBar::new();
        header_bar.set_show_end_title_buttons(false);
        header_bar.set_show_start_title_buttons(false);
        header_bar.add_css_class("flat");
        header_bar.add_css_class("headerbar");
        
        // Create custom window control buttons
        let close_button = gtk4::Button::from_icon_name("window-close-symbolic");
        close_button.set_tooltip_text(Some("Close"));
        close_button.add_css_class("window-controls");
        close_button.add_css_class("close");
        
        let minimize_button = gtk4::Button::from_icon_name("window-minimize-symbolic");
        minimize_button.set_tooltip_text(Some("Minimize"));
        minimize_button.add_css_class("window-controls");
        minimize_button.add_css_class("minimize");
        
        let maximize_button = gtk4::Button::from_icon_name("window-maximize-symbolic");
        maximize_button.set_tooltip_text(Some("Maximize"));
        maximize_button.add_css_class("window-controls");
        maximize_button.add_css_class("maximize");
        
        // Connect window control buttons
        let window_close = window.clone();
        close_button.connect_clicked(move |_| {
            // Hide window instead of closing
            window_close.hide();
        });
        
        let window_minimize = window.clone();
        minimize_button.connect_clicked(move |_| {
            window_minimize.minimize();
        });
        
        let window_maximize = window.clone();
        maximize_button.connect_clicked(move |_| {
            if window_maximize.is_maximized() {
                window_maximize.unmaximize();
            } else {
                window_maximize.maximize();
            }
        });

        // Create window controls container
        let window_controls = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        window_controls.add_css_class("window-controls-container");
        window_controls.append(&close_button);
        window_controls.append(&minimize_button);
        window_controls.append(&maximize_button);

        
        // Mailbox title (center) - will be updated dynamically
        let mailbox_title = gtk4::Label::new(Some("Inbox – 10 messages, 3 unread"));
        mailbox_title.add_css_class("title");
        mailbox_title.set_xalign(0.0); // Align to left
        
        // Compose button
        let compose_button = gtk4::Button::from_icon_name("mail-message-new-symbolic");
        compose_button.set_tooltip_text(Some("Compose"));
        compose_button.add_css_class("flat");

        // Action buttons (right side) in correct order
        let reply_button = gtk4::Button::from_icon_name("mail-reply-sender-symbolic");
        reply_button.set_tooltip_text(Some("Reply"));
        reply_button.add_css_class("flat");
        
        let reply_all_button = gtk4::Button::from_icon_name("mail-reply-all-symbolic");
        reply_all_button.set_tooltip_text(Some("Reply All"));
        reply_all_button.add_css_class("flat");
        
        let forward_button = gtk4::Button::from_icon_name("mail-forward-symbolic");
        forward_button.set_tooltip_text(Some("Forward"));
        forward_button.add_css_class("flat");
        
        let archive_button = gtk4::Button::from_icon_name("mail-archive-symbolic");
        archive_button.set_tooltip_text(Some("Archive"));
        archive_button.add_css_class("flat");
        
        let delete_button = gtk4::Button::from_icon_name("user-trash-symbolic");
        delete_button.set_tooltip_text(Some("Delete"));
        delete_button.add_css_class("flat");
        
        let spam_button = gtk4::Button::from_icon_name("mail-mark-junk-symbolic");
        spam_button.set_tooltip_text(Some("Mark as Spam"));
        spam_button.add_css_class("flat");
        
        let move_button = gtk4::MenuButton::builder()
            .icon_name("mail-move-symbolic")
            .build();
        move_button.set_tooltip_text(Some("Move to"));
        move_button.add_css_class("flat");
        
        // Flag menu with color options
        let flag_menu = gtk4::MenuButton::builder()
            .icon_name("mail-flag-symbolic")
            .has_frame(false)
            .build();
        flag_menu.add_css_class("flat");
        flag_menu.set_tooltip_text(Some("Flag"));

        let pop = gtk4::PopoverMenu::from_model(None::<&gtk4::gio::MenuModel>);
        let list = gtk4::ListBox::new();
        list.add_css_class("flag-list");

        for (name, color) in [
            ("None", "none"),
            ("Red", "red"),
            ("Orange", "orange"),
            ("Yellow", "yellow"),
            ("Green", "green"),
            ("Blue", "blue"),
            ("Purple", "purple"),
        ] {
            let row = gtk4::ListBoxRow::new();
            let h = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
            let swatch = gtk4::DrawingArea::new();
            swatch.add_css_class(&format!("flag-swatch-{}", color));
            swatch.set_size_request(12, 12);
            let lab = gtk4::Label::new(Some(name));
            h.append(&swatch);
            h.append(&lab);
            row.set_child(Some(&h));
            list.append(&row);

            // Connect flag selection (simplified for now)
            let color_clone = color.to_string();
            row.connect_activate(move |_| {
                println!("Selected flag color: {}", color_clone);
                // TODO: Implement flag color persistence
            });
        }
        pop.set_child(Some(&list));
        flag_menu.set_popover(Some(&pop));
        
        let search_button = gtk4::ToggleButton::new();
        search_button.set_icon_name("system-search-symbolic");
        search_button.set_tooltip_text(Some("Search"));
        search_button.add_css_class("flat");
        
        // Sidebar toggle button
        let sidebar_toggle = gtk4::ToggleButton::new();
        sidebar_toggle.set_icon_name("sidebar-show-symbolic");
        sidebar_toggle.set_tooltip_text(Some("Toggle Sidebar"));
        sidebar_toggle.add_css_class("flat");
        sidebar_toggle.set_active(ui_state.sidebar_visible); // Use loaded state
        
        // Create header container using Box layout instead of paned widgets
        let header_container = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        header_container.set_hexpand(true);
        
        // Left section: window controls only (aligns with left panel)
        let header_left = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        header_left.set_size_request(ui_state.sidebar_width - 6, -1); // Use loaded state
        header_left.append(&window_controls);

        // Application menu button
        let app_menu_button = gtk4::MenuButton::builder()
            .icon_name("open-menu-symbolic")
            .has_frame(false)
            .build();
        app_menu_button.set_tooltip_text(Some("Application Menu"));
        app_menu_button.add_css_class("flat");
        
        // Create application menu
        let app_menu = Menu::new();
        app_menu.append(Some("Exit"), Some("app.exit"));
        
        let app_popover = gtk4::PopoverMenu::from_model(Some(&app_menu));
        app_menu_button.set_popover(Some(&app_popover));
        
        // Add exit action to the application
        let exit_action = gio::SimpleAction::new("exit", None);
        exit_action.connect_activate(move |_, _| {
            println!("Exit requested - quitting application");
            std::process::exit(0);
        });
        app.add_action(&exit_action);

        // Quit button (prominent button to actually close the app)
        let quit_button = gtk4::Button::from_icon_name("application-exit-symbolic");
        quit_button.set_tooltip_text(Some("Quit Application"));
        quit_button.add_css_class("flat");
        quit_button.add_css_class("destructive-action");
        
        // Connect quit button to exit action
        let quit_action = gio::SimpleAction::new("quit", None);
        quit_action.connect_activate(move |_, _| {
            println!("Quit button clicked - terminating application");
            std::process::exit(0);
        });
        app.add_action(&quit_action);
        
        quit_button.connect_clicked(move |_| {
            println!("Quit button clicked - terminating application");
            std::process::exit(0);
        });

        // Middle section: sidebar toggle + title (aligns with center panel)
        let header_middle = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
        header_middle.set_size_request(ui_state.middle_width + 50, -1); // Use loaded state
        header_middle.append(&sidebar_toggle);
        header_middle.append(&app_menu_button);
        header_middle.append(&quit_button);
        header_middle.append(&mailbox_title);
        
        // Add sections to container
        header_container.append(&header_left);
        header_container.append(&header_middle);

        // Create button groups with spacers for proper Apple Mail layout
        let button_group0 = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        button_group0.append(&compose_button);

        let button_group1 = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        button_group1.append(&reply_button);
        button_group1.append(&reply_all_button);
        button_group1.append(&forward_button);
        
        let button_group2 = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        button_group2.append(&archive_button);
        button_group2.append(&delete_button);
        button_group2.append(&spam_button);
        
        let button_group3 = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        button_group3.append(&move_button);
        button_group3.append(&flag_menu);
        
        // Separate search button into its own group
        let button_group_search = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        button_group_search.append(&search_button);
        
        // Right section will be added to the main header container
        
        // Right section: buttons (aligns with right panel)
        // Create a container that will proportionally distribute buttons
        let header_right = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        header_right.set_margin_end(0);
        header_right.set_margin_start(0);
        
        // Create proportional spacing using expandable spacers
        // This ensures buttons are distributed proportionally across the available space

        let button_spacer0 = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        button_spacer0.set_hexpand(true);
        let button_spacer1 = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        button_spacer1.set_hexpand(true);

        // Add all button groups together - they will be tightly packed
        header_right.append(&button_group0);
        header_right.append(&button_spacer0);
        header_right.append(&button_group1);
        header_right.append(&button_group2);
        header_right.append(&button_group3);
        header_right.append(&button_spacer1);
        header_right.append(&button_group_search);

        // Add right section to the main header container
        header_container.append(&header_right);

        // Set the header container as the title widget
        header_bar.set_title_widget(Some(&header_container));
        header_bar.set_margin_end(0);
        header_bar.set_halign(Align::Fill);

        // message list now returns (container, list)
        let (messages_container, message_list) = build_message_list(&backend, &app_state, &ui_state.category_states);

        // reader shell + updater
        let reader = build_reader_shell();

        // Build UI components
        let sidebar = build_sidebar(&backend, &app_state, &mailbox_title, &message_list, &reader);

        // Initial thread render
        {
            let acc = app_state.borrow().current_account.unwrap();
            let msgs = backend.borrow().get_messages_for_mailbox(acc, "INBOX").iter().map(|m| (*m).clone()).collect();
            let threads = thread_helpers::group_into_threads(msgs);
            if let Some(id) = app_state.borrow().current_thread_id.clone() {
                if let Some(th) = threads.into_iter().find(|t| t.id == id) {
                    update_reader_for_thread(&reader, &th);
                }
            }
        }

        // Selection → update AppState + reader
        {
            let backend = backend.clone();
            let app_state = app_state.clone();
            let reader = reader.clone();
            message_list.connect_selected_rows_changed(move |list| {
                if let Some(row) = list.selected_row() {
                    // fetch id we stashed on the row
                    let id: String = unsafe { row.data::<String>("thread-id").unwrap().as_ref() }.clone();
                    app_state.borrow_mut().current_thread_id = Some(id.clone());

                    let acc = app_state.borrow().current_account.unwrap();
                    let current_mailbox = app_state.borrow().current_mailbox.as_ref().unwrap().clone();
                    
                    // Get messages from the current mailbox (handle unified views)
                    let mut all_messages = Vec::new();
                    if current_mailbox.starts_with("ALL_") || current_mailbox == "VIPS" {
                        // Handle unified views
                        let backend_guard = backend.borrow();
                        let accounts = backend_guard.get_accounts().clone();
                        // drop(backend_guard); // Release the borrow
                        
                        for account in accounts {
                            let mailbox_type = match current_mailbox.as_str() {
                                "ALL_INBOXES" => "INBOX",
                                "ALL_DRAFTS" => "DRAFTS", 
                                "ALL_SENT" => "SENT",
                                "VIPS" => "VIP",
                                "FLAGGED" => "FLAGGED",
                                "REMINDERS" => "REMINDERS",
                                _ => continue,
                            };
                            let backend_guard = backend.borrow();
                            let messages = backend_guard.get_messages_for_mailbox(account.id, mailbox_type);
                            let messages_clone: Vec<_> = messages.iter().map(|m| (*m).clone()).collect();
                            drop(backend_guard); // Release the borrow
                            all_messages.extend(messages_clone);
                        }
                    } else if current_mailbox.contains(':') {
                        // Handle account-specific mailboxes
                        let parts: Vec<&str> = current_mailbox.split(':').collect();
                        if parts.len() == 2 {
                            if let Ok(account_uuid) = uuid::Uuid::parse_str(parts[0]) {
                                let backend_guard = backend.borrow();
                                let messages = backend_guard.get_messages_for_mailbox(account_uuid, parts[1]);
                                let messages_clone: Vec<_> = messages.iter().map(|m| (*m).clone()).collect();
                                drop(backend_guard); // Release the borrow
                                all_messages.extend(messages_clone);
                            }
                        }
                    } else {
                        // Handle legacy single-account mailboxes
                        let backend_guard = backend.borrow();
                        let messages = backend_guard.get_messages_for_mailbox(acc, &current_mailbox);
                        let messages_clone: Vec<_> = messages.iter().map(|m| (*m).clone()).collect();
                        drop(backend_guard); // Release the borrow
                        all_messages.extend(messages_clone);
                    }
                    
                    let msgs = all_messages;
                    let threads = thread_helpers::group_into_threads(msgs);
                    if let Some(th) = threads.into_iter().find(|t| t.id == id) {
                        update_reader_for_thread(&reader, &th);
                        
                        // Mark the last message in the thread as read (the one being viewed)
                        if let Some(last_message) = th.messages.last() {
                            let _ = backend.borrow_mut().mark_message_as_read(acc, last_message.id);
                            // Update mailbox counts after marking as read
                            backend.borrow_mut().update_mailbox_counts_dynamically(acc);
                        }
                    }
                }
            });
        }

        // Create scrollable sidebar
        let sidebar_scroll = gtk4::ScrolledWindow::new();
        sidebar_scroll.set_child(Some(&sidebar));
        sidebar_scroll.set_policy(gtk4::PolicyType::Never, gtk4::PolicyType::Automatic); // Only vertical scroll
        sidebar_scroll.set_vexpand(true);
        sidebar_scroll.set_hexpand(false);

        // Paned A: left ↔ middle (user drags to set widths)
        let paned_a = gtk4::Paned::new(gtk4::Orientation::Horizontal);
        paned_a.set_shrink_start_child(false);  // Don't shrink left panel below minimum
        paned_a.set_shrink_end_child(true);     // Allow middle panel to shrink
        paned_a.set_resize_start_child(false);  // Don't allow left panel to resize (keep fixed)
        paned_a.set_resize_end_child(true);     // Allow middle panel to resize
        paned_a.set_start_child(Some(&sidebar_scroll));
        paned_a.set_end_child(Some(&messages_container));
        paned_a.set_position(SIDEBAR_DEFAULT); // initial sidebar width

        // Paned B: (left+middle) ↔ right (only right grows/shrinks on window resize)
        let lm_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        lm_box.append(&paned_a);

        // Set minimum widths
        const SIDEBAR_MIN: i32 = 100;  // Left panel minimum
        const SIDEBAR_DEFAULT: i32 = 260;
        const MID_MIN: i32 = 100;      // Middle panel minimum
        const MID_DEFAULT: i32 = 420;
        const RIGHT_MIN: i32 = 80;    // Right panel minimum
        const SYSTEM_BUTTONS_WIDTH: i32 = 95; // Fine-tuned width to account for close/minimize/maximize buttons + spacing

        sidebar_scroll.set_width_request(SIDEBAR_MIN);
        messages_container.set_width_request(MID_MIN);
        messages_container.set_hexpand(true); // Allow messages container to expand with middle panel
        reader.root.set_width_request(RIGHT_MIN);
        paned_a.set_position(ui_state.sidebar_width);
        
        // Initial positioning will be set after paned_b is created

        let paned_b = gtk4::Paned::new(gtk4::Orientation::Horizontal);
        paned_b.set_start_child(Some(&lm_box));
        paned_b.set_end_child(Some(&reader.root));
        // Configure paned_b for proper resizing behavior
        paned_b.set_shrink_start_child(true);   // Allow left+middle to shrink when window shrinks
        paned_b.set_shrink_end_child(false);    // Don't shrink right panel below minimum
        paned_b.set_resize_start_child(false);  // Don't allow left+middle to resize (handled by paned_a)
        paned_b.set_resize_end_child(false);    // Don't allow right panel to resize beyond minimum
        
        // Set initial position for paned_b
        paned_b.set_position(ui_state.sidebar_width + ui_state.middle_width);
        
        // Enforce minimum sizes and prevent window expansion
        {
            let window = window.clone();
            let paned_a = paned_a.clone();
            let paned_b_clone1 = paned_b.clone();
            
            // Connect to paned_a position changes to enforce minimum sizes
            paned_a.connect_notify_local(Some("position"), move |paned, _| {
                let position = paned.position();
                
                // Enforce minimum sidebar size
                if position < SIDEBAR_MIN {
                    paned.set_position(SIDEBAR_MIN);
                    return;
                }
                
                // Calculate middle panel size and enforce minimum
                let paned_b_position = paned_b_clone1.position();
                let middle_size = paned_b_position - position;
                
                if middle_size < MID_MIN {
                    let new_paned_b_position = position + MID_MIN;
                    paned_b_clone1.set_position(new_paned_b_position);
                }
            });
        }
        
        {
            let window = window.clone();
            let paned_b = paned_b.clone();
            
            // Connect to paned_b position changes to prevent window expansion
            paned_b.connect_notify_local(Some("position"), move |paned, _| {
                let position = paned.position();
                let current_width = window.default_width();
                let max_allowed_width = position + RIGHT_MIN;
                
                // If we're trying to expand beyond the constraint, reset the position
                if current_width < max_allowed_width {
                    // Block the resize by resetting to previous valid position
                    let valid_position = current_width - RIGHT_MIN;
                    paned.set_position(valid_position);
                }
            });
        }
        
        // Synchronize header section widths with main paned positions
        {
            let header_left = header_left.clone();
            paned_a.connect_notify_local(Some("position"), move |paned, _| {
                let position = paned.position();
                header_left.set_size_request(position - 6, -1); // Account for window controls offset
            });
        }

        {
            let header_middle = header_middle.clone();
            let paned_a_clone2 = paned_a.clone();
            paned_b.connect_notify_local(Some("position"), move |paned, _| {
                let position = paned.position();
                // Calculate middle section width: paned_b position - paned_a position
                let paned_a_position = paned_a_clone2.position();
                let middle_width = position - paned_a_position;
                header_middle.set_size_request(middle_width, -1);
            });
        }
        
        // Add periodic state saving for UI changes
        {
            let app_state = app_state.clone();
            let sidebar_toggle = sidebar_toggle.clone();
            let paned_a = paned_a.clone();
            let paned_b = paned_b.clone();
            let sidebar_scroll = sidebar_scroll.clone();
            let reader = reader.clone();
            
            // Save state every 5 seconds
            let timer = gtk4::glib::timeout_add_seconds_local(5, move || {
                let current_state = UIState {
                    window_width: 1400, // Default, will be updated on close
                    window_height: 900, // Default, will be updated on close
                    window_x: 100,
                    window_y: 100,
                    window_maximized: false, // Will be updated on close
                    sidebar_width: paned_a.position(),
                    middle_width: paned_b.position() - paned_a.position(),
                    right_width: 1400 - paned_b.position(), // Default width
                    sidebar_visible: sidebar_toggle.is_active(),
                    current_mailbox: app_state.borrow().current_mailbox.clone(),
                    current_thread_id: app_state.borrow().current_thread_id.clone(),
                    accordion_states: app_state.borrow().accordion_states.borrow().clone(),
                    sidebar_scroll_position: sidebar_scroll.vadjustment().value(),
                    message_list_scroll_position: 0.0, // TODO: Implement
                    reader_scroll_position: reader.scroller.vadjustment().value(),
                    category_states: HashMap::new(), // TODO: Implement
                };
                
                if let Err(e) = save_ui_state(&current_state) {
                    println!("Failed to save UI state: {}", e);
                }
                
                gtk4::glib::ControlFlow::Continue
            });
        }

        // Apply loaded sidebar visibility state
        sidebar_scroll.set_visible(ui_state.sidebar_visible);
        header_left.set_visible(ui_state.sidebar_visible);
        
        // Connect sidebar toggle button
        {
            let sidebar_scroll = sidebar_scroll.clone();
            let header_left = header_left.clone();
            sidebar_toggle.connect_toggled(move |btn| {
                let is_active = btn.is_active();
                if is_active {
                    // Show sidebar
                    sidebar_scroll.set_visible(true);
                    header_left.set_visible(true);
                } else {
                    // Hide sidebar
                    sidebar_scroll.set_visible(false);
                    header_left.set_visible(false);
                }
                // Keep the same icon but let the button's active state control its appearance
                btn.set_icon_name("sidebar-show-symbolic");
            });
        }

        // Create main container with header bar (AdwApplicationWindow doesn't support set_titlebar)
        let main_container = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        main_container.append(&header_bar);
        main_container.append(&paned_b);
        
        window.set_content(Some(&main_container));
        // Update the mailbox title with real counts
        update_mailbox_title(&mailbox_title, &backend, &app_state);
        
        // Set window position if not maximized
        if !ui_state.window_maximized {
            window.set_default_size(ui_state.window_width, ui_state.window_height);
            // Note: GTK doesn't support setting window position directly in libadwaita
            // The window manager will handle positioning
        } else {
            window.maximize();
        }
        
        // Add state saving on window close and integrate with system tray
        {
            let app_state = app_state.clone();
            let sidebar_toggle = sidebar_toggle.clone();
            let paned_a = paned_a.clone();
            let paned_b = paned_b.clone();
            let sidebar_scroll = sidebar_scroll.clone();
            let message_list = message_list.clone();
            let reader = reader.clone();
            
            window.connect_close_request(move |window| {
                // Check if this is a programmatic show (not user-initiated close)
                let is_programmatic_show = PROGRAMMATIC_SHOW.load(Ordering::Relaxed);
                println!("🚪 Close request handler called - programmatic_show: {}, visible: {}", is_programmatic_show, window.is_visible());
                
                if is_programmatic_show {
                    println!("✅ Ignoring close request during programmatic show");
                    gtk4::glib::Propagation::Proceed // Allow normal behavior
                } else if window.is_visible() {
                    // Collect current state
                    let mut current_state = UIState {
                        window_width: window.default_width(),
                        window_height: window.default_height(),
                        window_x: 100, // Default, can't get actual position
                        window_y: 100, // Default, can't get actual position
                        window_maximized: window.is_maximized(),
                        sidebar_width: paned_a.position(),
                        middle_width: paned_b.position() - paned_a.position(),
                        right_width: window.default_width() - paned_b.position(),
                        sidebar_visible: sidebar_toggle.is_active(),
                        current_mailbox: app_state.borrow().current_mailbox.clone(),
                        current_thread_id: app_state.borrow().current_thread_id.clone(),
                        accordion_states: app_state.borrow().accordion_states.borrow().clone(),
                        sidebar_scroll_position: sidebar_scroll.vadjustment().value(),
                        message_list_scroll_position: 0.0, // TODO: Get from message list scroller
                        reader_scroll_position: reader.scroller.vadjustment().value(),
                        category_states: HashMap::new(), // TODO: Implement category state tracking
                    };
                    
                    // Save state
                    if let Err(e) = save_ui_state(&current_state) {
                        println!("Failed to save UI state: {}", e);
                    }
                    
                    // Hide window instead of closing (system tray behavior)
                    println!("Hiding window instead of closing");
                    window.hide();
                    
                    // Clean up lock file when window is hidden (system tray behavior)
                    if let Err(e) = remove_lock_file() {
                        eprintln!("Failed to remove lock file: {}", e);
                    }
                    
                    gtk4::glib::Propagation::Stop // Prevent default close behavior
                } else {
                    // Window is already hidden, allow normal close behavior
                    println!("Window already hidden, allowing close");
                    
                    // Clean up lock file when app is actually closed
                    if let Err(e) = remove_lock_file() {
                        eprintln!("Failed to remove lock file: {}", e);
                    }
                    
                    gtk4::glib::Propagation::Proceed
                }
            });
        }
        
        // Create system tray icon
        create_system_tray(&window);
        
        println!("🪟 Window created and presenting...");
        window.present();
        println!("✅ Window presented successfully");
        
        // Restore scroll positions after the window is shown
        {
            let sidebar_scroll = sidebar_scroll.clone();
            let reader = reader.clone();
            
            // Use a timeout to ensure the window is fully rendered before restoring scroll positions
            let _timer = gtk4::glib::timeout_add_local(std::time::Duration::from_millis(100), move || {
                sidebar_scroll.vadjustment().set_value(ui_state.sidebar_scroll_position);
                reader.scroller.vadjustment().set_value(ui_state.reader_scroll_position);
                gtk4::glib::ControlFlow::Break
            });
        }
        
        // Set up DBus callback to show window and keep connection alive
        let dbus_service_callback = dbus_service_clone.clone();
        // Use a channel to communicate between threads instead of moving the window reference
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<DbusMethod>();
        
        // Spawn a thread to handle DBus callbacks
        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                // Set up the callback
                dbus_service_callback.set_method_callback(move |method| {
                    println!("DBus callback: method '{:?}' called", method);
                    let _ = tx.send(method);
                }).await;
                
                // Keep the DBus connection alive by waiting indefinitely
                // This prevents the DBus name from being released
                println!("DBus service is now active and waiting for requests...");
                loop {
                    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                }
            });
        });
        
        // Handle DBus messages on the main thread using a timeout-based approach
        let window_weak = window.downgrade();
        let mut rx_handle = Some(rx);
        glib::MainContext::default().spawn_local(async move {
            if let Some(mut rx) = rx_handle.take() {
                while let Some(method) = rx.recv().await {
                    match method {
                        DbusMethod::ShowWindow => {
                            println!("Handling show_window method");
                            if let Some(window) = window_weak.upgrade() {
                                window.present();
                                println!("Window presented successfully");
                            } else {
                                println!("Window reference is no longer valid");
                            }
                        }
                        DbusMethod::Ping => {
                            println!("Handling ping method");
                            // Ping is already handled by returning "pong"
                        }
                        DbusMethod::GetPid => {
                            println!("Handling get_pid method");
                            // PID is already handled by returning the PID value
                        }
                    }
                }
            }
        });
        
        // Show the window
        println!("🪟 Showing window...");
        window.present();
        println!("✅ Window presented successfully");
    });

    println!("🚀 Starting GTK main loop...");
    
    // Run GTK main loop directly (this will block until the app exits)
    println!("Running app...");
    let result = app.run();
    println!("🏁 GTK main loop exited with result: {:?}", result);
    
    // Unregister DBus service on exit
    if let Err(e) = std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(dbus_service.unregister_service())
    }).join().unwrap() {
        eprintln!("Failed to unregister DBus service: {}", e);
    }
    
    Ok(())
}

fn add_sidebar_section(list_box: &gtk4::ListBox, title: &str) {
    let header = gtk4::Label::new(Some(title));
    header.add_css_class("sidebar-section");
    header.set_xalign(0.0);
    header.set_margin_start(12);
    header.set_margin_end(12);
    header.set_margin_top(8);
    header.set_margin_bottom(4);
    
    let row = gtk4::ListBoxRow::new();
    row.set_child(Some(&header));
    row.set_selectable(false);
    row.set_activatable(false);
    list_box.append(&row);
}

fn add_sidebar_subsection(list_box: &gtk4::ListBox, title: &str) {
    let header = gtk4::Label::new(Some(title));
    header.add_css_class("sidebar-subsection");
    header.set_xalign(0.0);
    header.set_margin_start(24); // More indented than main sections
    header.set_margin_end(12);
    header.set_margin_top(4);
    header.set_margin_bottom(2);
    
    let row = gtk4::ListBoxRow::new();
    row.set_child(Some(&header));
    row.set_selectable(false);
    row.set_activatable(false);
    list_box.append(&row);
}

fn add_expandable_sidebar_item(
    list_box: &gtk4::ListBox,
    name: &str,
    icon_name: &str,
    is_selected: bool,
    mailbox_name: &str,
    backend: &Rc<RefCell<EmailBackend>>,
    app_state: &Rc<RefCell<AppState>>,
    mailbox_title: &gtk4::Label,
    message_list: &gtk4::ListBox,
    reader: &Reader,
) -> (gtk4::ListBoxRow, gtk4::Revealer, gtk4::ListBox) {
    let row = gtk4::ListBoxRow::new();
    let h = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
    h.set_margin_start(12);  // Same margin as regular items for icon alignment
    h.set_margin_end(12);
    h.set_margin_top(6);
    h.set_margin_bottom(6);
    
    // COMMENTED OUT ARROW CREATION COMPLETELY
    // let arrow = gtk4::Label::new(Some(">"));
    // arrow.add_css_class("expandable-arrow");
    // arrow.set_xalign(0.0);
    
    // Icon (aligned with other icons)
    let icon = gtk4::Image::from_icon_name(icon_name);
    icon.set_icon_size(gtk4::IconSize::Normal);
    icon.add_css_class("mailbox-icon");
    
    // Label
    let lab = gtk4::Label::new(Some(name));
    lab.set_xalign(0.0);
    lab.set_hexpand(true);
    
    // COMMENTED OUT ARROWS FOR NOW - START FROM SCRATCH
    // let arrow_container = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
    // arrow_container.set_size_request(16, -1); // Fixed width for arrow space
    // arrow_container.append(&arrow);
    
    // h.append(&arrow_container);
    h.append(&icon);
    h.append(&lab);
    
    row.set_child(Some(&h));
    row.add_css_class("mailbox-row");
    
    if is_selected {
        row.add_css_class("selected");
    }
    
    // Create revealer for child items
    let revealer = gtk4::Revealer::new();
    revealer.set_reveal_child(false);
    
    // Create a container for child items
    let child_container = gtk4::ListBox::new();
    child_container.add_css_class("sidebar");
    revealer.set_child(Some(&child_container));
    
    // COMMENTED OUT CLICK HANDLER - NO ARROWS
    // let gesture = gtk4::GestureClick::new();
    // let revealer_clone = revealer.clone();
    // let arrow_clone = arrow.clone();
    // gesture.connect_pressed(move |_, _, _, _| {
    //     let is_revealed = revealer_clone.reveals_child();
    //     revealer_clone.set_reveal_child(!is_revealed);
    //     arrow_clone.set_text(if !is_revealed { "v" } else { ">" });
    // });
    // row.add_controller(gesture);
    
    list_box.append(&row);
    list_box.append(&revealer);
    
    (row, revealer, child_container)
}

fn add_sidebar_item(
    list_box: &gtk4::ListBox, 
    name: &str, 
    icon_name: &str, 
    is_selected: bool,
    mailbox_name: &str,
    backend: &Rc<RefCell<EmailBackend>>,
    app_state: &Rc<RefCell<AppState>>,
    mailbox_title: &gtk4::Label,
    message_list: &gtk4::ListBox,
    reader: &Reader,
    expandable: bool,
    expanded: bool,
) -> Option<(gtk4::Revealer, gtk4::ListBox)> {
    let row = gtk4::ListBoxRow::new();
    row.set_activatable(true);
    
    let container = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
    container.set_margin_start(8);
    container.set_margin_end(12);
    container.set_margin_top(6);
    container.set_margin_bottom(6);
    
    // Expand symbol (if expandable)
    if expandable {
        let expand_symbol = gtk4::Label::new(Some(if expanded { "×" } else { "+" }));
        expand_symbol.add_css_class("expand-symbol");
        expand_symbol.set_xalign(0.0);
        container.append(&expand_symbol);
    } else {
        // Empty spacer to maintain alignment when not expandable
        let spacer = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        spacer.set_size_request(16, -1); // Same width as expand symbol
        container.append(&spacer);
    }
    
    // Icon (only if icon_name is not empty)
    if !icon_name.is_empty() {
        let icon = gtk4::Image::from_icon_name(icon_name);
        icon.set_icon_size(gtk4::IconSize::Normal);
        icon.add_css_class("mailbox-icon");
        container.append(&icon);
    }
    
    // Label
    let label = gtk4::Label::builder()
        .label(name)
        .xalign(0.0)
        .hexpand(true)
        .build();
    
    container.append(&label);
    
    // Get real unread count from backend
    let unread_count = if let Some(account_id) = app_state.borrow().current_account {
        if let Some(mailbox) = backend.borrow().get_mailboxes(account_id)
            .iter()
            .find(|m| m.name.to_uppercase() == mailbox_name.to_uppercase()) {
            mailbox.unread_count
        } else {
            0
        }
    } else {
        0
    };
    
    // Unread count badge
    if unread_count > 0 {
        let badge = gtk4::Label::builder()
            .label(&unread_count.to_string())
            .build();
        badge.add_css_class("count-badge");
        container.append(&badge);
    }
    
    row.set_child(Some(&container));
    list_box.append(&row);
    
    if is_selected {
        list_box.select_row(Some(&row));
    }
    
    // Handle expandable functionality
    let mut revealer = None;
    let mut child_container = None;
    
    if expandable {
        // Create revealer for child items
        let revealer_widget = gtk4::Revealer::new();
        revealer_widget.set_reveal_child(expanded);
        
        // Create a container for child items
        let child_list = gtk4::ListBox::new();
        child_list.add_css_class("sidebar");
        revealer_widget.set_child(Some(&child_list));
        
        // Add separate click handlers for expand symbol and label
        let expand_symbol = if expandable {
            // Find the expand symbol we just added
            container.first_child().unwrap().downcast::<gtk4::Label>().unwrap()
        } else {
            panic!("Expand symbol not found");
        };
        
        // Click handler for expand symbol - only toggles expansion, no focus change
        let expand_gesture = gtk4::GestureClick::new();
        let revealer_clone = revealer_widget.clone();
        let expand_symbol_clone = expand_symbol.clone();
        let app_state_clone = app_state.clone();
        let mailbox_name_for_state = mailbox_name.to_string();
        expand_gesture.connect_pressed(move |gesture, _, _, _| {
            // Prevent focus change by claiming the event
            gesture.set_state(gtk4::EventSequenceState::Claimed);
            
            let is_revealed = revealer_clone.reveals_child();
            revealer_clone.set_reveal_child(!is_revealed);
            expand_symbol_clone.set_text(if !is_revealed { "×" } else { "+" });
            
            // Save accordion state
            app_state_clone.borrow().accordion_states.borrow_mut().insert(mailbox_name_for_state.clone(), !is_revealed);
        });
        expand_symbol.add_controller(expand_gesture);
        
        // Click handler for the entire row - switches mailbox (except when clicking expand symbol)
        let row_gesture = gtk4::GestureClick::new();
        let backend_clone = backend.clone();
        let app_state_clone = app_state.clone();
        let mailbox_title_clone = mailbox_title.clone();
        let message_list_clone = message_list.clone();
        let reader_clone = reader.clone();
        let mailbox_name_clone = mailbox_name.to_string();
        let revealer_clone = revealer_widget.clone();
        let expand_symbol_clone = expand_symbol.clone();
        
        row_gesture.connect_pressed(move |gesture, n_press, _, _| {
            if n_press == 1 {
                // Single click - switch to this mailbox (for expandable groups like "All Inboxes")
                app_state_clone.borrow_mut().current_mailbox = Some(mailbox_name_clone.clone());
                update_mailbox_title(&mailbox_title_clone, &backend_clone, &app_state_clone);
                update_message_list_for_mailbox(&message_list_clone, &backend_clone, &app_state_clone, &reader_clone);
                backend_clone.borrow_mut().update_mailbox_counts_dynamically(app_state_clone.borrow().current_account.unwrap());
                update_mailbox_title(&mailbox_title_clone, &backend_clone, &app_state_clone);
                
                println!("Switched to mailbox: {}", mailbox_name_clone);
            } else if n_press == 2 {
                // Double click - toggle accordion expansion
                let is_revealed = revealer_clone.reveals_child();
                revealer_clone.set_reveal_child(!is_revealed);
                expand_symbol_clone.set_text(if !is_revealed { "×" } else { "+" });
                
                // Save accordion state
                app_state_clone.borrow().accordion_states.borrow_mut().insert(mailbox_name_clone.clone(), !is_revealed);
                
                println!("Toggled accordion for: {}", mailbox_name_clone);
            }
        });
        row.add_controller(row_gesture);
        
        // Add revealer to the list box
        list_box.append(&revealer_widget);
        
        revealer = Some(revealer_widget);
        child_container = Some(child_list);
    } else {
        // Add click handler to switch mailbox (only for non-expandable items)
        let backend = backend.clone();
        let app_state = app_state.clone();
        let mailbox_title = mailbox_title.clone();
        let message_list = message_list.clone();
        let reader = reader.clone();
        let mailbox_name = mailbox_name.to_string();
        
        // Use GestureClick for reliable click handling
        let gesture = gtk4::GestureClick::new();
        gesture.connect_pressed(move |_, _, _, _| {
            // Update current mailbox in app state
            app_state.borrow_mut().current_mailbox = Some(mailbox_name.clone());
            
            // Update the mailbox title
            update_mailbox_title(&mailbox_title, &backend, &app_state);
            
            // Update the message list to show messages from the new mailbox
            update_message_list_for_mailbox(&message_list, &backend, &app_state, &reader);
            
            // Update mailbox counts after switching
            backend.borrow_mut().update_mailbox_counts_dynamically(app_state.borrow().current_account.unwrap());
            
            // Refresh the mailbox title with updated counts
            update_mailbox_title(&mailbox_title, &backend, &app_state);
            
            println!("Switched to mailbox: {}", mailbox_name);
        });
        row.add_controller(gesture);
    }
    
    // Return revealer and child container if expandable
    if let (Some(r), Some(c)) = (revealer, child_container) {
        Some((r, c))
    } else {
        None
    }
}

fn create_message_row_three_tier(
    sender: &str,
    subject: &str,
    preview: &str,
    time: &str,
    is_unread: bool,
    is_selected: bool,
    has_attachments: bool,
    is_reply: bool,
) -> gtk4::ListBoxRow {
    let row = gtk4::ListBoxRow::new();
    row.set_activatable(true);
    
    // Main row container
    let row_container = gtk4::Box::new(gtk4::Orientation::Horizontal, 10);
    row_container.set_margin_start(12);
    row_container.set_margin_end(12);
    row_container.set_margin_top(6);
    row_container.set_margin_bottom(6);
    // row_container.add_css_class("message-row"); // Removed to prevent GTK styling
    
    // Left badges - clear and add only one indicator
    let badges = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
    badges.add_css_class("badge-box");
    badges.set_valign(gtk4::Align::Start); // Align badges to top
    
    // Only show reply icon, no unread indicator to avoid vertical lines
    if is_reply {
        let reply_icon = gtk4::Image::from_icon_name("mail-reply-sender-symbolic");
        reply_icon.set_icon_size(gtk4::IconSize::Normal);
        reply_icon.add_css_class("state-strong");
        badges.append(&reply_icon);
    } else if is_unread {
        let unread_dot = gtk4::Label::builder().label("•").build();
        unread_dot.add_css_class("unread-dot");
        badges.append(&unread_dot);
    }
    
    // Center content (three lines)
    let center = gtk4::Box::new(gtk4::Orientation::Vertical, 2);
    center.set_hexpand(true);
    
    // Tier 1: Sender (bold, 13px)
    let sender_label = gtk4::Label::builder()
        .label(sender)
        .xalign(0.0)
        .build();
    sender_label.add_css_class("row-sender");
    
    if is_unread {
        sender_label.add_css_class("semibold");
    }
    
    // Tier 2: Subject (regular 13px)
    let subject_label = gtk4::Label::builder()
        .label(subject)
        .xalign(0.0)
        .ellipsize(gtk4::pango::EllipsizeMode::End)
        .build();
    subject_label.add_css_class("row-subject");
    
    if is_unread {
        subject_label.add_css_class("semibold");
    }
    
    // Tier 3: Snippet (12px, exactly two lines, muted)
    let snippet_label = gtk4::Label::builder()
        .label(preview)
        .xalign(0.0)
        .wrap(true)
        .wrap_mode(gtk4::pango::WrapMode::WordChar)
        .lines(2)
        .ellipsize(gtk4::pango::EllipsizeMode::End)
        .build();
    snippet_label.add_css_class("row-snippet");
    
    center.append(&sender_label);
    center.append(&subject_label);
    center.append(&snippet_label);
    
    // Right meta (date + clip)
    let meta = gtk4::Box::new(gtk4::Orientation::Vertical, 2);
    
    // Date/Time (12px, muted, right-aligned)
    let date_label = gtk4::Label::builder()
        .label(time)
        .xalign(1.0)
        .build();
    date_label.add_css_class("row-date");
    
    // Attachment paperclip (under date)
    let attachment_icon = gtk4::Image::from_icon_name("mail-attachment-symbolic");
    attachment_icon.set_halign(gtk4::Align::End);
    attachment_icon.add_css_class("row-clip");
    if !has_attachments {
        attachment_icon.set_visible(false);
    }
    
    meta.append(&date_label);
    meta.append(&attachment_icon);
    
    // Assemble the row
    row_container.append(&badges);
    row_container.append(&center);
    row_container.append(&meta);
    
    row.set_child(Some(&row_container));
    
    if is_selected {
        row.add_css_class("selected");
    }
    
    row
}

fn create_thread_header_row(
    subject: &str,
    time: &str,
    count: usize,
    is_expanded: bool,
) -> gtk4::ListBoxRow {
    let row = gtk4::ListBoxRow::new();
    row.set_activatable(true);
    
    let container = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
    container.set_margin_start(12);
    container.set_margin_end(12);
    container.set_margin_top(6);
    container.set_margin_bottom(6);
    
    // Disclosure arrow
    let arrow_icon = if is_expanded { "pan-down-symbolic" } else { "pan-end-symbolic" };
    let arrow = gtk4::Image::from_icon_name(arrow_icon);
    arrow.set_icon_size(gtk4::IconSize::Normal);
    arrow.add_css_class("thread-arrow");
    
    // Subject
    let subject_label = gtk4::Label::builder()
        .label(subject)
        .xalign(0.0)
        .hexpand(true)
        .build();
    subject_label.add_css_class("thread-subject");
    
    // Thread count
    let count_label = gtk4::Label::builder()
        .label(&format!("{}", count))
        .build();
    count_label.add_css_class("thread-count");
    
    // Time
    let time_label = gtk4::Label::builder()
        .label(time)
        .build();
    time_label.add_css_class("thread-time");
    
    container.append(&arrow);
    container.append(&subject_label);
    container.append(&count_label);
    container.append(&time_label);
    
    row.set_child(Some(&container));
    row.add_css_class("thread-header");
    
    row
}

fn create_sub_message_row(
    sender: &str,
    preview: &str,
    time: &str,
    is_unread: bool,
    is_selected: bool,
) -> gtk4::ListBoxRow {
    let row = gtk4::ListBoxRow::new();
    row.set_activatable(true);
    
    let container = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
    container.set_margin_start(32); // Indented for sub-messages
    container.set_margin_end(12);
    container.set_margin_top(4);
    container.set_margin_bottom(4);
    
    // Reply/Forward arrow
    let arrow = gtk4::Image::from_icon_name("mail-reply-sender-symbolic");
    arrow.set_icon_size(gtk4::IconSize::Normal);
    arrow.add_css_class("reply-arrow");
    
    // Sender
    let sender_label = gtk4::Label::builder()
        .label(sender)
        .xalign(0.0)
        .build();
    sender_label.add_css_class("sub-sender");
    
    // Remove the unread CSS class that might be causing vertical lines
    // if is_unread {
    //     sender_label.add_css_class("unread");
    // }
    
    // Preview
    let preview_label = gtk4::Label::builder()
        .label(preview)
        .xalign(0.0)
        .hexpand(true)
        .ellipsize(gtk4::pango::EllipsizeMode::End)
        .build();
    preview_label.add_css_class("sub-preview");
    
    // Time
    let time_label = gtk4::Label::builder()
        .label(time)
        .build();
    time_label.add_css_class("sub-time");
    
    container.append(&arrow);
    container.append(&sender_label);
    container.append(&preview_label);
    container.append(&time_label);
    
    row.set_child(Some(&container));
    row.add_css_class("sub-message");
    
    if is_selected {
        row.add_css_class("selected");
    }
    
    row
}

fn build_sidebar(
    backend: &Rc<RefCell<EmailBackend>>,
    app_state: &Rc<RefCell<AppState>>,
    mailbox_title: &gtk4::Label,
    message_list: &gtk4::ListBox,
    reader: &Reader,
) -> gtk4::ListBox {
    let sidebar = gtk4::ListBox::new();
    sidebar.set_selection_mode(gtk4::SelectionMode::Single);
    sidebar.add_css_class("sidebar");

    // FAVORITES section
    add_sidebar_section(&sidebar, "FAVORITES");
    
    // Get accounts once
    let backend_guard = backend.borrow();
    let accounts = backend_guard.get_accounts();
    
    // Get accordion states from AppState
    let app_state_guard = app_state.borrow();
    let accordion_states_guard = app_state_guard.accordion_states.borrow();
    
    // All Inboxes - EXPANDABLE
    let all_inboxes_expanded = accordion_states_guard.get("ALL_INBOXES").copied().unwrap_or(false);
    if let Some((revealer, child_container)) = add_sidebar_item(&sidebar, "All Inboxes", "mail-inbox-symbolic", false, "ALL_INBOXES", backend, app_state, mailbox_title, message_list, reader, true, all_inboxes_expanded) {
        // Add account children for All Inboxes
        for account in &accounts {
            add_sidebar_item(&child_container, &account.display_name, "", false, &format!("{}:INBOX", account.id), backend, app_state, mailbox_title, message_list, reader, false, false);
        }
    }
    let vips_expanded = accordion_states_guard.get("VIPS").copied().unwrap_or(false);
    if let Some((revealer, child_container)) = add_sidebar_item(&sidebar, "VIPs", "emblem-favorite-symbolic", false, "VIPS", backend, app_state, mailbox_title, message_list, reader, true, vips_expanded) {
        // Add account children for All Inboxes
        for account in &accounts {
            add_sidebar_item(&child_container, &account.display_name, "", false, &format!("{}:VIP", account.id), backend, app_state, mailbox_title, message_list, reader, false, false);
        }
    }
    
    add_sidebar_item(&sidebar, "Flagged", "flag-symbolic", false, "FLAGGED", backend, app_state, mailbox_title, message_list, reader, false, false);
    add_sidebar_item(&sidebar, "Reminders", "alarm-symbolic", false, "REMINDERS", backend, app_state, mailbox_title, message_list, reader, false, false);
    
    // All Drafts - EXPANDABLE
    let all_drafts_expanded = accordion_states_guard.get("ALL_DRAFTS").copied().unwrap_or(false);
    if let Some((revealer2, child_container2)) = add_sidebar_item(&sidebar, "All Drafts", "mail-drafts-symbolic", false, "ALL_DRAFTS", backend, app_state, mailbox_title, message_list, reader, true, all_drafts_expanded) {
        // Add account children for All Drafts
        for account in &accounts {
            add_sidebar_item(&child_container2, &account.display_name, "", false, &format!("{}:DRAFTS", account.id), backend, app_state, mailbox_title, message_list, reader, false, false);
        }
    }
    
    // All Sent - EXPANDABLE
    let all_sent_expanded = accordion_states_guard.get("ALL_SENT").copied().unwrap_or(false);
    if let Some((revealer3, child_container3)) = add_sidebar_item(&sidebar, "All Sent", "mail-sent-symbolic", false, "ALL_SENT", backend, app_state, mailbox_title, message_list, reader, true, all_sent_expanded) {
        // Add account children for All Sent
        for account in &accounts {
            add_sidebar_item(&child_container3, &account.display_name, "", false, &format!("{}:SENT", account.id), backend, app_state, mailbox_title, message_list, reader, false, false);
        }
    }
    
    // Individual account sections
    for account in &accounts {
        add_sidebar_section(&sidebar, &account.display_name);
        
        // Standard mailboxes for each account
        add_sidebar_item(&sidebar, "Inbox", "mail-inbox-symbolic", false, &format!("{}:INBOX", account.id), backend, app_state, mailbox_title, message_list, reader, false, false);
        
        // TODO: Add custom folders for each account here
        // For now, we'll add the standard mailboxes
        add_sidebar_item(&sidebar, "Drafts", "mail-drafts-symbolic", false, &format!("{}:DRAFTS", account.id), backend, app_state, mailbox_title, message_list, reader, false, false);
        add_sidebar_item(&sidebar, "Sent", "mail-sent-symbolic", false, &format!("{}:SENT", account.id), backend, app_state, mailbox_title, message_list, reader, false, false);
        add_sidebar_item(&sidebar, "Junk", "mail-junk-symbolic", false, &format!("{}:JUNK", account.id), backend, app_state, mailbox_title, message_list, reader, false, false);
        add_sidebar_item(&sidebar, "Trash", "user-trash-symbolic", false, &format!("{}:TRASH", account.id), backend, app_state, mailbox_title, message_list, reader, false, false);
        add_sidebar_item(&sidebar, "Archive", "mail-archive-symbolic", false, &format!("{}:ARCHIVE", account.id), backend, app_state, mailbox_title, message_list, reader, false, false);
    }
    
    // Select the saved mailbox if it exists
    if let Some(saved_mailbox) = &app_state.borrow().current_mailbox {
        // Find and select the corresponding row
        let mut row_index = 0;
        let mut found = false;
        
        while let Some(row) = sidebar.row_at_index(row_index) {
            // Check if this row corresponds to the saved mailbox
            // This is a simplified approach - in a real implementation, you'd store mailbox names on rows
            if row_index > 0 { // Skip section headers
                row.add_css_class("selected");
                found = true;
                break;
            }
            row_index += 1;
        }
        
        if found {
            println!("Restored selected mailbox: {}", saved_mailbox);
        }
    }
    
    sidebar
}

fn build_message_list(backend: &Rc<RefCell<EmailBackend>>, app_state: &Rc<RefCell<AppState>>, category_states: &HashMap<String, bool>) -> (gtk4::Box, gtk4::ListBox) {
    let container = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    
    // Category chips (Gmail-style)
    let categories_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 4);
    categories_box.set_margin_start(12);
    categories_box.set_margin_end(12);
    categories_box.set_margin_top(8);
    categories_box.set_margin_bottom(4);
    categories_box.add_css_class("category-chips");
    
    let categories = ["Primary", "Social", "Promotions", "Updates", "Forums", "Important"];
    let mut first_button = None;
    
    for (i, category) in categories.iter().enumerate() {
        let button = gtk4::ToggleButton::with_label(category);
        button.add_css_class("pill");
        button.set_tooltip_text(Some(&format!("Show {} messages", category)));
        
        // Use loaded state or default to first category active
        let is_active = category_states.get(*category).copied().unwrap_or(i == 0);
        button.set_active(is_active);
        
        if i == 0 || is_active {
            if first_button.is_none() {
                first_button = Some(button.clone());
            }
        } else if let Some(ref first) = first_button {
            button.set_group(Some(first));
        }
        
        categories_box.append(&button);
    }
    
    // Message list
    let list = gtk4::ListBox::new();
    list.set_selection_mode(gtk4::SelectionMode::Single);
    list.add_css_class("message-list");
    
    container.append(&categories_box);
    container.append(&list);

    // === NEW: make threads ===
    let mut all_messages = Vec::new();
    let backend_guard = backend.borrow();
    let accounts = backend_guard.get_accounts();
    for account in accounts {
        let messages = backend_guard.get_messages_for_mailbox(account.id, "INBOX");
        all_messages.extend(messages.iter().map(|m| (*m).clone()));
    }
    let threads = thread_helpers::group_into_threads(all_messages);

    // Render each thread as ONE row using the 3-tier layout
    for (i, th) in threads.iter().enumerate() {
        // Find the appropriate "last" message based on current mailbox
        let last = find_last_message_for_mailbox(th, "ALL_INBOXES");
        let preview = truncate_preview(&last.body_text, 2); // Show only first 2 lines
        let row = create_message_row_three_tier(
            &last.from,                 // sender = last message's sender
            &th.subject,                // subject = thread subject
            &preview,                   // preview truncated to 2 lines
            &format_time(&last.date),   // date from last
            th.any_unread(),            // unread if any message unread
            i == 0,                     // select first thread initially
            th.has_attachments(),
            th.last_is_outgoing_reply(),
        );
        // stash the thread id for selection handling
        unsafe { row.set_data("thread-id", th.id.clone()); }
        list.append(&row);
    }

    // Select the saved thread or first thread if none saved
    let saved_thread_id = app_state.borrow().current_thread_id.clone();
    let mut selected_row = None;
    
    if let Some(saved_id) = saved_thread_id {
        // Try to find the saved thread
        let mut row_index = 0;
        while let Some(row) = list.row_at_index(row_index) {
            if let Some(thread_id_ptr) = unsafe { row.data::<String>("thread-id") } {
                let thread_id: String = unsafe { thread_id_ptr.as_ref() }.clone();
                if thread_id == saved_id {
                    selected_row = Some(row);
                    break;
                }
            }
            row_index += 1;
        }
    }
    
    // If no saved thread found or no saved thread, select the first one
    if selected_row.is_none() {
        selected_row = list.row_at_index(0);
    }
    
    if let Some(row) = selected_row {
        let id: String = unsafe { row.data::<String>("thread-id").unwrap().as_ref() }.clone();
        app_state.borrow_mut().current_thread_id = Some(id);
        list.select_row(Some(&row));
    }

    (container, list)
}

fn build_reader_shell() -> Reader {
    let root = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    root.add_css_class("message-view");
    root.set_hexpand(true);

    let meta_bar = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
    meta_bar.add_css_class("meta-bar");

    let count_label = gtk4::Label::new(Some("0 messages"));
    count_label.add_css_class("meta-count");

    let spacer = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
    spacer.set_hexpand(true);

    let summarize = gtk4::Button::new();
    summarize.add_css_class("flat");
    summarize.set_tooltip_text(Some("Summarize thread"));
    if gtk4::IconTheme::for_display(&gtk4::gdk::Display::default().unwrap()).has_icon("ai-summarize-symbolic") {
        summarize.set_icon_name("ai-summarize-symbolic");
    } else { 
        summarize.set_icon_name("tools-check-spelling"); 
    }

    meta_bar.append(&count_label);
    meta_bar.append(&spacer);
    meta_bar.append(&summarize);

    let scroller = gtk4::ScrolledWindow::new();
    scroller.set_policy(gtk4::PolicyType::Automatic, gtk4::PolicyType::Automatic);
    scroller.set_hexpand(true);
    scroller.set_vexpand(true);

    let cards = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    scroller.set_child(Some(&cards));

    root.append(&meta_bar);
    root.append(&scroller);

    Reader { root, meta_count: count_label, scroller, cards }
}

fn update_reader_for_thread(reader: &Reader, thread: &Thread) {
    // Counter = total in thread
    reader.meta_count.set_text(&pluralize_messages(thread.count()));

    // Clear current cards
    while let Some(child) = reader.cards.first_child() {
        reader.cards.remove(&child);
    }

    // Oldest → newest; unread expanded; last always expanded
    let last_idx = thread.messages.len().saturating_sub(1);
    let mut first_focus: Option<gtk4::Widget> = None;

    for (i, m) in thread.messages.iter().enumerate() {
        // Expand unread messages, the last message, and the first message
        let expanded = !m.is_read || i == last_idx || i == 0;
        let to = vec!["you@example.com".to_string(), "team@example.com".to_string()];
        let cc = vec!["cc@example.com".to_string()];
        let bcc = vec!["bcc@example.com".to_string()];
        
        let card = crate::widgets::message_card::MessageCard::new_collapsible(
            &m.from, &m.subject, &to, &cc, &bcc, &m.date, &m.body_text, m.has_attachments, expanded,
        );
        if first_focus.is_none() && !m.is_read { 
            first_focus = Some(card.widget().clone()); 
        }
        reader.cards.append(card.widget());
    }
    if first_focus.is_none() {
        if let Some(last) = reader.cards.last_child() { 
            first_focus = Some(last); 
        }
    }
    if let Some(target) = first_focus {
        target.grab_focus();
        let sc = reader.scroller.clone();
        target.connect_map(move |w| {
            let adj = sc.vadjustment();
            let y = w.allocation().y() as f64;
            let upper = adj.upper() - adj.page_size();
            adj.set_value(y.clamp(0.0, upper.max(0.0)));
        });
    }
}

fn pluralize_messages(n: usize) -> String {
    if n == 1 { "1 message".into() } else { format!("{} messages", n) }
}

fn format_time(date: &OffsetDateTime) -> String {
    let now = OffsetDateTime::now_utc();
    let duration = now - *date;

    if duration.as_seconds_f64() < 60.0 {
        "now".to_string()
    } else if duration.as_seconds_f64() < 3600.0 {
        format!("{}m", (duration.as_seconds_f64() / 60.0) as u32)
    } else if duration.as_seconds_f64() < 86400.0 {
        format!("{}h", (duration.as_seconds_f64() / 3600.0) as u32)
    } else if duration.as_seconds_f64() < 604800.0 {
        format!("{}d", (duration.as_seconds_f64() / 86400.0) as u32)
    } else {
        match date.month() {
            time::Month::January => format!("Jan {}", date.day()),
            time::Month::February => format!("Feb {}", date.day()),
            time::Month::March => format!("Mar {}", date.day()),
            time::Month::April => format!("Apr {}", date.day()),
            time::Month::May => format!("May {}", date.day()),
            time::Month::June => format!("Jun {}", date.day()),
            time::Month::July => format!("Jul {}", date.day()),
            time::Month::August => format!("Aug {}", date.day()),
            time::Month::September => format!("Sep {}", date.day()),
            time::Month::October => format!("Oct {}", date.day()),
            time::Month::November => format!("Nov {}", date.day()),
            time::Month::December => format!("Dec {}", date.day()),
        }
    }
}

fn truncate_preview(text: &str, max_lines: usize) -> String {
    let lines: Vec<&str> = text.lines().collect();
    if lines.len() <= max_lines {
        text.to_string()
    } else {
        let truncated = lines[..max_lines].join("\n");
        format!("{}...", truncated)
    }
}

fn find_last_message_for_mailbox<'a>(thread: &'a thread_helpers::Thread, mailbox: &str) -> &'a email_backend::EmailMessage {
    use thread_helpers::is_outgoing_message;
    
    match mailbox.to_uppercase().as_str() {
        "INBOX" => {
            // For Inbox, find the last incoming message
            thread.messages.iter()
                .rev() // Start from the most recent
                .find(|m| !is_outgoing_message(m))
                .unwrap_or_else(|| thread.last().unwrap())
        },
        "SENT" => {
            // For Sent, find the last outgoing message
            thread.messages.iter()
                .rev() // Start from the most recent
                .find(|m| is_outgoing_message(m))
                .unwrap_or_else(|| thread.last().unwrap())
        },
        _ => {
            // For other mailboxes, just return the last message
            thread.last().unwrap()
        }
    }
}

fn update_mailbox_title(
    title_label: &gtk4::Label,
    backend: &Rc<RefCell<EmailBackend>>,
    app_state: &Rc<RefCell<AppState>>,
) {
    let backend_guard = backend.borrow();
    let app_state_guard = app_state.borrow();
    
    if let (Some(account_id), Some(mailbox_name)) = (&app_state_guard.current_account, &app_state_guard.current_mailbox) {
        // Handle unified views (All Inboxes, All Drafts, etc.)
        if mailbox_name.starts_with("ALL_") || mailbox_name == "VIPS" {
            let (display_name, mailbox_type) = match mailbox_name.as_str() {
                "ALL_INBOXES" => ("All Inboxes", "INBOX"),
                "ALL_DRAFTS" => ("All Drafts", "DRAFTS"),
                "ALL_SENT" => ("All Sent", "SENT"),
                "VIPS" => ("VIPs", "VIP"),
                "FLAGGED" => ("Flagged", "FLAGGED"),
                "REMINDERS" => ("Reminders", "REMINDERS"),
                _ => ("Unknown", "INBOX"),
            };
            
            // Calculate total counts across all accounts
            let accounts = backend_guard.get_accounts();
            let mut total_messages = 0;
            let mut total_unread = 0;
            
            for account in accounts {
                let messages = backend_guard.get_messages_for_mailbox(account.id, mailbox_type);
                total_messages += messages.len();
                total_unread += messages.iter().filter(|m| !m.is_read).count();
            }
            
            let title = if total_unread > 0 {
                format!("<b>{}</b>\n<span color='gray'>Primary - {} messages - {} unread</span>", display_name, total_messages, total_unread)
            } else {
                format!("<b>{}</b>\n<span color='gray'>Primary - {} messages</span>", display_name, total_messages)
            };
            
            title_label.set_markup(&title);
        } 
        // Handle account-specific mailboxes (UUID:MAILBOX format)
        else if mailbox_name.contains(':') {
            let parts: Vec<&str> = mailbox_name.split(':').collect();
            if parts.len() == 2 {
                if let Ok(account_uuid) = uuid::Uuid::parse_str(parts[0]) {
                    // Find the account name
                    let account_name = backend_guard.get_accounts()
                        .iter()
                        .find(|a| a.id == account_uuid)
                        .map(|a| a.display_name.as_str())
                        .unwrap_or("Unknown Account");
                    
                    let mailbox_type = parts[1];
                    let mailbox_display = match mailbox_type.to_uppercase().as_str() {
                        "INBOX" => "Inbox",
                        "SENT" => "Sent",
                        "DRAFTS" => "Drafts",
                        "JUNK" => "Junk",
                        "TRASH" => "Trash",
                        "ARCHIVE" => "Archive",
                        _ => mailbox_type,
                    };
                    
                    let messages = backend_guard.get_messages_for_mailbox(account_uuid, mailbox_type);
                    let total_messages = messages.len();
                    let total_unread = messages.iter().filter(|m| !m.is_read).count();
                    
                    let title = if total_unread > 0 {
                        format!("<b>{} – {}</b>\n<span color='gray'>Primary - {} messages - {} unread</span>", mailbox_display, account_name, total_messages, total_unread)
                    } else {
                        format!("<b>{} – {}</b>\n<span color='gray'>Primary - {} messages</span>", mailbox_display, account_name, total_messages)
                    };
                    
                    title_label.set_markup(&title);
                } else {
                    title_label.set_text("Invalid account ID");
                }
            } else {
                title_label.set_text("Invalid mailbox format");
            }
        }
        // Handle legacy single-account mailboxes
        else {
            // Get the mailbox to access its counts
            if let Some(mailbox) = backend_guard.get_mailboxes(*account_id)
                .iter()
                .find(|m| m.name.to_uppercase() == mailbox_name.to_uppercase()) {
                
                let display_name = match mailbox_name.to_uppercase().as_str() {
                    "INBOX" => "Inbox",
                    "SENT" => "Sent",
                    "DRAFTS" => "Drafts",
                    "JUNK" => "Junk",
                    "TRASH" => "Trash",
                    "ARCHIVE" => "Archive",
                    _ => &mailbox.display_name,
                };
                
                let title = if mailbox.unread_count > 0 {
                    format!("<b>{}</b>\n<span color='gray'>Primary - {} messages - {} unread</span>", display_name, mailbox.message_count, mailbox.unread_count)
                } else {
                    format!("<b>{}</b>\n<span color='gray'>Primary - {} messages</span>", display_name, mailbox.message_count)
                };
                
                title_label.set_markup(&title);
            } else {
                // Fallback if mailbox not found
                title_label.set_text(&format!("{} – 0 messages", mailbox_name));
            }
        }
    } else {
        // Fallback if no current mailbox
        title_label.set_text("No mailbox selected");
    }
}

fn update_message_list_for_mailbox(
    message_list: &gtk4::ListBox,
    backend: &Rc<RefCell<EmailBackend>>,
    app_state: &Rc<RefCell<AppState>>,
    reader: &Reader,
) {
    let backend_guard = backend.borrow();
    let app_state_guard = app_state.borrow();
    
    if let Some(mailbox_name) = &app_state_guard.current_mailbox {
        let mut all_messages = Vec::new();
        
        // Handle unified views (All Inboxes, All Drafts, VIPS, etc.)
        if mailbox_name.starts_with("ALL_") || mailbox_name == "VIPS" {
            let accounts = backend_guard.get_accounts();
            for account in accounts {
                let mailbox_type = match mailbox_name.as_str() {
                    "ALL_INBOXES" => "INBOX",
                    "ALL_DRAFTS" => "DRAFTS", 
                    "ALL_SENT" => "SENT",
                    "VIPS" => "VIP",
                    "FLAGGED" => "FLAGGED",
                    "REMINDERS" => "REMINDERS",
                    _ => continue,
                };
                
                let messages = backend_guard.get_messages_for_mailbox(account.id, mailbox_type);
                all_messages.extend(messages.iter().map(|m| (*m).clone()));
            }
        } else if mailbox_name.contains(':') {
            // Handle account-specific mailboxes (account_id:mailbox_name)
            let parts: Vec<&str> = mailbox_name.split(':').collect();
            if parts.len() == 2 {
                if let Ok(account_uuid) = uuid::Uuid::parse_str(parts[0]) {
                    let messages = backend_guard.get_messages_for_mailbox(account_uuid, parts[1]);
                    all_messages.extend(messages.iter().map(|m| (*m).clone()));
                }
            }
        } else if let Some(account_id) = app_state_guard.current_account {
            // Handle legacy single-account mailboxes
            let messages = backend_guard.get_messages_for_mailbox(account_id, mailbox_name);
            all_messages.extend(messages.iter().map(|m| (*m).clone()));
        }
        
        // Sort messages by date (newest first)
        all_messages.sort_by(|a, b| b.date.cmp(&a.date));
        
        // Clear the current message list
        while let Some(child) = message_list.first_child() {
            message_list.remove(&child);
        }
        
        // Group messages into threads
        let threads = thread_helpers::group_into_threads(all_messages);
        
        // Add threads to the message list
        for thread in threads {
            let last = find_last_message_for_mailbox(&thread, mailbox_name);
            let preview = truncate_preview(&last.body_text, 2);
            let time = format_time(&last.date);
            
            // Determine if this is an outgoing reply
            let last_is_outgoing_reply = thread.messages.last()
                .map(|m| thread_helpers::is_outgoing_message(m) && thread_helpers::is_reply_message(m))
                .unwrap_or(false);
            
            let row = create_message_row_three_tier(
                &last.from,
                &last.subject,
                &preview,
                &time,
                thread.any_unread,
                last.has_attachments,
                last_is_outgoing_reply,
                false, // is_selected
            );
            
            // Store thread ID on the row for selection handling
            unsafe {
                row.set_data("thread-id", thread.id.clone());
            }
            message_list.append(&row);
        }
        
        // Clear the reader since we switched mailboxes
        reader.clear();
    }
    
}