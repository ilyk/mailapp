//! Message view widget for displaying email content

use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Label, Orientation, Align, Button, ScrolledWindow, Separator, DrawingArea};
// use libadwaita::prelude::*;
// use libadwaita::Avatar;
use asgard_core::message::Message;
use std::cell::RefCell;
use time::OffsetDateTime;

/// Message view widget for the right pane
pub struct MessageView {
    /// Main widget container
    pub widget: GtkBox,
    /// Meta bar with message count
    meta_bar: GtkBox,
    /// Meta count label
    meta_count: Label,
    /// Scrolled window for message cards
    scroller: ScrolledWindow,
    /// Container for message cards
    cards: GtkBox,
    /// Current message
    current_message: RefCell<Option<Message>>,
}

impl MessageView {
    /// Create a new message view widget
    pub fn new() -> Self {
        let widget = GtkBox::new(Orientation::Vertical, 0);
        widget.add_css_class("message-view");
        widget.set_hexpand(true);

        // Meta bar
        let meta_bar = GtkBox::new(Orientation::Horizontal, 8);
        meta_bar.add_css_class("meta-bar");

        let count_label = Label::new(Some("No messages selected"));
        count_label.add_css_class("meta-count");

        let spacer = GtkBox::new(Orientation::Horizontal, 0);
        spacer.set_hexpand(true);

        let summarize = Button::new();
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

        // Scrolled window for message cards
        let scroller = ScrolledWindow::new();
        scroller.set_policy(gtk4::PolicyType::Automatic, gtk4::PolicyType::Automatic);
        scroller.set_hexpand(true);
        scroller.set_vexpand(true);

        let cards = GtkBox::new(Orientation::Vertical, 0);
        scroller.set_child(Some(&cards));

        widget.append(&meta_bar);
        widget.append(&scroller);

        Self {
            widget,
            meta_bar,
            meta_count: count_label,
            scroller,
            cards,
            current_message: RefCell::new(None),
        }
    }
    
    /// Show a message in the view
    pub fn show_message(&self, message: &Message) {
        *self.current_message.borrow_mut() = Some(message.clone());
        self.update_message_display(message);
    }
    
    /// Clear the message view
    pub fn clear(&self) {
        *self.current_message.borrow_mut() = None;
        
        // Clear all message cards
        while let Some(child) = self.cards.first_child() {
            self.cards.remove(&child);
        }
        
        // Reset the meta count
        self.meta_count.set_text("No messages selected");
    }
    
    fn update_message_display(&self, message: &Message) {
        // Clear existing cards
        while let Some(child) = self.cards.first_child() {
            self.cards.remove(&child);
        }
        
        // Update meta count
        self.meta_count.set_text("1 message");
        
        // Create message card
        let card = self.create_message_card(message);
        self.cards.append(&card);
    }
    
    fn create_message_card(&self, message: &Message) -> GtkBox {
        let root = GtkBox::new(Orientation::Vertical, 8);
        root.add_css_class("msg-card");
        root.set_vexpand(false);
        root.set_hexpand(false);

        // Header
        let header = self.create_message_header(message);
        
        // Separator
        let separator = Separator::new(Orientation::Horizontal);
        separator.add_css_class("card-sep");

        // Message body
        let body = self.create_message_body(message);

        root.append(&header);
        root.append(&separator);
        root.append(&body);

        root
    }
    
    fn create_message_header(&self, message: &Message) -> GtkBox {
        let header = GtkBox::new(Orientation::Horizontal, 8);
        
        // Avatar (36px circle with initials)
        let avatar = self.create_avatar_widget(&self.get_sender_name(message));
        avatar.add_css_class("avatar36");

        // Sender name (bold, 13-14px)
        let name_label = Label::builder()
            .label(&self.get_sender_name(message))
            .xalign(0.0)
            .build();
        name_label.add_css_class("hdr-name");

        // Subject (regular, 13px)
        let subject_label = Label::builder()
            .label(&message.headers.subject)
            .xalign(0.0)
            .build();
        subject_label.add_css_class("hdr-subject");

        // Build recipient display
        let recipient_container = self.build_recipient_display(message);

        // Date/time (right-aligned, 12px)
        let when_label = Label::builder()
            .label(&self.format_datetime_full(&message.headers.date.unwrap_or(OffsetDateTime::now_utc())))
            .xalign(1.0)
            .build();
        when_label.add_css_class("hdr-when");

        // Layout: Avatar | Name/Subject/Recipients | Spacer | Date
        let left_content = GtkBox::new(Orientation::Vertical, 2);
        left_content.append(&name_label);
        left_content.append(&subject_label);
        left_content.append(&recipient_container);

        let spacer = GtkBox::new(Orientation::Horizontal, 0);
        spacer.set_hexpand(true);

        let right_content = GtkBox::new(Orientation::Vertical, 2);
        right_content.add_css_class("right_content");
        right_content.set_halign(Align::End);
        right_content.append(&when_label);
        
        header.append(&avatar);
        header.append(&left_content);
        header.append(&spacer);
        header.append(&right_content);

        header
    }
    
