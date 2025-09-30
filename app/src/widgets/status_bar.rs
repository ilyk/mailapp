//! Status bar widget for application state

use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Label, Orientation, ProgressBar};
use std::cell::RefCell;

/// Status bar widget
pub struct StatusBar {
    /// Main widget container
    pub widget: GtkBox,
    /// Status label
    status_label: Label,
    /// Progress bar
    progress_bar: ProgressBar,
    /// Connection status label
    connection_label: Label,
    /// Current status
    current_status: RefCell<String>,
}

impl StatusBar {
    /// Create a new status bar widget
    pub fn new() -> Self {
        let widget = GtkBox::new(Orientation::Horizontal, 8);
        widget.set_margin_start(12);
        widget.set_margin_end(12);
        widget.set_margin_top(4);
        widget.set_margin_bottom(4);
        widget.add_css_class("status-bar");
        
        // Status label
        let status_label = Label::new(Some("Ready"));
        status_label.set_xalign(0.0);
        status_label.add_css_class("status-label");
        
        // Progress bar (initially hidden)
        let progress_bar = ProgressBar::new();
        progress_bar.set_visible(false);
        progress_bar.add_css_class("status-progress");
        
        // Connection status label
        let connection_label = Label::new(Some("Connected"));
        connection_label.set_xalign(1.0);
        connection_label.add_css_class("connection-status");
        
        // Spacer
        let spacer = GtkBox::new(Orientation::Horizontal, 0);
        spacer.set_hexpand(true);
        
        widget.append(&status_label);
        widget.append(&progress_bar);
        widget.append(&spacer);
        widget.append(&connection_label);
        
        Self {
            widget,
            status_label,
            progress_bar,
            connection_label,
            current_status: RefCell::new("Ready".to_string()),
        }
    }
    
    /// Set the status message
    pub fn set_status(&self, status: &str) {
        self.status_label.set_text(status);
        *self.current_status.borrow_mut() = status.to_string();
    }
    
    /// Get the current status
    pub fn get_status(&self) -> String {
        self.current_status.borrow().clone()
    }
    
    /// Show progress bar with message
    pub fn show_progress(&self, message: &str, progress: f64) {
        self.set_status(message);
        self.progress_bar.set_fraction(progress);
        self.progress_bar.set_visible(true);
    }
    
    /// Hide progress bar
    pub fn hide_progress(&self) {
        self.progress_bar.set_visible(false);
    }
    
    /// Set connection status
    pub fn set_connection_status(&self, status: &str, connected: bool) {
        self.connection_label.set_text(status);
        if connected {
            self.connection_label.add_css_class("connected");
            self.connection_label.remove_css_class("disconnected");
        } else {
            self.connection_label.add_css_class("disconnected");
            self.connection_label.remove_css_class("connected");
        }
    }
    
    /// Show sync status
    pub fn show_sync_status(&self, message: &str) {
        self.set_status(&format!("Syncing: {}", message));
        self.show_progress(&format!("Syncing: {}", message), 0.0);
    }
    
    /// Hide sync status
    pub fn hide_sync_status(&self) {
        self.hide_progress();
        self.set_status("Ready");
    }
    
    /// Show error status
    pub fn show_error(&self, error: &str) {
        self.set_status(&format!("Error: {}", error));
        self.connection_label.set_text("Error");
        self.connection_label.add_css_class("error");
        self.connection_label.remove_css_class("connected");
        self.connection_label.remove_css_class("disconnected");
    }
    
    /// Clear error status
    pub fn clear_error(&self) {
        self.connection_label.remove_css_class("error");
        self.set_connection_status("Connected", true);
    }
}

impl Clone for StatusBar {
    fn clone(&self) -> Self {
        Self {
            widget: self.widget.clone(),
            status_label: self.status_label.clone(),
            progress_bar: self.progress_bar.clone(),
            connection_label: self.connection_label.clone(),
            current_status: RefCell::new(self.current_status.borrow().clone()),
        }
    }
}