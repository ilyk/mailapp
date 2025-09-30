//! Compose window for writing emails

use gtk4::prelude::*;
use gtk4::{ApplicationWindow, Box as GtkBox, Orientation, Button, Entry, TextView, Label, ScrolledWindow};
// use libadwaita::prelude::*;
use asgard_core::error::AsgardResult;

/// Compose window for writing emails
pub struct ComposeWindow {
    /// GTK window
    window: ApplicationWindow,
    /// Main content box
    content_box: GtkBox,
    /// To entry
    to_entry: Entry,
    /// Subject entry
    subject_entry: Entry,
    /// Message text view
    message_text: TextView,
    /// Send button
    send_button: Button,
}

impl ComposeWindow {
    /// Create a new compose window
    pub fn new() -> AsgardResult<Self> {
        // Create application window
        let window = ApplicationWindow::builder()
            .title("Compose Message")
            .default_width(800)
            .default_height(600)
            .build();

        // Create main content box
        let content_box = GtkBox::new(Orientation::Vertical, 8);
        content_box.set_margin_start(12);
        content_box.set_margin_end(12);
        content_box.set_margin_top(12);
        content_box.set_margin_bottom(12);

        // To field
        let to_label = Label::new(Some("To:"));
        to_label.set_xalign(0.0);
        let to_entry = Entry::new();
        to_entry.set_placeholder_text(Some("recipient@example.com"));

        // Subject field
        let subject_label = Label::new(Some("Subject:"));
        subject_label.set_xalign(0.0);
        let subject_entry = Entry::new();
        subject_entry.set_placeholder_text(Some("Enter subject..."));

        // Message text view
        let message_label = Label::new(Some("Message:"));
        message_label.set_xalign(0.0);
        let message_text = TextView::new();
        message_text.set_wrap_mode(gtk4::WrapMode::Word);
        
        let message_scrolled = ScrolledWindow::new();
        message_scrolled.set_child(Some(&message_text));
        message_scrolled.set_vexpand(true);
        message_scrolled.set_hexpand(true);

        // Buttons
        let button_box = GtkBox::new(Orientation::Horizontal, 8);
        button_box.set_halign(gtk4::Align::End);

        let send_button = Button::with_label("Send");
        send_button.add_css_class("suggested-action");
        
        let cancel_button = Button::with_label("Cancel");
        cancel_button.add_css_class("destructive-action");

        // Connect button actions
        let window_clone = window.clone();
        send_button.connect_clicked(move |_| {
            println!("Send button clicked");
            window_clone.close();
        });

        let window_clone = window.clone();
        cancel_button.connect_clicked(move |_| {
            println!("Cancel button clicked");
            window_clone.close();
        });

        button_box.append(&cancel_button);
        button_box.append(&send_button);

        // Assemble the layout
        content_box.append(&to_label);
        content_box.append(&to_entry);
        content_box.append(&subject_label);
        content_box.append(&subject_entry);
        content_box.append(&message_label);
        content_box.append(&message_scrolled);
        content_box.append(&button_box);

        window.set_child(Some(&content_box));

        Ok(Self {
            window,
            content_box,
            to_entry,
            subject_entry,
            message_text,
            send_button,
        })
    }

    /// Show the compose window
    pub fn show(&self) {
        self.window.present();
    }

    /// Hide the compose window
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

impl Clone for ComposeWindow {
    fn clone(&self) -> Self {
        Self {
            window: self.window.clone(),
            content_box: self.content_box.clone(),
            to_entry: self.to_entry.clone(),
            subject_entry: self.subject_entry.clone(),
            message_text: self.message_text.clone(),
            send_button: self.send_button.clone(),
        }
    }
}