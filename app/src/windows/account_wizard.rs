//! Account wizard for adding email accounts

use gtk4::prelude::*;
use gtk4::{ApplicationWindow, Box as GtkBox, Orientation, Button, Entry, Label, ComboBoxText, Separator};
// use libadwaita::prelude::*;
use asgard_core::error::AsgardResult;

/// Account wizard for adding email accounts
pub struct AccountWizard {
    /// GTK window
    window: ApplicationWindow,
    /// Main content box
    content_box: GtkBox,
    /// Email entry
    email_entry: Entry,
    /// Password entry
    password_entry: Entry,
    /// Provider combo box
    provider_combo: ComboBoxText,
}

impl AccountWizard {
    /// Create a new account wizard
    pub fn new() -> AsgardResult<Self> {
        // Create application window
        let window = ApplicationWindow::builder()
            .title("Add Email Account")
            .default_width(500)
            .default_height(400)
            .build();

        // Create main content box
        let content_box = GtkBox::new(Orientation::Vertical, 12);
        content_box.set_margin_start(24);
        content_box.set_margin_end(24);
        content_box.set_margin_top(24);
        content_box.set_margin_bottom(24);

        // Title
        let title_label = Label::new(Some("Add Email Account"));
        title_label.add_css_class("title-2");
        content_box.append(&title_label);

        // Description
        let desc_label = Label::new(Some("Enter your email account details to get started."));
        desc_label.add_css_class("dim-label");
        content_box.append(&desc_label);

        // Separator
        let separator1 = Separator::new(Orientation::Horizontal);
        content_box.append(&separator1);

        // Email field
        let email_label = Label::new(Some("Email Address:"));
        email_label.set_xalign(0.0);
        let email_entry = Entry::new();
        email_entry.set_placeholder_text(Some("your.email@example.com"));
        
        content_box.append(&email_label);
        content_box.append(&email_entry);

        // Password field
        let password_label = Label::new(Some("Password:"));
        password_label.set_xalign(0.0);
        let password_entry = Entry::new();
        password_entry.set_placeholder_text(Some("Enter your password"));
        password_entry.set_visibility(false); // Hide password
        
        content_box.append(&password_label);
        content_box.append(&password_entry);

        // Provider field
        let provider_label = Label::new(Some("Email Provider:"));
        provider_label.set_xalign(0.0);
        let provider_combo = ComboBoxText::new();
        provider_combo.append_text("Gmail");
        provider_combo.append_text("Outlook");
        provider_combo.append_text("Yahoo");
        provider_combo.append_text("Other");
        provider_combo.set_active(Some(0)); // Select Gmail by default
        
        content_box.append(&provider_label);
        content_box.append(&provider_combo);

        // Separator
        let separator2 = Separator::new(Orientation::Horizontal);
        content_box.append(&separator2);

        // Buttons
        let button_box = GtkBox::new(Orientation::Horizontal, 8);
        button_box.set_halign(gtk4::Align::End);

        let add_button = Button::with_label("Add Account");
        add_button.add_css_class("suggested-action");
        
        let cancel_button = Button::with_label("Cancel");
        cancel_button.add_css_class("destructive-action");

        // Connect button actions
        let window_clone = window.clone();
        add_button.connect_clicked(move |_| {
            println!("Add account button clicked");
            window_clone.close();
        });

        let window_clone = window.clone();
        cancel_button.connect_clicked(move |_| {
            println!("Cancel button clicked");
            window_clone.close();
        });

        button_box.append(&cancel_button);
        button_box.append(&add_button);
        content_box.append(&button_box);

        window.set_child(Some(&content_box));

        Ok(Self {
            window,
            content_box,
            email_entry,
            password_entry,
            provider_combo,
        })
    }

    /// Show the account wizard
    pub fn show(&self) {
        self.window.present();
    }

    /// Hide the account wizard
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
}

impl Clone for AccountWizard {
    fn clone(&self) -> Self {
        Self {
            window: self.window.clone(),
            content_box: self.content_box.clone(),
            email_entry: self.email_entry.clone(),
            password_entry: self.password_entry.clone(),
            provider_combo: self.provider_combo.clone(),
        }
    }
}