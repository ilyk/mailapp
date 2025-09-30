//! Reusable UI widgets for Asgard Mail

pub mod mailbox_tree;
pub mod message_list;
pub mod message_view;
pub mod message_card;
pub mod search_bar;
pub mod status_bar;

pub use mailbox_tree::MailboxTree;
pub use message_list::MessageList;
pub use message_view::MessageView;
// pub use message_card::MessageCard;
pub use search_bar::SearchBar;
pub use status_bar::StatusBar;
