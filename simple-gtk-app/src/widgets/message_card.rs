use gtk4::prelude::*;
use gtk4::*;
use time::OffsetDateTime;

#[derive(Clone)]
pub struct MessageCard {
    pub root: Box,
    pub header: Box,
    pub revealer: Revealer,
    pub collapsed_revealer: Revealer,
    pub chevron: Label,
    pub body_label: Label,
}

impl MessageCard {
    pub fn new_collapsible(
        sender: &str,
        subject: &str,
        to: &[String],
        cc: &[String],
        bcc: &[String],
        date: &OffsetDateTime,
        body_text: &str,
        has_attachments: bool,
        expanded: bool,
    ) -> Self {
        let root = Box::new(Orientation::Vertical, 8);
        root.add_css_class("msg-card");
        root.set_vexpand(false);
        root.set_hexpand(false);

        let (header, chevron) = Self::build_header(sender, subject, to, cc, bcc, date, has_attachments, expanded);
        
        // Create body content - show full text when expanded, first line when collapsed
        let display_text = if expanded {
            body_text.to_string()
        } else {
            // Show only first line when collapsed
            body_text.lines().next().unwrap_or("").to_string()
        };
        
        let (body, body_label) = Self::build_body_webview(&display_text);
        
        // Add CSS classes for styling control
        if expanded {
            body.add_css_class("expanded");
            body.remove_css_class("collapsed");
        } else {
            body.add_css_class("collapsed");
            body.remove_css_class("expanded");
        }

        // Single revealer for the body content - always show content, let CSS control styling
        let revealer = Revealer::builder()
            .transition_type(RevealerTransitionType::SlideDown)
            .reveal_child(true) // Always show the content
            .build();
        revealer.set_child(Some(&body));
        
        // Collapsed preview is no longer needed
        let collapsed_revealer = Revealer::new(); // Placeholder for compatibility
        

        let separator = Separator::new(Orientation::Horizontal);
        separator.add_css_class("card-sep");

        // Make header clickable to toggle expansion
        header.set_cursor_from_name(Some("pointer"));
        let chevron_clone = chevron.clone();
        let body_clone = body.clone();
        let body_label_clone = body_label.clone();
        let body_text_clone = body_text.to_string();
        
        // Add a gesture to handle clicks on header
        let header_gesture = gtk4::GestureClick::new();
        header_gesture.connect_pressed(move |_, _, _, _| {
            // Check current state by looking at CSS classes
            let is_expanded = body_clone.has_css_class("expanded");
            
            // Toggle CSS classes and update content
            if is_expanded {
                body_clone.remove_css_class("expanded");
                body_clone.add_css_class("collapsed");
                chevron_clone.set_label("▶");
                
                // Update body content to show only first line
                let first_line = body_text_clone.lines().next().unwrap_or("");
                body_label_clone.set_label(first_line);
            } else {
                body_clone.remove_css_class("collapsed");
                body_clone.add_css_class("expanded");
                chevron_clone.set_label("▼");
                
                // Update body content to show full text
                body_label_clone.set_label(&body_text_clone);
            }
        });
        header.add_controller(header_gesture);

        // Make body clickable to expand when collapsed
        body.set_cursor_from_name(Some("pointer"));
        let chevron_clone2 = chevron.clone();
        let body_clone2 = body.clone();
        let body_label_clone2 = body_label.clone();
        let body_text_clone2 = body_text.to_string();
        
        // Add a gesture to handle clicks on body
        let body_gesture = gtk4::GestureClick::new();
        body_gesture.connect_pressed(move |_, _, _, _| {
            // Check current state by looking at CSS classes
            let is_expanded = body_clone2.has_css_class("expanded");
            
            if !is_expanded { // Only expand if currently collapsed
                body_clone2.remove_css_class("collapsed");
                body_clone2.add_css_class("expanded");
                chevron_clone2.set_label("▼");
                
                // Update body content to show full text
                body_label_clone2.set_label(&body_text_clone2);
            }
        });
        body.add_controller(body_gesture);

        root.append(&header);
        root.append(&separator);
        root.append(&revealer);

        Self { root, header, revealer, collapsed_revealer, chevron, body_label }
    }

