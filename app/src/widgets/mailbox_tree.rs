//! Mailbox tree widget for sidebar navigation

use gtk4::prelude::*;
use gtk4::{Box as GtkBox, ListBox, ListBoxRow, Label, Orientation, Image, ScrolledWindow};
// use libadwaita::prelude::*;
// use libadwaita::ActionRow;
use asgard_core::storage::StorageManager;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::collections::HashMap;

/// Mailbox tree widget for the sidebar
pub struct MailboxTree {
    /// Main widget container
    pub widget: GtkBox,
    /// List box for mailbox items
    list_box: ListBox,
    /// Storage manager
    storage: Arc<Mutex<StorageManager>>,
    /// Accordion states (mailbox_name -> expanded)
    accordion_states: HashMap<String, bool>,
}

impl MailboxTree {
    /// Create a new mailbox tree widget
    pub fn new(storage: Arc<Mutex<StorageManager>>) -> Self {
        let widget = GtkBox::new(Orientation::Vertical, 0);
        widget.set_size_request(280, -1);
        widget.add_css_class("sidebar");
        
        let list_box = ListBox::new();
        list_box.set_selection_mode(gtk4::SelectionMode::Single);
        list_box.add_css_class("sidebar");
        
        let scrolled_window = ScrolledWindow::new();
        scrolled_window.set_child(Some(&list_box));
        scrolled_window.set_policy(gtk4::PolicyType::Never, gtk4::PolicyType::Automatic);
        scrolled_window.set_vexpand(true);
        scrolled_window.set_hexpand(false);
        
        widget.append(&scrolled_window);
        
        Self {
            widget,
            list_box,
            storage,
            accordion_states: HashMap::new(),
        }
    }
    
    /// Build the mailbox tree with demo data
    pub fn build_demo(&self) {
        // Clear existing items
        while let Some(child) = self.list_box.first_child() {
            self.list_box.remove(&child);
        }
        
        // FAVORITES section
        self.add_section_header("FAVORITES");
        
        // All Inboxes - EXPANDABLE
        let all_inboxes_expanded = self.accordion_states.get("ALL_INBOXES").copied().unwrap_or(false);
        if let Some((_revealer, _child_container)) = self.add_sidebar_item("All Inboxes", "mail-inbox-symbolic", false, "ALL_INBOXES", true, all_inboxes_expanded) {
            // Add account children for All Inboxes
            self.add_sidebar_item("Demo Account", "", false, "demo:INBOX", false, false);
        }
        
        self.add_sidebar_item("VIPs", "emblem-favorite-symbolic", false, "VIPS", false, false);
        self.add_sidebar_item("Flagged", "flag-symbolic", false, "FLAGGED", false, false);
        self.add_sidebar_item("Reminders", "alarm-symbolic", false, "REMINDERS", false, false);
        
        // All Drafts - EXPANDABLE
        let all_drafts_expanded = self.accordion_states.get("ALL_DRAFTS").copied().unwrap_or(false);
        if let Some((_revealer2, _child_container2)) = self.add_sidebar_item("All Drafts", "mail-drafts-symbolic", false, "ALL_DRAFTS", true, all_drafts_expanded) {
            // Add account children for All Drafts
            self.add_sidebar_item("Demo Account", "", false, "demo:DRAFTS", false, false);
        }
        
        // All Sent - EXPANDABLE
        let all_sent_expanded = self.accordion_states.get("ALL_SENT").copied().unwrap_or(false);
        if let Some((_revealer3, _child_container3)) = self.add_sidebar_item("All Sent", "mail-sent-symbolic", false, "ALL_SENT", true, all_sent_expanded) {
            // Add account children for All Sent
            self.add_sidebar_item("Demo Account", "", false, "demo:SENT", false, false);
        }
        
        // Individual account sections
        self.add_section_header("DEMO ACCOUNT");
        
        // Standard mailboxes for the demo account
        self.add_sidebar_item("Inbox", "mail-inbox-symbolic", true, "demo:INBOX", false, false);
        self.add_sidebar_item("Drafts", "mail-drafts-symbolic", false, "demo:DRAFTS", false, false);
        self.add_sidebar_item("Sent", "mail-sent-symbolic", false, "demo:SENT", false, false);
        self.add_sidebar_item("Junk", "mail-junk-symbolic", false, "demo:JUNK", false, false);
        self.add_sidebar_item("Trash", "user-trash-symbolic", false, "demo:TRASH", false, false);
        self.add_sidebar_item("Archive", "mail-archive-symbolic", false, "demo:ARCHIVE", false, false);
    }
    
    fn add_section_header(&self, title: &str) {
        let header = Label::new(Some(title));
        header.add_css_class("sidebar-section");
        header.set_xalign(0.0);
        header.set_margin_start(12);
        header.set_margin_end(12);
        header.set_margin_top(8);
        header.set_margin_bottom(4);
        
        let row = ListBoxRow::new();
        row.set_child(Some(&header));
        row.set_selectable(false);
        row.set_activatable(false);
        self.list_box.append(&row);
    }
    
