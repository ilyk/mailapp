//! Main window for Asgard Mail

use crate::notifications::NotificationManager;
use crate::widgets::{MailboxTree, MessageList, MessageView, SearchBar, StatusBar};
use asgard_core::error::AsgardResult;
use asgard_core::config::Config;
use asgard_core::storage::StorageManager;
// use asgard_core::search::TantivySearchIndex;
use asgard_core::sync::SyncManager;
// use asgard_oauth::TokenManager;
use gtk4::prelude::*;
use gtk4::{ApplicationWindow, Box as GtkBox, Orientation, Paned, Label, Button, HeaderBar, Align};
// use libadwaita::prelude::*;
use std::sync::Arc;
use tokio::sync::Mutex;
// use std::rc::Rc;
// use std::cell::RefCell;

/// Main application window
pub struct MainWindow {
    /// GTK window
    window: ApplicationWindow,
    /// Main content box
    content_box: GtkBox,
    /// Header bar
    header_bar: HeaderBar,
    /// Left pane (mailbox list)
    left_pane: GtkBox,
    /// Middle pane (message list)
    middle_pane: GtkBox,
    /// Right pane (message view)
    right_pane: GtkBox,
    /// Main paned widget
    main_paned: Paned,
    /// Secondary paned widget
    secondary_paned: Paned,
    /// Mailbox tree widget
    mailbox_tree: MailboxTree,
    /// Message list widget
    message_list: MessageList,
    /// Message view widget
    message_view: MessageView,
    /// Search bar widget
    search_bar: SearchBar,
    /// Status bar widget
    status_bar: StatusBar,
    /// Quit callback
    quit_callback: Arc<Mutex<Option<Box<dyn Fn() + Send + Sync>>>>,
}