    pub fn new(
        sender: &str,
        subject: &str,
        recipients: &[String],
        date: &OffsetDateTime,
        body: &str,
    ) -> Self {
        let outer = Box::new(Orientation::Vertical, 8);
        outer.add_css_class("msg-card");

        // Header grid
        let grid = Grid::new();
        grid.set_column_spacing(8);
        grid.set_row_spacing(2);
        grid.set_column_homogeneous(false);

        // Avatar (36px circle with initials)
        let avatar = Self::create_avatar_widget(sender);
        avatar.add_css_class("avatar36");

        // Sender name (bold, 13-14px)
        let name_label = Label::builder()
            .label(sender)
            .xalign(0.0)
            .build();
        name_label.add_css_class("hdr-name");

        // Subject (regular 13px)
        let subject_label = Label::builder()
            .label(subject)
            .xalign(0.0)
            .build();
        subject_label.add_css_class("hdr-subject");

        // To recipients (muted 12px)
        let to_text = if recipients.is_empty() {
            "To: (no recipients)".to_string()
        } else {
            format!("To: {}", recipients.join(", "))
        };
        let to_label = Label::builder()
            .label(&to_text)
            .xalign(0.0)
            .build();
        to_label.add_css_class("hdr-to");

        // Date/Time (right-aligned, 12px)
        let date_str = Self::format_datetime_full(date);
        let when_label = Label::builder()
            .label(&date_str)
            .xalign(1.0)
            .build();
        when_label.add_css_class("hdr-when");

        // Attach to grid
        grid.attach(&avatar, 0, 0, 1, 3);  // spans 3 rows
        grid.attach(&name_label, 1, 0, 1, 1);
        grid.attach(&subject_label, 1, 1, 1, 1);
        grid.attach(&to_label, 1, 2, 1, 1);
        grid.attach(&when_label, 2, 0, 1, 1);

        outer.append(&grid);

        // Separator
        let sep = Separator::new(Orientation::Horizontal);
        sep.add_css_class("card-sep");
        outer.append(&sep);

        // Message body
        let body_label = Label::builder()
            .label(body)
            .xalign(0.0)
            .yalign(0.0)
            .wrap(true)
            .justify(Justification::Left)
            .build();
        body_label.add_css_class("msg-body");
        body_label.set_vexpand(true);
        body_label.set_hexpand(true);

        outer.append(&body_label);

        Self { 
            root: outer, 
            header: Box::new(Orientation::Horizontal, 0), // placeholder
            revealer: Revealer::new(), // placeholder
            collapsed_revealer: Revealer::new(), // placeholder
            chevron: Label::new(None), // placeholder
            body_label: Label::new(None), // placeholder
        }
    }

    fn create_avatar_widget(sender: &str) -> Widget {
        // Create a simple colored box with initials for now
        let drawing = DrawingArea::new();
        drawing.set_content_width(36);
        drawing.set_content_height(36);
        drawing.add_css_class("avatar36");
        
        // Clone sender string to move into closure
        let sender_owned = sender.to_string();
        
        // Set a background color based on sender name hash
        let color = Self::get_color_for_name(&sender_owned);
        drawing.set_draw_func(move |_, cr, w, h| {
            // Draw circle background
            cr.set_source_rgba(color.0, color.1, color.2, 1.0);
            cr.arc(w as f64 / 2.0, h as f64 / 2.0, (w.min(h) as f64) / 2.0, 0.0, 2.0 * std::f64::consts::PI);
            cr.fill().unwrap();
            
            // Draw initials
            cr.set_source_rgba(1.0, 1.0, 1.0, 1.0); // White text
            cr.select_font_face("Sans", gtk4::cairo::FontSlant::Normal, gtk4::cairo::FontWeight::Bold);
            cr.set_font_size(14.0);
            
            let initials = Self::get_initials(&sender_owned);
            let extents = cr.text_extents(&initials).unwrap();
            let x = (w as f64 - extents.width()) / 2.0;
            let y = (h as f64 + extents.height()) / 2.0;
            cr.move_to(x, y);
            cr.show_text(&initials).unwrap();
        });
        
        drawing.upcast()
    }

    fn get_color_for_name(name: &str) -> (f64, f64, f64) {
        // Simple hash-based color generation
        let hash = name.chars().fold(0u32, |acc, c| acc.wrapping_mul(31).wrapping_add(c as u32));
        let r = ((hash & 0xFF) as f64) / 255.0;
        let g = (((hash >> 8) & 0xFF) as f64) / 255.0;
        let b = (((hash >> 16) & 0xFF) as f64) / 255.0;
        (r, g, b)
    }

