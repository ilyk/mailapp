//! Preferences window for application settings

use gtk4::prelude::*;
use gtk4::{ApplicationWindow, Box as GtkBox, Orientation, Button, Label, Switch, Separator};
// use libadwaita::prelude::*;
use asgard_core::error::AsgardResult;

/// Preferences window for application settings
pub struct PreferencesWindow {
    /// GTK window
    window: ApplicationWindow,
    /// Main content box
    content_box: GtkBox,
}

impl PreferencesWindow {
    /// Create a new preferences window
    pub fn new() -> AsgardResult<Self> {
        // Create application window
        let window = ApplicationWindow::builder()
            .title("Preferences")
            .default_width(600)
            .default_height(400)
            .build();

        // Create main content box
        let content_box = GtkBox::new(Orientation::Vertical, 12);
        content_box.set_margin_start(24);
        content_box.set_margin_end(24);
        content_box.set_margin_top(24);
        content_box.set_margin_bottom(24);

        // General section
        let general_label = Label::new(Some("General"));
        general_label.add_css_class("title-2");
        content_box.append(&general_label);

        // Notifications setting
        let notifications_box = GtkBox::new(Orientation::Horizontal, 12);
        let notifications_label = Label::new(Some("Enable notifications"));
        notifications_label.set_hexpand(true);
        notifications_label.set_xalign(0.0);
        
        let notifications_switch = Switch::new();
        notifications_switch.set_active(true);
        
        notifications_box.append(&notifications_label);
        notifications_box.append(&notifications_switch);
        content_box.append(&notifications_box);

        // Separator
        let separator1 = Separator::new(Orientation::Horizontal);
        content_box.append(&separator1);

        // Appearance section
        let appearance_label = Label::new(Some("Appearance"));
        appearance_label.add_css_class("title-2");
        content_box.append(&appearance_label);

        // Dark mode setting
        let dark_mode_box = GtkBox::new(Orientation::Horizontal, 12);
        let dark_mode_label = Label::new(Some("Dark mode"));
        dark_mode_label.set_hexpand(true);
        dark_mode_label.set_xalign(0.0);
        
        let dark_mode_switch = Switch::new();
        dark_mode_switch.set_active(false);
        
        dark_mode_box.append(&dark_mode_label);
        dark_mode_box.append(&dark_mode_switch);
        content_box.append(&dark_mode_box);

        // Separator
        let separator2 = Separator::new(Orientation::Horizontal);
        content_box.append(&separator2);

        // Buttons
        let button_box = GtkBox::new(Orientation::Horizontal, 8);
        button_box.set_halign(gtk4::Align::End);

        let close_button = Button::with_label("Close");
        close_button.add_css_class("suggested-action");

        // Connect button action
        let window_clone = window.clone();
        close_button.connect_clicked(move |_| {
            window_clone.close();
        });

        button_box.append(&close_button);
        content_box.append(&button_box);

        window.set_child(Some(&content_box));

        Ok(Self {
            window,
            content_box,
        })
    }

    /// Show the preferences window
    pub fn show(&self) {
        self.window.present();
    }

    /// Hide the preferences window
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

impl Clone for PreferencesWindow {
    fn clone(&self) -> Self {
        Self {
            window: self.window.clone(),
            content_box: self.content_box.clone(),
        }
    }
}