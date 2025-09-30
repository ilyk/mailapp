//! Message list widget with three-tier layout

use gtk4::prelude::*;
use gtk4::{Box as GtkBox, ListBox, ListBoxRow, Label, Orientation, Image, ScrolledWindow, ToggleButton};
use asgard_core::storage::StorageManager;
use asgard_core::message::Message;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::collections::HashMap;
use crate::thread_helpers::{group_into_threads, format_date_short};

/// Message list widget for the middle pane
pub struct MessageList {
    /// Main widget container
    pub widget: GtkBox,
    /// List box for message items
    list_box: ListBox,
    /// Storage manager
    storage: Arc<Mutex<StorageManager>>,
    /// Category states
    category_states: HashMap<String, bool>,
}

impl MessageList {
    /// Create a new message list widget
    pub fn new(storage: Arc<Mutex<StorageManager>>) -> Self {
        let widget = GtkBox::new(Orientation::Vertical, 0);
        widget.add_css_class("message-list");
        
        // Category chips (Gmail-style)
        let categories_box = GtkBox::new(Orientation::Horizontal, 4);
        categories_box.set_margin_start(12);
        categories_box.set_margin_end(12);
        categories_box.set_margin_top(8);
        categories_box.set_margin_bottom(4);
        categories_box.add_css_class("category-chips");
        
        let categories = ["Primary", "Social", "Promotions", "Updates", "Forums", "Important"];
        let mut first_button = None;
        
        for (i, category) in categories.iter().enumerate() {
            let button = ToggleButton::with_label(category);
            button.add_css_class("pill");
            button.set_tooltip_text(Some(&format!("Show {} messages", category)));
            
            // Use default to first category active
            let is_active = i == 0;
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
        let list_box = ListBox::new();
        list_box.set_selection_mode(gtk4::SelectionMode::Single);
        list_box.add_css_class("message-list");
        
        let scrolled_window = ScrolledWindow::new();
        scrolled_window.set_child(Some(&list_box));
        scrolled_window.set_policy(gtk4::PolicyType::Automatic, gtk4::PolicyType::Automatic);
        scrolled_window.set_vexpand(true);
        scrolled_window.set_hexpand(true);
        
        widget.append(&categories_box);
        widget.append(&scrolled_window);
        
        Self {
            widget,
            list_box,
            storage,
            category_states: HashMap::new(),
        }
    }
    
    /// Update messages for a specific mailbox
    pub fn update_messages(&self, _mailbox_name: &str) {
        // Clear existing items
        while let Some(child) = self.list_box.first_child() {
            self.list_box.remove(&child);
        }
        
        // Create demo messages
        let demo_messages = self.create_demo_messages();
        
        // Group messages into threads
        let threads = group_into_threads(demo_messages);
        
        // Add threads to the message list
        for (i, thread) in threads.iter().enumerate() {
            let last = thread.last().unwrap();
            let preview = self.truncate_preview(&self.get_message_body(last), 2);
            let time = format_date_short(&last.headers.date.unwrap_or(time::OffsetDateTime::now_utc()));
            
            // Determine if this is an outgoing reply
            let last_is_outgoing_reply = thread.messages.last()
                .map(|m| crate::thread_helpers::is_outgoing_message(m) && crate::thread_helpers::is_reply_message(m))
                .unwrap_or(false);
            
            let row = self.create_message_row_three_tier(
                &self.get_sender_name(last),
                &last.headers.subject,
                &preview,
                &time,
                thread.any_unread(),
                i == 0, // select first thread initially
                thread.has_attachments(),
                last_is_outgoing_reply,
            );
            
            // Store thread ID on the row for selection handling
            unsafe {
                row.set_data("thread-id", thread.id.clone());
            }
            self.list_box.append(&row);
        }
        
        // Select the first thread
        if let Some(first_row) = self.list_box.row_at_index(0) {
            self.list_box.select_row(Some(&first_row));
        }
    }
    
    fn create_demo_messages(&self) -> Vec<Message> {
        use asgard_core::message::{MessageHeaders, EmailAddress, MessageImportance};
        use std::collections::HashMap;
        use uuid::Uuid;
        
        let mut messages = Vec::new();
        
        // Demo message 1
        let headers1 = MessageHeaders {
            message_id: Some("msg1@example.com".to_string()),
            in_reply_to: None,
            references: None,
            subject: "Welcome to Asgard Mail".to_string(),
            from: vec![EmailAddress {
                name: Some("Apple Developer".to_string()),
                email: "developer@apple.com".to_string(),
            }],
            to: vec![EmailAddress {
                name: None,
                email: "demo@gmail.com".to_string(),
            }],
            cc: vec![],
            bcc: vec![],
            reply_to: vec![],
            date: Some(time::OffsetDateTime::now_utc() - time::Duration::hours(2)),
            received_date: Some(time::OffsetDateTime::now_utc() - time::Duration::hours(2)),
            importance: MessageImportance::Normal,
            custom: HashMap::new(),
        };
        
        let mut message1 = Message::new(Uuid::new_v4(), Uuid::new_v4(), headers1);
        message1.mark_as_unread();
        messages.push(message1);
        
        // Demo message 2
        let headers2 = MessageHeaders {
            message_id: Some("msg2@example.com".to_string()),
            in_reply_to: Some("msg1@example.com".to_string()),
            references: Some("msg1@example.com".to_string()),
            subject: "Re: Welcome to Asgard Mail".to_string(),
            from: vec![EmailAddress {
                name: Some("Demo User".to_string()),
                email: "demo@gmail.com".to_string(),
            }],
            to: vec![EmailAddress {
                name: Some("Apple Developer".to_string()),
                email: "developer@apple.com".to_string(),
            }],
            cc: vec![],
            bcc: vec![],
            reply_to: vec![],
            date: Some(time::OffsetDateTime::now_utc() - time::Duration::hours(1)),
            received_date: Some(time::OffsetDateTime::now_utc() - time::Duration::hours(1)),
            importance: MessageImportance::Normal,
            custom: HashMap::new(),
        };
        
        let mut message2 = Message::new(Uuid::new_v4(), Uuid::new_v4(), headers2);
        message2.mark_as_read();
        messages.push(message2);
        
        // Demo message 3
        let headers3 = MessageHeaders {
            message_id: Some("msg3@example.com".to_string()),
            in_reply_to: None,
            references: None,
            subject: "Getting Started with Email".to_string(),
            from: vec![EmailAddress {
                name: Some("Support Team".to_string()),
                email: "support@example.com".to_string(),
            }],
            to: vec![EmailAddress {
                name: None,
                email: "demo@gmail.com".to_string(),
            }],
            cc: vec![],
            bcc: vec![],
            reply_to: vec![],
            date: Some(time::OffsetDateTime::now_utc() - time::Duration::minutes(30)),
            received_date: Some(time::OffsetDateTime::now_utc() - time::Duration::minutes(30)),
            importance: MessageImportance::Normal,
            custom: HashMap::new(),
        };
        
        let mut message3 = Message::new(Uuid::new_v4(), Uuid::new_v4(), headers3);
        message3.mark_as_unread();
        messages.push(message3);
        
        messages
    }
    
    fn get_sender_name(&self, message: &Message) -> String {
        if let Some(from) = message.headers.from.first() {
            if let Some(name) = &from.name {
                name.clone()
            } else {
                from.email.clone()
            }
        } else {
            "Unknown Sender".to_string()
        }
    }
    
    fn get_message_body(&self, message: &Message) -> String {
        // For demo purposes, return a sample body
        match message.headers.subject.as_str() {
            "Welcome to Asgard Mail" => "Welcome to Asgard Mail! This is a demo message to show you how the application works. You can compose, read, and manage your emails with this modern interface.".to_string(),
            "Re: Welcome to Asgard Mail" => "Thank you for the welcome message! I'm excited to try out this new email client.".to_string(),
            "Getting Started with Email" => "Here are some tips to get started with Asgard Mail. The interface is designed to be intuitive and efficient.".to_string(),
            _ => "This is a demo message body. In a real implementation, this would contain the actual email content.".to_string(),
        }
    }
    
    fn truncate_preview(&self, text: &str, max_lines: usize) -> String {
        let lines: Vec<&str> = text.lines().collect();
        if lines.len() <= max_lines {
            text.to_string()
        } else {
            let truncated = lines[..max_lines].join("\n");
            format!("{}...", truncated)
        }
    }
    
    fn create_message_row_three_tier(
        &self,
        sender: &str,
        subject: &str,
        preview: &str,
        time: &str,
        is_unread: bool,
        is_selected: bool,
        has_attachments: bool,
        is_reply: bool,
    ) -> ListBoxRow {
        let row = ListBoxRow::new();
        row.set_activatable(true);
        
        // Main row container
        let row_container = GtkBox::new(Orientation::Horizontal, 10);
        row_container.set_margin_start(12);
        row_container.set_margin_end(12);
        row_container.set_margin_top(6);
        row_container.set_margin_bottom(6);
        
        // Left badges - clear and add only one indicator
        let badges = GtkBox::new(Orientation::Horizontal, 6);
        badges.add_css_class("badge-box");
        badges.set_valign(gtk4::Align::Start); // Align badges to top
        
        // Only show reply icon, no unread indicator to avoid vertical lines
        if is_reply {
            let reply_icon = Image::from_icon_name("mail-reply-sender-symbolic");
            reply_icon.set_icon_size(gtk4::IconSize::Normal);
            reply_icon.add_css_class("state-strong");
            badges.append(&reply_icon);
        } else if is_unread {
            let unread_dot = Label::builder().label("•").build();
            unread_dot.add_css_class("unread-dot");
            badges.append(&unread_dot);
        }
        
        // Center content (three lines)
        let center = GtkBox::new(Orientation::Vertical, 2);
        center.set_hexpand(true);
        
        // Tier 1: Sender (bold, 13px)
        let sender_label = Label::builder()
            .label(sender)
            .xalign(0.0)
            .build();
        sender_label.add_css_class("row-sender");
        
        if is_unread {
            sender_label.add_css_class("semibold");
        }
        
        // Tier 2: Subject (regular 13px)
        let subject_label = Label::builder()
            .label(subject)
            .xalign(0.0)
            .ellipsize(gtk4::pango::EllipsizeMode::End)
            .build();
        subject_label.add_css_class("row-subject");
        
        if is_unread {
            subject_label.add_css_class("semibold");
        }
        
        // Tier 3: Snippet (12px, exactly two lines, muted)
        let snippet_label = Label::builder()
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
        let meta = GtkBox::new(Orientation::Vertical, 2);
        
        // Date/Time (12px, muted, right-aligned)
        let date_label = Label::builder()
            .label(time)
            .xalign(1.0)
            .build();
        date_label.add_css_class("row-date");
        
        // Attachment paperclip (under date)
        let attachment_icon = Image::from_icon_name("mail-attachment-symbolic");
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
}

impl Clone for MessageList {
    fn clone(&self) -> Self {
        Self {
            widget: self.widget.clone(),
            list_box: self.list_box.clone(),
            storage: self.storage.clone(),
            category_states: self.category_states.clone(),
        }
    }
}