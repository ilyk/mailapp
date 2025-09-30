use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Label, Orientation, Align, Button};
use libadwaita::prelude::*;
use libadwaita::Avatar;
use email_backend::EmailMessage;
use std::cell::RefCell;

pub struct MessageView {
    pub widget: GtkBox,
    message_header: GtkBox,
    message_body: Label,
    current_message: RefCell<Option<EmailMessage>>,
}

impl MessageView {
    pub fn new() -> Self {
        let widget = GtkBox::new(Orientation::Vertical, 0);
        widget.add_css_class("viewer");
        
        // Message header with avatar, from, date, and actions
        let message_header = create_message_header();
        
        // Message body
        let message_body = Label::new(None);
        message_body.add_css_class("message-body");
        message_body.set_xalign(0.0);
        message_body.set_wrap(true);
        message_body.set_justify(gtk4::Justification::Left);
        message_body.set_margin_top(20);
        
        widget.append(&message_header);
        widget.append(&message_body);
        
        Self {
            widget,
            message_header,
            message_body,
            current_message: RefCell::new(None),
        }
    }
    
    pub fn show_message(&self, message: &EmailMessage) {
        *self.current_message.borrow_mut() = Some(message.clone());
        self.update_header(message);
        self.update_body(message);
    }
    
    fn update_header(&self, message: &EmailMessage) {
        // Update avatar with sender initial
        if let Some(avatar) = self.message_header.first_child().and_then(|child| child.downcast::<Avatar>().ok()) {
            let initial = message.from.chars().next().unwrap_or('?').to_uppercase().next().unwrap_or('?');
            avatar.set_text(Some(&initial.to_string()));
        }
        
        // Update from name and email
        if let Some(from_container) = self.message_header.nth_child(1).and_then(|child| child.downcast::<GtkBox>().ok()) {
            if let Some(from_name) = from_container.first_child().and_then(|child| child.downcast::<Label>().ok()) {
                // Extract name from sender (e.g., "Apple Developer <developer@apple.com>")
                let from_text = if let Some(start) = message.from.find('<') {
                    message.from[..start].trim()
                } else {
                    &message.from
                };
                from_name.set_text(from_text);
            }
            
            if let Some(from_email) = from_container.last_child().and_then(|child| child.downcast::<Label>().ok()) {
                // Extract email from sender
                let email_text = if let Some(start) = message.from.find('<') {
                    if let Some(end) = message.from.find('>') {
                        &message.from[start+1..end]
                    } else {
                        &message.from[start+1..]
                    }
                } else {
                    &message.from
                };
                from_email.set_text(&format!("<{}>", email_text));
            }
        }
        
        // Update date
        if let Some(date_label) = self.message_header.nth_child(2).and_then(|child| child.downcast::<Label>().ok()) {
            let date_str = format_time(&message.date);
            date_label.set_text(&date_str);
        }
    }
    
    fn update_body(&self, message: &EmailMessage) {
        self.message_body.set_text(&message.body_text);
    }
    
    pub fn clear(&self) {
        *self.current_message.borrow_mut() = None;
        self.message_body.set_text("");
        
        // Clear header fields
        if let Some(avatar) = self.message_header.first_child().and_then(|child| child.downcast::<Avatar>().ok()) {
            avatar.set_text(Some("?"));
        }
        
        if let Some(from_container) = self.message_header.nth_child(1).and_then(|child| child.downcast::<GtkBox>().ok()) {
            if let Some(from_name) = from_container.first_child().and_then(|child| child.downcast::<Label>().ok()) {
                from_name.set_text("");
            }
            if let Some(from_email) = from_container.last_child().and_then(|child| child.downcast::<Label>().ok()) {
                from_email.set_text("");
            }
        }
        
        if let Some(date_label) = self.message_header.nth_child(2).and_then(|child| child.downcast::<Label>().ok()) {
            date_label.set_text("");
        }
    }
}

fn create_message_header() -> GtkBox {
    let header = GtkBox::new(Orientation::Horizontal, 12);
    header.add_css_class("message-header");
    header.set_margin_start(36);
    header.set_margin_end(36);
    header.set_margin_top(20);
    header.set_margin_bottom(20);
    
    // Avatar
    let avatar = Avatar::new(28, None, true);
    avatar.set_text(Some("?"));
    avatar.add_css_class("message-avatar");
    
    // From info container
    let from_container = GtkBox::new(Orientation::Vertical, 2);
    from_container.set_hexpand(true);
    from_container.set_halign(Align::Start);
    
    let from_name = Label::new(None);
    from_name.add_css_class("message-from-name");
    from_name.set_xalign(0.0);
    
    let from_email = Label::new(None);
    from_email.add_css_class("message-from-email");
    from_email.set_xalign(0.0);
    
    from_container.append(&from_name);
    from_container.append(&from_email);
    
    // Date
    let date_label = Label::new(None);
    date_label.add_css_class("message-header-date");
    date_label.set_halign(Align::End);
    
    // Action buttons container
    let actions_container = GtkBox::new(Orientation::Horizontal, 4);
    actions_container.set_halign(Align::End);
    
    // Reply button
    let reply_button = Button::from_icon_name("mail-reply-sender-symbolic");
    reply_button.add_css_class("header-button");
    reply_button.set_tooltip_text(Some("Reply"));
    
    // Reply all button
    let reply_all_button = Button::from_icon_name("mail-reply-all-symbolic");
    reply_all_button.add_css_class("header-button");
    reply_all_button.set_tooltip_text(Some("Reply All"));
    
    // Forward button
    let forward_button = Button::from_icon_name("mail-forward-symbolic");
    forward_button.add_css_class("header-button");
    forward_button.set_tooltip_text(Some("Forward"));
    
    actions_container.append(&reply_button);
    actions_container.append(&reply_all_button);
    actions_container.append(&forward_button);
    
    header.append(&avatar);
    header.append(&from_container);
    header.append(&date_label);
    header.append(&actions_container);
    
    header
}

fn format_time(date: &time::OffsetDateTime) -> String {
    let now = time::OffsetDateTime::now_utc();
    let duration = now - *date;
    
    if duration.as_seconds_f64() < 3600.0 {
        format!("{}m ago", (duration.as_seconds_f64() / 60.0) as u32)
    } else if duration.as_seconds_f64() < 86400.0 {
        format!("{}h ago", (duration.as_seconds_f64() / 3600.0) as u32)
    } else if duration.as_seconds_f64() < 604800.0 {
        format!("{}d ago", (duration.as_seconds_f64() / 86400.0) as u32)
    } else {
        match date.month() {
            time::Month::January => format!("Jan {}, {}", date.day(), date.year()),
            time::Month::February => format!("Feb {}, {}", date.day(), date.year()),
            time::Month::March => format!("Mar {}, {}", date.day(), date.year()),
            time::Month::April => format!("Apr {}, {}", date.day(), date.year()),
            time::Month::May => format!("May {}, {}", date.day(), date.year()),
            time::Month::June => format!("Jun {}, {}", date.day(), date.year()),
            time::Month::July => format!("Jul {}, {}", date.day(), date.year()),
            time::Month::August => format!("Aug {}, {}", date.day(), date.year()),
            time::Month::September => format!("Sep {}, {}", date.day(), date.year()),
            time::Month::October => format!("Oct {}, {}", date.day(), date.year()),
            time::Month::November => format!("Nov {}, {}", date.day(), date.year()),
            time::Month::December => format!("Dec {}, {}", date.day(), date.year()),
        }
    }
}