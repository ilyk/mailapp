use gtk4::prelude::*;
use gtk4::*;
use time::OffsetDateTime;

#[derive(Clone)]
pub struct MessageHeader {
    pub root: Box,
}

impl MessageHeader {
    pub fn new(sender: &str, subject: &str, recipients: &[String], date: &OffsetDateTime) -> Self {
        let root = Box::new(Orientation::Vertical, 6);
        root.add_css_class("message-header");
        
        // Top row: Avatar + From/Subject + To/CC chips + Date
        let top = Box::new(Orientation::Horizontal, 8);
        
        // Avatar (28px circle)
        let avatar = Picture::for_resource("/com/asgard/mail/icons/avatar-default.png");
        avatar.set_size_request(28, 28);
        avatar.add_css_class("avatar");
        
        // Names and subject
        let names = Box::new(Orientation::Vertical, 2);
        
        let subject_label = Label::builder()
            .label(subject)
            .xalign(0.0)
            .build();
        subject_label.add_css_class("header-subject");
        
        let from_label = Label::builder()
            .label(sender)
            .xalign(0.0)
            .build();
        from_label.add_css_class("header-from");
        
        names.append(&subject_label);
        names.append(&from_label);
        
        // Recipient chips
        let chips_container = Box::new(Orientation::Horizontal, 4);
        for recipient in recipients.iter().take(3) { // Show max 3 recipients
            let chip = Label::builder()
                .label(recipient)
                .build();
            chip.add_css_class("recipient-chip");
            chips_container.append(&chip);
        }
        if recipients.len() > 3 {
            let more_chip = Label::builder()
                .label(&format!("+{} more", recipients.len() - 3))
                .build();
            more_chip.add_css_class("recipient-chip");
            chips_container.append(&more_chip);
        }
        
        // Spacer
        let spacer = Box::new(Orientation::Horizontal, 0);
        spacer.set_hexpand(true);
        
        // Date
        let date_str = format_human_date(date);
        let date_label = Label::builder()
            .label(&date_str)
            .xalign(1.0)
            .build();
        date_label.add_css_class("header-date");
        
        top.append(&avatar);
        top.append(&names);
        top.append(&chips_container);
        top.append(&spacer);
        top.append(&date_label);
        
        // Action buttons row
        let actions = Self::create_action_row();
        
        root.append(&top);
        root.append(&actions);
        
        Self { root }
    }
    
    fn create_action_row() -> Box {
        let actions = Box::new(Orientation::Horizontal, 4);
        actions.add_css_class("action-row");
        
        let reply = Button::from_icon_name("mail-reply-sender-symbolic");
        reply.set_tooltip_text(Some("Reply"));
        reply.add_css_class("flat");
        
        let reply_all = Button::from_icon_name("mail-reply-all-symbolic");
        reply_all.set_tooltip_text(Some("Reply All"));
        reply_all.add_css_class("flat");
        
        let forward = Button::from_icon_name("mail-forward-symbolic");
        forward.set_tooltip_text(Some("Forward"));
        forward.add_css_class("flat");
        
        let archive = Button::from_icon_name("mail-archive-symbolic");
        archive.set_tooltip_text(Some("Archive"));
        archive.add_css_class("flat");
        
        let trash = Button::from_icon_name("user-trash-symbolic");
        trash.set_tooltip_text(Some("Delete"));
        trash.add_css_class("flat");
        
        let move_to = MenuButton::builder()
            .icon_name("mail-move-symbolic")
            .build();
        move_to.set_tooltip_text(Some("Move to"));
        move_to.add_css_class("flat");
        
        let flag = ToggleButton::from_icon_name("starred-symbolic");
        flag.set_tooltip_text(Some("Flag"));
        flag.add_css_class("flat");
        
        actions.append(&reply);
        actions.append(&reply_all);
        actions.append(&forward);
        actions.append(&archive);
        actions.append(&trash);
        actions.append(&move_to);
        actions.append(&flag);
        
        actions
    }
    
    pub fn widget(&self) -> &Widget {
        self.root.upcast_ref()
    }
}

fn format_human_date(date: &OffsetDateTime) -> String {
    let now = OffsetDateTime::now_utc();
    let duration = now - *date;
    
    if duration.is_negative() {
        "Future".to_string()
    } else {
        let days = duration.whole_days();
        let hours = duration.whole_hours();
        let minutes = duration.whole_minutes();
        
        if days > 7 {
            // Show date for older messages
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
        } else if days > 0 {
            format!("{}d", days)
        } else if hours > 0 {
            format!("{}h", hours)
        } else if minutes > 0 {
            format!("{}m", minutes)
        } else {
            "Now".to_string()
        }
    }
}
