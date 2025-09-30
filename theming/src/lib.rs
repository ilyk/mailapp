//! Theming and styling for Asgard Mail

use gtk4::prelude::*;
use gtk4::CssProvider;

/// Theme manager for Asgard Mail
pub struct ThemeManager {
    /// CSS provider
    css_provider: CssProvider,
}

impl ThemeManager {
    /// Create a new theme manager
    pub fn new() -> Self {
        Self {
            css_provider: CssProvider::new(),
        }
    }

    /// Load the default theme
    pub fn load_default_theme(&self) -> Result<(), Box<dyn std::error::Error>> {
        let css = include_str!("../styles/asgard-mail.css");
        self.css_provider.load_from_data(css);
        
        gtk4::style_context_add_provider_for_display(
            &gtk4::gdk::Display::default().unwrap(),
            &self.css_provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
        
        Ok(())
    }

    /// Load a custom theme
    pub fn load_custom_theme(&self, css_content: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.css_provider.load_from_data(css_content);
        
        gtk4::style_context_add_provider_for_display(
            &gtk4::gdk::Display::default().unwrap(),
            &self.css_provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
        
        Ok(())
    }
}

impl Default for ThemeManager {
    fn default() -> Self {
        Self::new()
    }
}
