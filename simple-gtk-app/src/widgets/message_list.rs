use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Label, Orientation, Align, Image, ListBox, ListBoxRow};
use libadwaita::prelude::*;
use email_backend::{EmailBackend, EmailMessage};
use std::rc::Rc;
use std::cell::RefCell;
use uuid::Uuid;

pub struct MessageList {
    pub widget: ListBox,
    backend: Rc<RefCell<EmailBackend>>,
    on_message_selected: Box<dyn Fn(Uuid)>,
}

impl MessageList {
    pub fn new(
        backend: Rc<RefCell<EmailBackend>>,
        on_message_selected: Box<dyn Fn(Uuid)>,
    ) -> Self {
        let list_box = ListBox::new();
        list_box.set_selection_mode(gtk4::SelectionMode::Single);
        
        Self {
            widget: list_box,
            backend,
            on_message_selected,
        }
    }
    
    pub fn update_messages(&self, account_id: Uuid, mailbox_name: &str) {
        // Clear existing items
        while let Some(child) = self.widget.first_child() {
            self.widget.remove(&child);
        }
        
        let backend_guard = self.backend.borrow();
        let messages = backend_guard.get_messages(account_id, mailbox_name);
        
        for (i, message) in messages.iter().enumerate() {
            let row = create_message_row(message, i == 0); // Select first message
            self.widget.append(&row);
        }
        
        // Connect selection
        let callback = self.on_message_selected.clone();
        self.widget.connect_row_selected(move |_, row| {
            if let Some(row) = row {
                if let Some(message_id) = row.property::<Option<Uuid>>("message-id") {
                    if let Some(message_id) = message_id {
                        callback(message_id);
                    }
                }
            }
        });
    }
}

fn create_message_row(message: &EmailMessage, is_selected: bool) -> ListBoxRow {
    let row = ListBoxRow::new();
    row.add_css_class("message-row");
    row.set_property("message-id", message.id);
    
    if is_selected {
        row.add_css_class("selected");
    }
    
    let container = GtkBox::new(Orientation::Horizontal, 0);
    container.set_margin_start(12);
    container.set_margin_end(12);
    container.set_margin_top(8);
    container.set_margin_bottom(8);
    
    // Left column: Subject and snippet
    let left_column = GtkBox::new(Orientation::Vertical, 4);
    left_column.set_hexpand(true);
    left_column.set_halign(Align::Start);
    
    let subject_label = Label::new(Some(&message.subject));
    subject_label.add_css_class("message-subject");
    subject_label.set_xalign(0.0);
    subject_label.set_ellipsize(gtk4::pango::EllipsizeMode::End);
    
    if !message.is_read {
        subject_label.add_css_class("unread");
    }
    
    let snippet_label = Label::new(Some(&message.body_text));
    snippet_label.add_css_class("message-snippet");
    snippet_label.set_xalign(0.0);
    snippet_label.set_ellipsize(gtk4::pango::EllipsizeMode::End);
    snippet_label.set_max_width_chars(60);
    
    left_column.append(&subject_label);
    left_column.append(&snippet_label);
    
    // Right column: Date and badges
    let right_column = GtkBox::new(Orientation::Horizontal, 4);
    right_column.set_halign(Align::End);
    right_column.set_valign(Align::Center);
    
    let date_label = Label::new(Some(&format_time(&message.date)));
    date_label.add_css_class("message-date");
    
    // Unread dot (will be shown conditionally)
    let unread_dot = GtkBox::new(Orientation::Horizontal, 0);
    unread_dot.add_css_class("unread-dot");
    unread_dot.set_visible(!message.is_read);
    
    // Attachment icon (will be shown conditionally)
    let attachment_icon = Image::from_icon_name("mail-attachment-symbolic");
    attachment_icon.set_icon_size(gtk4::IconSize::Normal);
    attachment_icon.set_opacity(0.6);
    attachment_icon.set_visible(false); // No attachments in our sample data
    
    right_column.append(&date_label);
    right_column.append(&unread_dot);
    right_column.append(&attachment_icon);
    
    container.append(&left_column);
    container.append(&right_column);
    
    row.set_child(Some(&container));
    row
}

fn format_time(date: &time::OffsetDateTime) -> String {
    let now = time::OffsetDateTime::now_utc();
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