    fn create_message_body(&self, message: &Message) -> GtkBox {
        let body_container = GtkBox::new(Orientation::Vertical, 0);
        body_container.add_css_class("msg-body");
        body_container.set_vexpand(false);
        body_container.set_hexpand(false);
        
        let body_text = self.get_message_body(message);
        let body_label = Label::builder()
            .label(&body_text)
            .xalign(0.0)
            .yalign(0.0)
            .wrap(true)
            .justify(gtk4::Justification::Left)
            .build();
        body_label.add_css_class("msg-body");
        body_label.set_hexpand(false);
        
        body_container.append(&body_label);
        body_container
    }
    
    fn create_avatar_widget(&self, sender: &str) -> gtk4::Widget {
        // Create a simple colored box with initials for now
        let drawing = DrawingArea::new();
        drawing.set_content_width(36);
        drawing.set_content_height(36);
        drawing.add_css_class("avatar36");
        
        // Clone sender string and compute values to move into closure
        let sender_owned = sender.to_string();
        let color = self.get_color_for_name(&sender_owned);
        let initials = self.get_initials(&sender_owned);
        
        drawing.set_draw_func(move |_, cr, w, h| {
            // Draw circle background
            cr.set_source_rgba(color.0, color.1, color.2, 1.0);
            cr.arc(w as f64 / 2.0, h as f64 / 2.0, (w.min(h) as f64) / 2.0, 0.0, 2.0 * std::f64::consts::PI);
            cr.fill().unwrap();
            
            // Draw initials
            cr.set_source_rgba(1.0, 1.0, 1.0, 1.0); // White text
            cr.select_font_face("Sans", gtk4::cairo::FontSlant::Normal, gtk4::cairo::FontWeight::Bold);
            cr.set_font_size(14.0);
            
            let extents = cr.text_extents(&initials).unwrap();
            let x = (w as f64 - extents.width()) / 2.0;
            let y = (h as f64 + extents.height()) / 2.0;
            cr.move_to(x, y);
            cr.show_text(&initials).unwrap();
        });
        
        drawing.upcast()
    }
    
    fn get_color_for_name(&self, name: &str) -> (f64, f64, f64) {
        // Simple hash-based color generation
        let hash = name.chars().fold(0u32, |acc, c| acc.wrapping_mul(31).wrapping_add(c as u32));
        let r = ((hash & 0xFF) as f64) / 255.0;
        let g = (((hash >> 8) & 0xFF) as f64) / 255.0;
        let b = (((hash >> 16) & 0xFF) as f64) / 255.0;
        (r, g, b)
    }
    
    fn get_initials(&self, name: &str) -> String {
        let words: Vec<&str> = name.split_whitespace().collect();
        if words.is_empty() {
            "?".to_string()
        } else if words.len() == 1 {
            words[0].chars().take(2).collect::<String>().to_uppercase()
        } else {
            format!("{}{}", 
                words[0].chars().next().unwrap_or('?'),
                words[1].chars().next().unwrap_or('?')
            ).to_uppercase()
        }
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
        // For demo purposes, return a sample body based on subject
        match message.headers.subject.as_str() {
            "Welcome to Asgard Mail" => "Welcome to Asgard Mail! This is a demo message to show you how the application works. You can compose, read, and manage your emails with this modern interface.\n\nThis email client is designed to be intuitive and efficient, with a clean interface inspired by Apple Mail. You can organize your emails into folders, search through your messages, and manage multiple email accounts.\n\nThank you for trying out Asgard Mail!".to_string(),
            "Re: Welcome to Asgard Mail" => "Thank you for the welcome message! I'm excited to try out this new email client. The interface looks clean and modern, and I'm looking forward to using it for my daily email management.\n\nI particularly like the three-pane layout and the way messages are organized. It reminds me of the Apple Mail interface which I'm familiar with.".to_string(),
            "Getting Started with Email" => "Here are some tips to get started with Asgard Mail:\n\n1. Set up your email accounts in the preferences\n2. Organize your emails into folders\n3. Use the search functionality to find specific messages\n4. Customize the interface to your liking\n\nIf you have any questions or need help, please don't hesitate to contact our support team.".to_string(),
            _ => "This is a demo message body. In a real implementation, this would contain the actual email content from the message parts.".to_string(),
        }
    }
    