    fn add_sidebar_item(
        &self,
        name: &str,
        icon_name: &str,
        is_selected: bool,
        mailbox_name: &str,
        expandable: bool,
        expanded: bool,
    ) -> Option<(gtk4::Revealer, ListBox)> {
        let row = ListBoxRow::new();
        row.set_activatable(true);
        
        let container = GtkBox::new(Orientation::Horizontal, 8);
        container.set_margin_start(8);
        container.set_margin_end(12);
        container.set_margin_top(6);
        container.set_margin_bottom(6);
        
        // Expand symbol (if expandable)
        if expandable {
            let expand_symbol = Label::new(Some(if expanded { "×" } else { "+" }));
            expand_symbol.add_css_class("expand-symbol");
            expand_symbol.set_xalign(0.0);
            container.append(&expand_symbol);
        } else {
            // Empty spacer to maintain alignment when not expandable
            let spacer = GtkBox::new(Orientation::Horizontal, 0);
            spacer.set_size_request(16, -1); // Same width as expand symbol
            container.append(&spacer);
        }
        
        // Icon (only if icon_name is not empty)
        if !icon_name.is_empty() {
            let icon = Image::from_icon_name(icon_name);
            icon.set_icon_size(gtk4::IconSize::Normal);
            icon.add_css_class("mailbox-icon");
            container.append(&icon);
        }
        
        // Label
        let label = Label::builder()
            .label(name)
            .xalign(0.0)
            .hexpand(true)
            .build();
        
        container.append(&label);
        
        // Unread count badge (demo data)
        let unread_count = if mailbox_name.contains("INBOX") { 2 } else { 0 };
        if unread_count > 0 {
            let badge = Label::builder()
                .label(&unread_count.to_string())
                .build();
            badge.add_css_class("count-badge");
            container.append(&badge);
        }
        
        row.set_child(Some(&container));
        self.list_box.append(&row);
        
        if is_selected {
            self.list_box.select_row(Some(&row));
        }
        
        // Handle expandable functionality
        let mut revealer = None;
        let mut child_container = None;
        
        if expandable {
            // Create revealer for child items
            let revealer_widget = gtk4::Revealer::new();
            revealer_widget.set_reveal_child(expanded);
            
            // Create a container for child items
            let child_list = ListBox::new();
            child_list.add_css_class("sidebar");
            revealer_widget.set_child(Some(&child_list));
            
            // Add separate click handlers for expand symbol and label
            let expand_symbol = if expandable {
                // Find the expand symbol we just added
                container.first_child().unwrap().downcast::<Label>().unwrap()
            } else {
                panic!("Expand symbol not found");
            };
            
            // Click handler for expand symbol - only toggles expansion, no focus change
            let expand_gesture = gtk4::GestureClick::new();
            let revealer_clone = revealer_widget.clone();
            let expand_symbol_clone = expand_symbol.clone();
            let mailbox_name_for_state = mailbox_name.to_string();
            expand_gesture.connect_pressed(move |gesture, _, _, _| {
                // Prevent focus change by claiming the event
                gesture.set_state(gtk4::EventSequenceState::Claimed);
                
                let is_revealed = revealer_clone.reveals_child();
                revealer_clone.set_reveal_child(!is_revealed);
                expand_symbol_clone.set_text(if !is_revealed { "×" } else { "+" });
                
                println!("Toggled accordion for: {}", mailbox_name_for_state);
            });
            expand_symbol.add_controller(expand_gesture);
            
            // Click handler for the entire row - switches mailbox (except when clicking expand symbol)
            let row_gesture = gtk4::GestureClick::new();
            let mailbox_name_clone = mailbox_name.to_string();
            let revealer_clone = revealer_widget.clone();
            let expand_symbol_clone = expand_symbol.clone();
            
            row_gesture.connect_pressed(move |_gesture, n_press, _, _| {
                if n_press == 1 {
                    // Single click - switch to this mailbox (for expandable groups like "All Inboxes")
                    println!("Switched to mailbox: {}", mailbox_name_clone);
                } else if n_press == 2 {
                    // Double click - toggle accordion expansion
                    let is_revealed = revealer_clone.reveals_child();
                    revealer_clone.set_reveal_child(!is_revealed);
                    expand_symbol_clone.set_text(if !is_revealed { "×" } else { "+" });
                    
                    println!("Toggled accordion for: {}", mailbox_name_clone);
                }
            });
            row.add_controller(row_gesture);
            
            // Add revealer to the list box
            self.list_box.append(&revealer_widget);
            
            revealer = Some(revealer_widget);
            child_container = Some(child_list);
        } else {
            // Add click handler to switch mailbox (only for non-expandable items)
            let mailbox_name = mailbox_name.to_string();
            
            // Use GestureClick for reliable click handling
            let gesture = gtk4::GestureClick::new();
            gesture.connect_pressed(move |_, _, _, _| {
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
}

impl Clone for MailboxTree {
    fn clone(&self) -> Self {
        Self {
            widget: self.widget.clone(),
            list_box: self.list_box.clone(),
            storage: self.storage.clone(),
            accordion_states: self.accordion_states.clone(),
        }
    }
}