    fn get_initials(name: &str) -> String {
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

    fn format_datetime_full(date: &OffsetDateTime) -> String {
        // Format as "September 21, 2025 at 4:24 PM"
        match date.month() {
            time::Month::January => format!("January {}, {} at {}", date.day(), date.year(), Self::format_time_12h(date)),
            time::Month::February => format!("February {}, {} at {}", date.day(), date.year(), Self::format_time_12h(date)),
            time::Month::March => format!("March {}, {} at {}", date.day(), date.year(), Self::format_time_12h(date)),
            time::Month::April => format!("April {}, {} at {}", date.day(), date.year(), Self::format_time_12h(date)),
            time::Month::May => format!("May {}, {} at {}", date.day(), date.year(), Self::format_time_12h(date)),
            time::Month::June => format!("June {}, {} at {}", date.day(), date.year(), Self::format_time_12h(date)),
            time::Month::July => format!("July {}, {} at {}", date.day(), date.year(), Self::format_time_12h(date)),
            time::Month::August => format!("August {}, {} at {}", date.day(), date.year(), Self::format_time_12h(date)),
            time::Month::September => format!("September {}, {} at {}", date.day(), date.year(), Self::format_time_12h(date)),
            time::Month::October => format!("October {}, {} at {}", date.day(), date.year(), Self::format_time_12h(date)),
            time::Month::November => format!("November {}, {} at {}", date.day(), date.year(), Self::format_time_12h(date)),
            time::Month::December => format!("December {}, {} at {}", date.day(), date.year(), Self::format_time_12h(date)),
        }
    }

    fn format_time_12h(date: &OffsetDateTime) -> String {
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

    pub fn set_expanded(&self, expanded: bool) {
        // Note: This method is not used in the current implementation
        // The expansion is controlled by the individual revealers
        self.chevron.set_label(if expanded { "▼" } else { "▶" });
    }

    pub fn widget(&self) -> &Widget {
        self.root.upcast_ref()
    }

    fn build_header(sender: &str, subject: &str, to: &[String], cc: &[String], bcc: &[String], date: &OffsetDateTime, has_attachments: bool, expanded: bool) -> (Box, Label) {
        let header = Box::new(Orientation::Horizontal, 8);
        
        // Avatar (36px circle with initials)
        let avatar = Self::create_avatar_widget(sender);
        avatar.add_css_class("avatar36");

        // Sender name (bold, 13-14px)
        let name_label = Label::builder()
            .label(sender)
            .xalign(0.0)
            .build();
        name_label.add_css_class("hdr-name");

        // Subject (regular, 13px)
        let subject_label = Label::builder()
            .label(subject)
            .xalign(0.0)
            .build();
        subject_label.add_css_class("hdr-subject");

        // Build recipient display with proper formatting
        let (recipient_container, has_multiple_types) = Self::build_recipient_display(to, cc, bcc);

        // Date/time (right-aligned, 12px)
        let when_label = Label::builder()
            .label(&Self::format_datetime_full(date))
            .xalign(1.0)
            .build();
        when_label.add_css_class("hdr-when");

        // Attachment icon (if present)
        let attachment_icon = if has_attachments {
            let icon = Image::from_icon_name("mail-attachment-symbolic");
            icon.set_icon_size(IconSize::Normal);
            icon.add_css_class("attachment-icon");
            Some(icon)
        } else {
            None
        };

        // Details link (if multiple recipient types)
        let details_link = if has_multiple_types {
            let link = Label::builder()
                .label("Details")
                .xalign(0.0)
                .build();
            link.add_css_class("details-link");
            
            // Add click handler to prevent event propagation to header
            let details_gesture = gtk4::GestureClick::new();
            details_gesture.connect_pressed(move |gesture, _, _, _| {
                // Stop event propagation to prevent header click handler
                gesture.set_state(gtk4::EventSequenceState::Claimed);
                // TODO: Add actual details expansion logic here
                println!("Details link clicked - should expand headers");
            });
            link.add_controller(details_gesture);
            
            Some(link)
        } else {
            None
        };

        // Add chevron to indicate expandable state
        let chevron = Label::builder()
            .label(if expanded { "▼" } else { "▶" })
            .xalign(0.5)
            .yalign(0.5)
            .build();
        chevron.add_css_class("hdr-chevron");

        // Layout: Avatar | Name/Subject/Recipients | Spacer | Date/Attachment | Details | Chevron
        let left_content = Box::new(Orientation::Vertical, 2);
        left_content.append(&name_label);
        left_content.append(&subject_label);
        
        // Recipient line with Details link on the right
        let recipient_line = Box::new(Orientation::Horizontal, 0);
        recipient_line.append(&recipient_container);
        recipient_line.set_hexpand(true);
        
        left_content.append(&recipient_line);

        let spacer = Box::new(Orientation::Horizontal, 0);
        spacer.set_hexpand(true);

        let right_content = Box::new(Orientation::Vertical, 2);
        right_content.add_css_class("right_content");
        right_content.set_halign(Align::End);
        
        // Row 1: Date/time (always present)
        right_content.append(&when_label);
        
        // Row 2: Attachment icon (always present, conditionally visible)
        let attachment_icon_widget = if let Some(ref icon) = attachment_icon {
            icon.clone()
        } else {
            let empty_icon = Image::from_icon_name("mail-attachment-symbolic");
            empty_icon.set_icon_size(IconSize::Normal);
            empty_icon.add_css_class("attachment-icon");
            empty_icon
        };
        attachment_icon_widget.set_halign(Align::End);
        if attachment_icon.is_none() {
            attachment_icon_widget.set_visible(false);
        }
        right_content.append(&attachment_icon_widget);

        // Row 3: Details link (always present, conditionally visible)
        let details_link_widget = if let Some(ref link) = details_link {
            link.clone()
        } else {
            let empty_link = Label::builder()
                .label("Details")
                .xalign(1.0)
                .build();
            empty_link.add_css_class("details-link");
            empty_link
        };
        details_link_widget.set_halign(Align::End);
        if details_link.is_none() {
            details_link_widget.set_visible(false);
        }
        right_content.append(&details_link_widget);
        
        header.append(&avatar);
        header.append(&left_content);
        header.append(&spacer);
        header.append(&right_content);
        header.append(&chevron);

        (header, chevron)
    }

    fn build_body_webview(body_text: &str) -> (Box, Label) {
        let body_container = Box::new(Orientation::Vertical, 0);
        body_container.add_css_class("msg-body");
        body_container.set_vexpand(false);
        body_container.set_hexpand(false);
        
        let body_label = Label::builder()
            .label(body_text)
            .xalign(0.0)
            .yalign(0.0)
            .wrap(true)
            .justify(Justification::Left)
            .build();
        body_label.add_css_class("msg-body");
        body_label.set_hexpand(false);
        
        body_container.append(&body_label);
        (body_container, body_label)
    }

    fn build_collapsed_preview(body_text: &str) -> Box {
        let preview_container = Box::new(Orientation::Horizontal, 8);
        preview_container.add_css_class("msg-collapsed-preview");
        preview_container.set_vexpand(false);
        preview_container.set_hexpand(false);
        
        // Get first line of body text for preview
        let first_line = body_text.lines().next().unwrap_or("").trim();
        let preview_text = if first_line.is_empty() {
            "Click to expand message".to_string()
        } else {
            first_line.to_string()
        };
        
        let preview_label = Label::builder()
            .label(&preview_text)
            .xalign(0.0)
            .yalign(0.5)
            .wrap(false)
            .justify(Justification::Left)
            .build();
        preview_label.add_css_class("msg-collapsed-text");
        
        preview_container.append(&preview_label);
        preview_container
    }

    fn build_recipient_display(to: &[String], cc: &[String], bcc: &[String]) -> (Box, bool) {
        let container = Box::new(Orientation::Horizontal, 4);
        let mut has_multiple_types = false;

        // To recipients
        if !to.is_empty() {
            // "To:" label (bold)
            let to_label = Label::builder()
                .label("To:")
                .xalign(0.0)
                .build();
            to_label.add_css_class("hdr-recipients");
            to_label.add_css_class("recipient-label-bold");
            container.append(&to_label);
            
            // Recipients (normal weight)
            let to_recipients = Label::builder()
                .label(&format!(" {}", to.join(", ")))
                .xalign(0.0)
                .build();
            to_recipients.add_css_class("hdr-recipients");
            to_recipients.add_css_class("recipient-text");
            container.append(&to_recipients);
        }

        // CC recipients
        if !cc.is_empty() {
            // "Cc:" label (bold)
            let cc_label = Label::builder()
                .label("Cc:")
                .xalign(0.0)
                .build();
            cc_label.add_css_class("hdr-recipients");
            cc_label.add_css_class("recipient-label-bold");
            container.append(&cc_label);
            
            // Recipients (normal weight)
            let cc_recipients = Label::builder()
                .label(&format!(" {}", cc.join(", ")))
                .xalign(0.0)
                .build();
            cc_recipients.add_css_class("hdr-recipients");
            cc_recipients.add_css_class("recipient-text");
            container.append(&cc_recipients);
            has_multiple_types = true;
        }

        // BCC recipients (only show if there are multiple types)
        if !bcc.is_empty() {
            has_multiple_types = true;
        }

        // If no recipients, show placeholder
        if to.is_empty() && cc.is_empty() {
            let placeholder_label = Label::builder()
                .label("To: (no recipients)")
                .xalign(0.0)
                .build();
            placeholder_label.add_css_class("hdr-recipients");
            placeholder_label.add_css_class("recipient-label-bold");
            container.append(&placeholder_label);
        }

        (container, has_multiple_types)
    }

}
