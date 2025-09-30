//! Test the threading algorithm

use email_backend::EmailMessage;
use crate::thread_helpers::{group_into_threads, normalize_subject};
use time::{Date, Month, Time, UtcOffset, OffsetDateTime};
use uuid::Uuid;

fn create_test_message(
    uid: &str,
    subject: &str,
    from: &str,
    date: OffsetDateTime,
    message_id: Option<&str>,
    in_reply_to: Option<&str>,
    references: Vec<&str>,
    server_thread_id: Option<&str>,
) -> EmailMessage {
    let mut msg = EmailMessage::new(
        Uuid::new_v4(),
        subject.to_string(),
        from.to_string(),
        vec!["demo@example.com".to_string()],
        "Test message body".to_string(),
        "INBOX".to_string(),
    );
    msg.date = date;
    msg.message_id = message_id.map(|s| s.to_string());
    msg.in_reply_to = in_reply_to.map(|s| s.to_string());
    msg.references = references.into_iter().map(|s| s.to_string()).collect();
    msg.server_thread_id = server_thread_id.map(|s| s.to_string());
    msg
}

fn test_date(day: u8) -> OffsetDateTime {
    OffsetDateTime::new_in_offset(
        Date::from_calendar_date(2024, Month::March, day).unwrap(),
        Time::from_hms(10, 0, 0).unwrap(),
        UtcOffset::from_hms(0, 0, 0).unwrap(),
    )
}

#[test]
fn test_gmail_thread_id_wins() {
    let messages = vec![
        create_test_message(
            "1", "Subject A", "alice@example.com", test_date(1),
            Some("<msg1@example.com>"), None, vec![], Some("gmail-thread-123")
        ),
        create_test_message(
            "2", "Subject B", "bob@example.com", test_date(2),
            Some("<msg2@example.com>"), None, vec![], Some("gmail-thread-123")
        ),
    ];

    let threads = group_into_threads(messages);
    assert_eq!(threads.len(), 1);
    assert_eq!(threads[0].messages.len(), 2);
    assert_eq!(threads[0].id, "gmail-thread-123");
}

#[test]
fn test_jwz_chain() {
    let messages = vec![
        create_test_message(
            "1", "Original", "alice@example.com", test_date(1),
            Some("<msg1@example.com>"), None, vec![], None
        ),
        create_test_message(
            "2", "Re: Original", "bob@example.com", test_date(2),
            Some("<msg2@example.com>"), Some("<msg1@example.com>"), vec!["<msg1@example.com>"], None
        ),
        create_test_message(
            "3", "Re: Original", "alice@example.com", test_date(3),
            Some("<msg3@example.com>"), Some("<msg2@example.com>"), vec!["<msg1@example.com>", "<msg2@example.com>"], None
        ),
    ];

    let threads = group_into_threads(messages);
    assert_eq!(threads.len(), 1);
    assert_eq!(threads[0].messages.len(), 3);
    assert_eq!(threads[0].messages[0].subject, "Original"); // oldest first
    assert_eq!(threads[0].messages[1].subject, "Re: Original");
    assert_eq!(threads[0].messages[2].subject, "Re: Original"); // newest last
}

#[test]
fn test_subject_normalization() {
    assert_eq!(normalize_subject("Re: Re: Original Subject"), "original subject");
    assert_eq!(normalize_subject("[List] Re: Fwd: Important"), "important");
    assert_eq!(normalize_subject("  Re:   Multiple   Spaces  "), "multiple spaces");
}

#[test]
fn test_thread_sorting() {
    let messages = vec![
        create_test_message(
            "1", "Older Thread", "alice@example.com", test_date(1),
            Some("<msg1@example.com>"), None, vec![], None
        ),
        create_test_message(
            "2", "Newer Thread", "bob@example.com", test_date(3),
            Some("<msg2@example.com>"), None, vec![], None
        ),
    ];

    let threads = group_into_threads(messages);
    assert_eq!(threads.len(), 2);
    assert_eq!(threads[0].messages[0].subject, "Newer Thread"); // newer thread first
    assert_eq!(threads[1].messages[0].subject, "Older Thread"); // older thread second
}

#[test]
fn test_orphan_messages() {
    let messages = vec![
        create_test_message(
            "1", "Orphan", "alice@example.com", test_date(1),
            Some("<msg1@example.com>"), Some("<nonexistent@example.com>"), vec![], None
        ),
    ];

    let threads = group_into_threads(messages);
    assert_eq!(threads.len(), 1);
    assert_eq!(threads[0].messages.len(), 1);
    assert_eq!(threads[0].messages[0].subject, "Orphan");
}
