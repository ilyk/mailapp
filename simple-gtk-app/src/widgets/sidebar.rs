use gtk4::prelude::*;
use gtk4::{Box as GtkBox, ListBox, ListBoxRow, Label, Orientation};
use libadwaita::prelude::*;
use libadwaita::ActionRow;
use email_backend::EmailBackend;
use std::rc::Rc;
use std::cell::RefCell;

pub struct Sidebar {
    pub widget: GtkBox,
    list_box: ListBox,
    backend: Rc<RefCell<EmailBackend>>,
    on_mailbox_selected: Box<dyn Fn(String)>,
}

impl Sidebar {
    pub fn new(
        backend: Rc<RefCell<EmailBackend>>,
        on_mailbox_selected: Box<dyn Fn(String)>,
    ) -> Self {
        let widget = GtkBox::new(Orientation::Vertical, 0);
        widget.set_size_request(280, -1);
        widget.add_css_class("sidebar");
        
        let list_box = ListBox::new();
        list_box.set_selection_mode(gtk4::SelectionMode::Single);
        
        Self {
            widget,
            list_box,
            backend,
            on_mailbox_selected,
        }
    }
    
    pub fn build(&self, account_id: uuid::Uuid) {
        // Clear existing items
        while let Some(child) = self.list_box.first_child() {
            self.list_box.remove(&child);
        }
        
        // Favorites section
        self.add_section_header("FAVORITES");
        self.add_mailbox_item("Inbox", "mail-inbox-symbolic", account_id, "INBOX", 2);
        self.add_mailbox_item("Sent", "mail-sent-symbolic", account_id, "Sent", 0);
        
        // Smart Mailboxes section
        self.add_section_header("SMART MAILBOXES");
        // Could add smart mailboxes here
        
        // Account section
        self.add_section_header("DEMO ACCOUNT");
        self.add_mailbox_item("Drafts", "mail-drafts-symbolic", account_id, "Drafts", 0);
        self.add_mailbox_item("Junk", "mail-junk-symbolic", account_id, "Spam", 0);
        self.add_mailbox_item("Trash", "user-trash-symbolic", account_id, "Trash", 0);
        self.add_mailbox_item("Archive", "mail-archive-symbolic", account_id, "Archive", 0);
        
        self.widget.append(&self.list_box);
        
        // Connect selection
        let callback = self.on_mailbox_selected.clone();
        self.list_box.connect_row_selected(move |_, row| {
            if let Some(row) = row {
                if let Some(action_row) = row.downcast_ref::<ActionRow>() {
                    if let Some(mailbox_name) = action_row.property::<Option<String>>("mailbox-name") {
                        if let Some(mailbox_name) = mailbox_name {
                            callback(mailbox_name);
                        }
                    }
                }
            }
        });
    }
    
    fn add_section_header(&self, title: &str) {
        let header = Label::new(Some(title));
        header.add_css_class("sidebar-section");
        header.set_xalign(0.0);
        header.set_margin_start(12);
        header.set_margin_end(12);
        header.set_margin_top(8);
        header.set_margin_bottom(4);
        
        self.widget.append(&header);
    }
    
    fn add_mailbox_item(
        &self,
        display_name: &str,
        icon_name: &str,
        account_id: uuid::Uuid,
        mailbox_name: &str,
        unread_count: u32,
    ) {
        let row = ActionRow::new();
        row.set_title(display_name);
        row.set_activatable(true);
        
        // Set mailbox name as property for selection callback
        row.set_property("mailbox-name", mailbox_name);
        
        // Add icon
        let icon = gtk4::Image::from_icon_name(icon_name);
        icon.set_icon_size(gtk4::IconSize::Normal);
        icon.add_css_class("mailbox-icon");
        row.add_prefix(&icon);
        
        // Add unread count badge if > 0
        if unread_count > 0 {
            let badge_label = Label::new(Some(&unread_count.to_string()));
            badge_label.add_css_class("count-badge");
            row.add_suffix(&badge_label);
        }
        
        self.list_box.append(&row);
        
        // Select first item (Inbox)
        if mailbox_name == "INBOX" {
            self.list_box.select_row(Some(&row));
        }
    }
}