impl MainWindow {
    /// Create a new main window
    pub async fn new(
        config: Config,
        storage: Arc<Mutex<StorageManager>>,
        // search_index: Arc<Mutex<TantivySearchIndex>>,
        _sync_manager: Arc<Mutex<SyncManager>>,
        // token_manager: TokenManager,
        _notification_manager: NotificationManager,
        _demo_mode: bool,
    ) -> AsgardResult<Self> {
        // Create application window
        let window = ApplicationWindow::builder()
            .title("Asgard Mail")
            .default_width(config.ui.window_geometry.width)
            .default_height(config.ui.window_geometry.height)
            .build();

        // Ensure opaque background
        window.add_css_class("background");

        // Create header bar
        let header_bar = HeaderBar::new();
        header_bar.set_show_title_buttons(false);
        header_bar.add_css_class("flat");
        header_bar.add_css_class("headerbar");

        // Create custom window control buttons
        let close_button = Button::from_icon_name("window-close-symbolic");
        close_button.set_tooltip_text(Some("Close"));
        close_button.add_css_class("window-controls");
        close_button.add_css_class("close");
        
        let minimize_button = Button::from_icon_name("window-minimize-symbolic");
        minimize_button.set_tooltip_text(Some("Minimize"));
        minimize_button.add_css_class("window-controls");
        minimize_button.add_css_class("minimize");
        
        let maximize_button = Button::from_icon_name("window-maximize-symbolic");
        maximize_button.set_tooltip_text(Some("Maximize"));
        maximize_button.add_css_class("window-controls");
        maximize_button.add_css_class("maximize");
        
        // Connect window control buttons
        let window_close = window.clone();
        close_button.connect_clicked(move |_| {
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
        let window_controls = GtkBox::new(Orientation::Horizontal, 0);
        window_controls.add_css_class("window-controls-container");
        window_controls.append(&close_button);
        window_controls.append(&minimize_button);
        window_controls.append(&maximize_button);

        // Mailbox title (center) - will be updated dynamically
        let mailbox_title = Label::new(Some("Inbox – 0 messages"));
        mailbox_title.add_css_class("title");
        mailbox_title.set_xalign(0.0);
        
        // Compose button
        let compose_button = Button::from_icon_name("mail-message-new-symbolic");
        compose_button.set_tooltip_text(Some("Compose"));
        compose_button.add_css_class("flat");

        // Action buttons (right side)
        let reply_button = Button::from_icon_name("mail-reply-sender-symbolic");
        reply_button.set_tooltip_text(Some("Reply"));
        reply_button.add_css_class("flat");
        
        let reply_all_button = Button::from_icon_name("mail-reply-all-symbolic");
        reply_all_button.set_tooltip_text(Some("Reply All"));
        reply_all_button.add_css_class("flat");
        
        let forward_button = Button::from_icon_name("mail-forward-symbolic");
        forward_button.set_tooltip_text(Some("Forward"));
        forward_button.add_css_class("flat");
        
        let archive_button = Button::from_icon_name("mail-archive-symbolic");
        archive_button.set_tooltip_text(Some("Archive"));
        archive_button.add_css_class("flat");
        
        let delete_button = Button::from_icon_name("user-trash-symbolic");
        delete_button.set_tooltip_text(Some("Delete"));
        delete_button.add_css_class("flat");
        
        let search_button = Button::from_icon_name("system-search-symbolic");
        search_button.set_tooltip_text(Some("Search"));
        search_button.add_css_class("flat");

        // Create header container
        let header_container = GtkBox::new(Orientation::Horizontal, 0);
        header_container.set_hexpand(true);
        
        // Left section: window controls
        let header_left = GtkBox::new(Orientation::Horizontal, 0);
        header_left.set_size_request(260, -1);
        header_left.append(&window_controls);

        // Middle section: title
        let header_middle = GtkBox::new(Orientation::Horizontal, 8);
        header_middle.set_size_request(420, -1);
        header_middle.append(&mailbox_title);
        
        // Right section: buttons
        let header_right = GtkBox::new(Orientation::Horizontal, 0);
        header_right.append(&compose_button);
        header_right.append(&reply_button);
        header_right.append(&reply_all_button);
        header_right.append(&forward_button);
        header_right.append(&archive_button);
        header_right.append(&delete_button);
        header_right.append(&search_button);

        // Add sections to container
        header_container.append(&header_left);
        header_container.append(&header_middle);
        header_container.append(&header_right);

        // Set the header container as the title widget
        header_bar.set_title_widget(Some(&header_container));
        header_bar.set_margin_end(0);
        header_bar.set_halign(Align::Fill);

        // Create main content box
        let content_box = GtkBox::new(Orientation::Vertical, 0);

        // Create main content area
        let main_content_box = GtkBox::new(Orientation::Horizontal, 0);

        // Create panes
        let left_pane = GtkBox::new(Orientation::Vertical, 0);
        let middle_pane = GtkBox::new(Orientation::Vertical, 0);
        let right_pane = GtkBox::new(Orientation::Vertical, 0);

        // Create paned widgets
        let main_paned = Paned::new(Orientation::Horizontal);
        let secondary_paned = Paned::new(Orientation::Horizontal);

        // Set pane sizes
        main_paned.set_position(config.ui.pane_sizes.left_pane_width);
        secondary_paned.set_position(config.ui.pane_sizes.right_pane_width);

        // Create widgets
        let mailbox_tree = MailboxTree::new(storage.clone());
        let message_list = MessageList::new(storage.clone());
        let message_view = MessageView::new();
        let search_bar = SearchBar::new();
        let status_bar = StatusBar::new();
        
        // Build the mailbox tree with demo data
        mailbox_tree.build_demo();
        
        // Update the message list with demo data
        message_list.update_messages("demo:INBOX");

        // Assemble the layout
        main_paned.set_start_child(Some(&left_pane));
        main_paned.set_end_child(Some(&secondary_paned));
        
        secondary_paned.set_start_child(Some(&middle_pane));
        secondary_paned.set_end_child(Some(&right_pane));

        // Add widgets to panes
        left_pane.append(&mailbox_tree.widget);
        middle_pane.append(&message_list.widget);
        right_pane.append(&message_view.widget);

        main_content_box.append(&main_paned);
        content_box.append(&header_bar);
        content_box.append(&main_content_box);
        content_box.append(&status_bar.widget);
        window.set_child(Some(&content_box));

        // Set up window properties
        if config.ui.window_geometry.maximized {
            window.maximize();
        }

        // Connect window close event
        window.connect_close_request(|window| {
            // Hide window instead of closing (minimize to tray)
            window.hide();
            gtk4::glib::Propagation::Stop
        });

        // Connect quit action
        let quit_callback = Arc::new(Mutex::new(None::<Box<dyn Fn() + Send + Sync>>));

        Ok(Self {
            window,
            content_box,
            header_bar,
            left_pane,
            middle_pane,
            right_pane,
            main_paned,
            secondary_paned,
            mailbox_tree,
            message_list,
            message_view,
            search_bar,
            status_bar,
            quit_callback,
        })
    }

    /// Show the main window
    pub fn show(&self) {
        self.window.present();
    }

    /// Hide the main window
    pub fn hide(&self) {
        self.window.hide();
    }

    /// Check if window is visible
    pub fn is_visible(&self) -> bool {
        self.window.is_visible()
    }

    /// Get the GTK window
    pub fn window(&self) -> &ApplicationWindow {
        &self.window
    }

    /// Set the quit callback
    pub async fn set_quit_callback<F>(&self, callback: F)
    where
        F: Fn() + Send + Sync + 'static,
    {
        let mut cb = self.quit_callback.lock().await;
        *cb = Some(Box::new(callback));
    }
}

impl Clone for MainWindow {
    fn clone(&self) -> Self {
        Self {
            window: self.window.clone(),
            content_box: self.content_box.clone(),
            header_bar: self.header_bar.clone(),
            left_pane: self.left_pane.clone(),
            middle_pane: self.middle_pane.clone(),
            right_pane: self.right_pane.clone(),
            main_paned: self.main_paned.clone(),
            secondary_paned: self.secondary_paned.clone(),
            mailbox_tree: self.mailbox_tree.clone(),
            message_list: self.message_list.clone(),
            message_view: self.message_view.clone(),
            search_bar: self.search_bar.clone(),
            status_bar: self.status_bar.clone(),
            quit_callback: self.quit_callback.clone(),
        }
    }
}
