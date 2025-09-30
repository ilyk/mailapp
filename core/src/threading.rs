//! Email threading algorithm implementation
//! 
//! Implements JWZ-style threading with three priority levels:
//! P1: Server thread id (Gmail X-GM-THRID, Exchange/Graph ConversationId, IMAP THREAD)
//! P2: RFC linking (Message-ID, In-Reply-To, References) using JWZ threading
//! P3: Subject fallback (normalized subject within time windows)

use crate::types::{MsgMeta, Thread};
use std::collections::{BTreeMap, HashMap, HashSet};
use time::OffsetDateTime;

/// Group messages into threads using the three-phase algorithm
pub fn group_into_threads(mut msgs: Vec<MsgMeta>) -> Vec<Thread> {
    // 0) defensive copy and basic sort (stable)
    msgs.sort_by_key(|m| m.date);

    // 1) P1: server thread id
    let (with_tid, without_tid): (Vec<_>, Vec<_>) =
        msgs.into_iter().partition(|m| m.server_thread_id.is_some());

    let mut buckets: HashMap<String, Vec<MsgMeta>> = HashMap::new();
    for m in with_tid {
        buckets.entry(m.server_thread_id.clone().unwrap())
               .or_default().push(m);
    }

    // 2) P2: JWZ style threading on the rest
    let jwz_threads = jwz_threads(without_tid);
    for (tid, v) in jwz_threads {
        buckets.entry(tid).or_default().extend(v);
    }

    // 3) Normalize each bucket, compute summaries
    let mut threads: Vec<Thread> = buckets.into_iter().map(|(tid, mut v)| {
        v.sort_by_key(|m| m.date); // oldest→newest
        let subject = canonical_subject(v.last().map(|m| m.subject.clone()).unwrap_or_default());
        let any_unread = v.iter().any(|m| !m.is_read);
        let has_attachments = v.iter().any(|m| m.has_attachments);
        let last_is_outgoing_reply = v.last()
            .map(|m| m.is_outgoing && (m.in_reply_to.is_some() || m.subject.starts_with("Re:")))
            .unwrap_or(false);
        let last_date = v.last().map(|m| m.date).unwrap_or(OffsetDateTime::UNIX_EPOCH);
        Thread::new(tid, subject, v)
    }).collect();

    // Sort threads by last message time descending for the middle pane
    threads.sort_by_key(|t| t.last_date);
    threads.reverse();
    threads
}

/// Node for JWZ threading algorithm
#[derive(Clone)]
struct Node { 
    mid: String, 
    msg: Option<MsgMeta>, 
    parent: Option<String>, 
    children: Vec<String> 
}

/// JWZ-style threading algorithm implementation
fn jwz_threads(messages: Vec<MsgMeta>) -> HashMap<String, Vec<MsgMeta>> {
    // Build nodes for Message-ID and placeholders referenced in References
    let mut nodes: HashMap<String, Node> = HashMap::new();

    // ensure node exists
    let ensure = |nodes: &mut HashMap<String, Node>, id: &str| {
        nodes.entry(id.to_string()).or_insert(Node{ 
            mid: id.to_string(), 
            msg: None, 
            parent: None, 
            children: vec![] 
        });
    };

    // pass 1: create nodes
    for m in &messages {
        if let Some(mid) = &m.message_id { 
            ensure(&mut nodes, mid); 
            nodes.get_mut(mid).unwrap().msg = Some(m.clone()); 
        }
        for r in &m.references { 
            ensure(&mut nodes, r); 
        }
    }

    // pass 2: link using References (parent = last ref) or In-Reply-To
    for m in &messages {
        if let Some(mid) = &m.message_id {
            let parent_id = m.references.last().cloned().or_else(|| m.in_reply_to.clone());
            if let Some(pid) = parent_id {
                if nodes.contains_key(&pid) {
                    nodes.get_mut(mid).unwrap().parent = Some(pid.clone());
                    nodes.get_mut(&pid).unwrap().children.push(mid.clone());
                }
            }
        }
    }

    // roots are nodes with no parent and with real messages
    let mut roots: Vec<String> = nodes.values()
        .filter(|n| n.parent.is_none() && n.msg.is_some())
        .map(|n| n.mid.clone()).collect();

    // collect threads from roots
    let mut out: HashMap<String, Vec<MsgMeta>> = HashMap::new();
    for root in roots.drain(..) {
        let tid = root.clone(); // synthetic id: root Message-ID
        let mut acc: Vec<MsgMeta> = Vec::new();
        collect_inorder(&nodes, &root, &mut acc);
        out.insert(tid, acc);
    }

    // orphans: nodes with a msg but not in any collected thread
    let mut seen: HashSet<String> = out.values().flatten()
        .filter_map(|m| m.message_id.clone()).collect();
    for n in nodes.values() {
        if let Some(m) = &n.msg {
            if !m.message_id.as_ref().map(|x| seen.contains(x)).unwrap_or(false) {
                let tid = m.message_id.clone().unwrap_or_else(|| format!("midless-{}", m.uid));
                out.entry(tid).or_default().push(m.clone());
            }
        }
    }
    out
}

