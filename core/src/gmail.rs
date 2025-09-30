//! Gmail-specific functionality for Asgard Mail

pub mod labels;
pub mod xoauth2;

pub use labels::GmailLabels;
pub use xoauth2::XOAUTH2;

/// Gmail-specific constants and utilities
pub mod constants {
    /// Gmail IMAP server
    pub const IMAP_HOST: &str = "imap.gmail.com";
    pub const IMAP_PORT: u16 = 993;
    
    /// Gmail SMTP server
    pub const SMTP_HOST: &str = "smtp.gmail.com";
    pub const SMTP_PORT: u16 = 587;
    
    /// Gmail OAuth scopes
    pub const OAUTH_SCOPES: &[&str] = &[
        "https://www.googleapis.com/auth/gmail.readonly",
        "https://www.googleapis.com/auth/gmail.send",
        "https://www.googleapis.com/auth/gmail.modify",
        "https://www.googleapis.com/auth/gmail.labels",
    ];
    
    /// Gmail OAuth redirect URI
    pub const OAUTH_REDIRECT_URI: &str = "http://127.0.0.1:8080";
    
    /// Gmail special folders
    pub const INBOX: &str = "INBOX";
    pub const SENT: &str = "[Gmail]/Sent Mail";
    pub const DRAFTS: &str = "[Gmail]/Drafts";
    pub const TRASH: &str = "[Gmail]/Trash";
    pub const SPAM: &str = "[Gmail]/Spam";
    pub const ALL_MAIL: &str = "[Gmail]/All Mail";
    pub const STARRED: &str = "[Gmail]/Starred";
    pub const IMPORTANT: &str = "[Gmail]/Important";
    
    /// Gmail system labels
    pub const SYSTEM_LABELS: &[&str] = &[
        INBOX,
        SENT,
        DRAFTS,
        TRASH,
        SPAM,
        ALL_MAIL,
        STARRED,
        IMPORTANT,
    ];
}

/// Gmail folder to label mapping
pub fn folder_to_label(folder_name: &str) -> String {
    match folder_name {
        constants::INBOX => "INBOX".to_string(),
        constants::SENT => "SENT".to_string(),
        constants::DRAFTS => "DRAFTS".to_string(),
        constants::TRASH => "TRASH".to_string(),
        constants::SPAM => "SPAM".to_string(),
        constants::ALL_MAIL => "ALL_MAIL".to_string(),
        constants::STARRED => "STARRED".to_string(),
        constants::IMPORTANT => "IMPORTANT".to_string(),
        _ => folder_name.to_string(),
    }
}

/// Label to Gmail folder mapping
pub fn label_to_folder(label_name: &str) -> String {
    match label_name {
        "INBOX" => constants::INBOX.to_string(),
        "SENT" => constants::SENT.to_string(),
        "DRAFTS" => constants::DRAFTS.to_string(),
        "TRASH" => constants::TRASH.to_string(),
        "SPAM" => constants::SPAM.to_string(),
        "ALL_MAIL" => constants::ALL_MAIL.to_string(),
        "STARRED" => constants::STARRED.to_string(),
        "IMPORTANT" => constants::IMPORTANT.to_string(),
        _ => label_name.to_string(),
    }
}

/// Check if a folder name is a Gmail system folder
pub fn is_system_folder(folder_name: &str) -> bool {
    constants::SYSTEM_LABELS.contains(&folder_name)
}

/// Check if a label name is a Gmail system label
pub fn is_system_label(label_name: &str) -> bool {
    matches!(label_name, "INBOX" | "SENT" | "DRAFTS" | "TRASH" | "SPAM" | "ALL_MAIL" | "STARRED" | "IMPORTANT")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_folder_to_label_mapping() {
        assert_eq!(folder_to_label(constants::INBOX), "INBOX");
        assert_eq!(folder_to_label(constants::SENT), "SENT");
        assert_eq!(folder_to_label(constants::DRAFTS), "DRAFTS");
        assert_eq!(folder_to_label(constants::TRASH), "TRASH");
        assert_eq!(folder_to_label("Custom Label"), "Custom Label");
    }

    #[test]
    fn test_label_to_folder_mapping() {
        assert_eq!(label_to_folder("INBOX"), constants::INBOX);
        assert_eq!(label_to_folder("SENT"), constants::SENT);
        assert_eq!(label_to_folder("DRAFTS"), constants::DRAFTS);
        assert_eq!(label_to_folder("TRASH"), constants::TRASH);
        assert_eq!(label_to_folder("Custom Label"), "Custom Label");
    }

    #[test]
    fn test_system_folder_detection() {
        assert!(is_system_folder(constants::INBOX));
        assert!(is_system_folder(constants::SENT));
        assert!(is_system_folder(constants::DRAFTS));
        assert!(!is_system_folder("Custom Folder"));
    }

    #[test]
    fn test_system_label_detection() {
        assert!(is_system_label("INBOX"));
        assert!(is_system_label("SENT"));
        assert!(is_system_label("DRAFTS"));
        assert!(!is_system_label("Custom Label"));
    }
}
