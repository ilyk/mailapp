use gtk4::CssProvider;

pub fn load_css() {
    let provider = CssProvider::new();
    provider.load_from_data(include_str!("asgard.css"));
    
    if let Some(display) = gtk4::gdk::Display::default() {
        gtk4::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
}