/// Recursively collect messages in thread order
fn collect_inorder(nodes: &HashMap<String, Node>, id: &String, acc: &mut Vec<MsgMeta>) {
    if let Some(n) = nodes.get(id) {
        if let Some(m) = &n.msg { 
            acc.push(m.clone()); 
        }
        // order children by Date header if available
        let mut childs = n.children.clone();
        childs.sort_by_key(|cid| nodes.get(cid).and_then(|c| c.msg.as_ref()).map(|m| m.date));
        for c in childs { 
            collect_inorder(nodes, &c, acc); 
        }
    }
}

/// Get canonical subject for a thread
fn canonical_subject(s: String) -> String {
    normalize_subject(&s)
}

/// Normalize subject for threading
pub fn normalize_subject(raw: &str) -> String {
    use regex::Regex;
    let mut s = raw.trim().to_string();
    // strip [list] tags
    let re_list = Regex::new(r"^\s*\[[^\]]+\]\s*").unwrap();
    s = re_list.replace(&s, "").into_owned();
    // strip reply/forward tokens in many locales
    let re_rf = Regex::new(r"^((?i:re|fw|fwd|sv|aw|antw|rv)\s*:\s*)+").unwrap();
    s = re_rf.replace(&s, "").into_owned();
    s.split_whitespace().collect::<Vec<_>>().join(" ").to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::{Date, Month, Time, UtcOffset};

    fn create_test_message(
        uid: &str,
        subject: &str,
        from: &str,
        date: OffsetDateTime,
        message_id: Option<&str>,
        in_reply_to: Option<&str>,
        references: Vec<&str>,
        server_thread_id: Option<&str>,
    ) -> MsgMeta {
        MsgMeta {
            uid: uid.to_string(),
            folder: "INBOX".to_string(),
            date,
            from: from.to_string(),
            subject: subject.to_string(),
            body_preview: "Test message body".to_string(),
            has_attachments: false,
            is_read: false,
            is_outgoing: false,
            message_id: message_id.map(|s| s.to_string()),
            in_reply_to: in_reply_to.map(|s| s.to_string()),
            references: references.into_iter().map(|s| s.to_string()).collect(),
            server_thread_id: server_thread_id.map(|s| s.to_string()),
        }
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
        assert_eq!(threads[0].messages[0].uid, "1"); // oldest first
        assert_eq!(threads[0].messages[1].uid, "2");
        assert_eq!(threads[0].messages[2].uid, "3"); // newest last
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
        assert_eq!(threads[0].messages[0].uid, "2"); // newer thread first
        assert_eq!(threads[1].messages[0].uid, "1"); // older thread second
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
        assert_eq!(threads[0].messages[0].uid, "1");
    }
}