    fn build_recipient_display(&self, message: &Message) -> GtkBox {
        let container = GtkBox::new(Orientation::Horizontal, 4);

        // To recipients
        if !message.headers.to.is_empty() {
            // "To:" label (bold)
            let to_label = Label::builder()
                .label("To:")
                .xalign(0.0)
                .build();
            to_label.add_css_class("hdr-recipients");
            to_label.add_css_class("recipient-label-bold");
            container.append(&to_label);
            
            // Recipients (normal weight)
            let to_emails: Vec<String> = message.headers.to.iter()
                .map(|addr| addr.email.clone())
                .collect();
            let to_recipients = Label::builder()
                .label(&format!(" {}", to_emails.join(", ")))
                .xalign(0.0)
                .build();
            to_recipients.add_css_class("hdr-recipients");
            to_recipients.add_css_class("recipient-text");
            container.append(&to_recipients);
        }

        // CC recipients
        if !message.headers.cc.is_empty() {
            // "Cc:" label (bold)
            let cc_label = Label::builder()
                .label("Cc:")
                .xalign(0.0)
                .build();
            cc_label.add_css_class("hdr-recipients");
            cc_label.add_css_class("recipient-label-bold");
            container.append(&cc_label);
            
            // Recipients (normal weight)
            let cc_emails: Vec<String> = message.headers.cc.iter()
                .map(|addr| addr.email.clone())
                .collect();
            let cc_recipients = Label::builder()
                .label(&format!(" {}", cc_emails.join(", ")))
                .xalign(0.0)
                .build();
            cc_recipients.add_css_class("hdr-recipients");
            cc_recipients.add_css_class("recipient-text");
            container.append(&cc_recipients);
        }

        // If no recipients, show placeholder
        if message.headers.to.is_empty() && message.headers.cc.is_empty() {
            let placeholder_label = Label::builder()
                .label("To: (no recipients)")
                .xalign(0.0)
                .build();
            placeholder_label.add_css_class("hdr-recipients");
            placeholder_label.add_css_class("recipient-label-bold");
            container.append(&placeholder_label);
        }

        container
    }
    
    fn format_datetime_full(&self, date: &OffsetDateTime) -> String {
        // Format as "September 21, 2025 at 4:24 PM"
        match date.month() {
            time::Month::January => format!("January {}, {} at {}", date.day(), date.year(), self.format_time_12h(date)),
            time::Month::February => format!("February {}, {} at {}", date.day(), date.year(), self.format_time_12h(date)),
            time::Month::March => format!("March {}, {} at {}", date.day(), date.year(), self.format_time_12h(date)),
            time::Month::April => format!("April {}, {} at {}", date.day(), date.year(), self.format_time_12h(date)),
            time::Month::May => format!("May {}, {} at {}", date.day(), date.year(), self.format_time_12h(date)),
            time::Month::June => format!("June {}, {} at {}", date.day(), date.year(), self.format_time_12h(date)),
            time::Month::July => format!("July {}, {} at {}", date.day(), date.year(), self.format_time_12h(date)),
            time::Month::August => format!("August {}, {} at {}", date.day(), date.year(), self.format_time_12h(date)),
            time::Month::September => format!("September {}, {} at {}", date.day(), date.year(), self.format_time_12h(date)),
            time::Month::October => format!("October {}, {} at {}", date.day(), date.year(), self.format_time_12h(date)),
            time::Month::November => format!("November {}, {} at {}", date.day(), date.year(), self.format_time_12h(date)),
            time::Month::December => format!("December {}, {} at {}", date.day(), date.year(), self.format_time_12h(date)),
        }
    }

    fn format_time_12h(&self, date: &OffsetDateTime) -> String {
        let hour = date.hour();
        let minute = date.minute();
        let (display_hour, am_pm) = if hour == 0 {
            (12, "AM")
        } else if hour < 12 {
            (hour, "AM")
        } else if hour == 12 {
            (12, "PM")
        } else {
            (hour - 12, "PM")
        };
        format!("{}:{:02} {}", display_hour, minute, am_pm)
    }
}

impl Clone for MessageView {
    fn clone(&self) -> Self {
        Self {
            widget: self.widget.clone(),
            meta_bar: self.meta_bar.clone(),
            meta_count: self.meta_count.clone(),
            scroller: self.scroller.clone(),
            cards: self.cards.clone(),
            current_message: RefCell::new(None),
        }
    }
}