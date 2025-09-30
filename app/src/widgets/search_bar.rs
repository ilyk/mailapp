//! Search bar widget for email search functionality

use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Entry, Button, Orientation};
// use asgard_core::search::TantivySearchIndex;
// use std::sync::Arc;
// use tokio::sync::Mutex;

/// Search bar widget
pub struct SearchBar {
    /// Main widget container
    pub widget: GtkBox,
    /// Search entry
    entry: Entry,
    /// Search button
    search_button: Button,
    /// Clear button
    clear_button: Button,
    // Search index
    // search_index: Arc<Mutex<TantivySearchIndex>>,
}

impl SearchBar {
    /// Create a new search bar widget
    pub fn new() -> Self {
        let widget = GtkBox::new(Orientation::Horizontal, 8);
        widget.set_margin_start(12);
        widget.set_margin_end(12);
        widget.set_margin_top(8);
        widget.set_margin_bottom(8);
        widget.add_css_class("search-bar");
        
        // Search entry
        let entry = Entry::new();
        entry.set_placeholder_text(Some("Search messages..."));
        entry.add_css_class("search-entry");
        
        // Search button
        let search_button = Button::from_icon_name("system-search-symbolic");
        search_button.set_tooltip_text(Some("Search"));
        search_button.add_css_class("flat");
        
        // Clear button
        let clear_button = Button::from_icon_name("edit-clear-symbolic");
        clear_button.set_tooltip_text(Some("Clear search"));
        clear_button.add_css_class("flat");
        clear_button.set_visible(false);
        
        // Connect search functionality
        let entry_clone = entry.clone();
        let clear_button_clone = clear_button.clone();
        search_button.connect_clicked(move |_| {
            let query = entry_clone.text();
            if !query.is_empty() {
                println!("Searching for: {}", query);
                clear_button_clone.set_visible(true);
            }
        });
        
        // Connect clear functionality
        let entry_clone = entry.clone();
        let clear_button_clone = clear_button.clone();
        clear_button.connect_clicked(move |_| {
            entry_clone.set_text("");
            clear_button_clone.set_visible(false);
            println!("Search cleared");
        });
        
        // Connect enter key to search
        let search_button_clone = search_button.clone();
        entry.connect_activate(move |_| {
            search_button_clone.emit_clicked();
        });
        
        // Show clear button when text is entered
        let clear_button_clone = clear_button.clone();
        entry.connect_changed(move |entry| {
            let has_text = !entry.text().is_empty();
            clear_button_clone.set_visible(has_text);
        });
        
        widget.append(&entry);
        widget.append(&search_button);
        widget.append(&clear_button);
        
        // For now, create a dummy search index
        // In a real implementation, this would be passed from the main window
        // let search_index = Arc::new(Mutex::new(
        //     TantivySearchIndex::new("/tmp/asgard_search").unwrap_or_else(|_| {
        //         // Create a dummy index if we can't create a real one
        //         TantivySearchIndex::new("/tmp/asgard_search_dummy").unwrap()
        //     })
        // ));
        
        Self {
            widget,
            entry,
            search_button,
            clear_button,
        }
    }
    
    /// Get the current search query
    pub fn get_query(&self) -> String {
        self.entry.text().to_string()
    }
    
    /// Set the search query
    pub fn set_query(&self, query: &str) {
        self.entry.set_text(query);
    }
    
    /// Clear the search
    pub fn clear(&self) {
        self.entry.set_text("");
        self.clear_button.set_visible(false);
    }
    
    /// Perform a search
    pub async fn search(&self, query: &str) -> Result<Vec<uuid::Uuid>, Box<dyn std::error::Error>> {
        if query.is_empty() {
            return Ok(Vec::new());
        }
        
        // For demo purposes, return empty results
        // In a real implementation, this would use the search index
        println!("Searching for: {}", query);
        Ok(Vec::new())
    }
}

impl Clone for SearchBar {
    fn clone(&self) -> Self {
        Self {
            widget: self.widget.clone(),
            entry: self.entry.clone(),
            search_button: self.search_button.clone(),
            clear_button: self.clear_button.clone(),
        }
    }
}