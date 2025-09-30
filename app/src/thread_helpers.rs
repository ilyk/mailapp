//! Thread helpers for email threading

use asgard_core::message::Message;
use std::collections::{HashMap, HashSet};
use time::OffsetDateTime;

/// Simple thread representation for demo purposes
#[derive(Debug, Clone)]
pub struct Thread {
    pub id: String,
    pub subject: String,
    pub messages: Vec<Message>,
    pub last_date: OffsetDateTime,
    pub any_unread: bool,
    pub last_is_outgoing_reply: bool,
    pub has_attachments: bool,
}

impl Thread {
    pub fn new(id: String, subject: String, messages: Vec<Message>) -> Self {
        let any_unread = messages.iter().any(|m| !m.is_read());
        let has_attachments = messages.iter().any(|m| m.has_attachments());
        let last_date = messages.last().map(|m| m.headers.date.unwrap_or(OffsetDateTime::UNIX_EPOCH)).unwrap_or(OffsetDateTime::UNIX_EPOCH);
        let last_is_outgoing_reply = messages.last()
            .map(|m| is_outgoing_message(m) && is_reply_message(m))
            .unwrap_or(false);

        Self {
            id,
            subject,
            messages,
            last_date,
            any_unread,
            last_is_outgoing_reply,
            has_attachments,
        }
    }

    /// Get the last message in the thread
    pub fn last(&self) -> Option<&Message> {
        self.messages.last()
    }

    /// Get the number of messages in the thread
    pub fn count(&self) -> usize {
        self.messages.len()
    }

    /// Check if any message in the thread is unread
    pub fn any_unread(&self) -> bool {
        self.any_unread
    }

    /// Check if the thread has attachments
    pub fn has_attachments(&self) -> bool {
        self.has_attachments
    }

    /// Check if the last message is an outgoing reply
    pub fn last_is_outgoing_reply(&self) -> bool {
        self.last_is_outgoing_reply
    }
}

/// Group messages into threads using JWZ-style threading
pub fn group_into_threads(mut msgs: Vec<Message>) -> Vec<Thread> {
    // Sort messages by date (oldest first)
    msgs.sort_by_key(|m| m.headers.date.unwrap_or(OffsetDateTime::UNIX_EPOCH));

    // 1) P1: server thread id (if available)
    let (with_tid, without_tid): (Vec<_>, Vec<_>) =
        msgs.into_iter().partition(|m| m.headers.message_id.is_some());

    let mut buckets: HashMap<String, Vec<Message>> = HashMap::new();
    for m in with_tid {
        if let Some(tid) = &m.headers.message_id {
            buckets.entry(tid.clone()).or_default().push(m);
        }
    }

    // 2) P2: JWZ style threading on the rest
    let jwz_threads = jwz_threads(without_tid);
    for (tid, v) in jwz_threads {
        buckets.entry(tid).or_default().extend(v);
    }

    // 3) Normalize each bucket, compute summaries
    let mut threads: Vec<Thread> = buckets.into_iter().map(|(tid, mut v)| {
        v.sort_by_key(|m| m.headers.date.unwrap_or(OffsetDateTime::UNIX_EPOCH)); // oldest→newest
        let subject = canonical_subject(v.last().map(|m| m.headers.subject.clone()).unwrap_or_default());
        Thread::new(tid, subject, v)
    }).collect();

    // Sort threads by last message time descending for the middle pane
    threads.sort_by_key(|t| t.last_date);
    threads.reverse();
    threads
}

// Build nodes for Message-ID and placeholders referenced in References
#[derive(Clone)]
struct Node { 
    mid: String, 
    msg: Option<Message>, 
    parent: Option<String>, 
    children: Vec<String> 
}

/// JWZ-style threading algorithm implementation
fn jwz_threads(messages: Vec<Message>) -> HashMap<String, Vec<Message>> {
    let mut nodes: HashMap<String, Node> = HashMap::new();

    // pass 1: create nodes
    for m in &messages {
        if let Some(mid) = &m.headers.message_id { 
            nodes.entry(mid.clone()).or_insert(Node{ 
                mid: mid.clone(), 
                msg: Some(m.clone()), 
                parent: None, 
                children: vec![] 
            });
        }
        for r in &m.headers.references { 
            nodes.entry(r.clone()).or_insert(Node{ 
                mid: r.clone(), 
                msg: None, 
                parent: None, 
                children: vec![] 
            });
        }
    }

    // pass 2: link using References (parent = last ref) or In-Reply-To
    for m in &messages {
        if let Some(mid) = &m.headers.message_id {
            let parent_id = m.headers.references.as_ref().and_then(|r| r.split_whitespace().last().map(|s| s.to_string())).or_else(|| m.headers.in_reply_to.clone());
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
    let mut out: HashMap<String, Vec<Message>> = HashMap::new();
    for root in roots.drain(..) {
        let tid = root.clone(); // synthetic id: root Message-ID
        let mut acc: Vec<Message> = Vec::new();
        collect_inorder(&nodes, &root, &mut acc);
        out.insert(tid, acc);
    }

    // orphans: nodes with a msg but not in any collected thread
    let seen: HashSet<String> = out.values().flatten()
        .filter_map(|m| m.headers.message_id.clone()).collect();
    for n in nodes.values() {
        if let Some(m) = &n.msg {
            if !m.headers.message_id.as_ref().map(|x| seen.contains(x)).unwrap_or(false) {
                let tid = m.headers.message_id.clone().unwrap_or_else(|| format!("midless-{}", m.id));
                out.entry(tid).or_default().push(m.clone());
            }
        }
    }
    out
}

/// Recursively collect messages in thread order
fn collect_inorder(nodes: &HashMap<String, Node>, id: &String, acc: &mut Vec<Message>) {
    if let Some(n) = nodes.get(id) {
        if let Some(m) = &n.msg { 
            acc.push(m.clone()); 
        }
        // order children by Date header if available
        let mut childs = n.children.clone();
        childs.sort_by_key(|cid| nodes.get(cid).and_then(|c| c.msg.as_ref()).map(|m| m.headers.date.unwrap_or(OffsetDateTime::UNIX_EPOCH)));
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

/// Helper function to format date for display
pub fn format_date_short(date: &time::OffsetDateTime) -> String {
    let now = time::OffsetDateTime::now_utc();
    let duration = now - *date;

    if duration.as_seconds_f64() < 60.0 {
        "now".to_string()
    } else if duration.whole_minutes() < 60 {
        format!("{}m ago", duration.whole_minutes())
    } else if duration.whole_hours() < 24 {
        format!("{}h ago", duration.whole_hours())
    } else if duration.whole_days() == 1 {
        "Yesterday".to_string()
    } else if duration.whole_days() < 7 {
        // Simple weekday format
        match date.weekday() {
            time::Weekday::Monday => "Mon".to_string(),
            time::Weekday::Tuesday => "Tue".to_string(),
            time::Weekday::Wednesday => "Wed".to_string(),
            time::Weekday::Thursday => "Thu".to_string(),
            time::Weekday::Friday => "Fri".to_string(),
            time::Weekday::Saturday => "Sat".to_string(),
            time::Weekday::Sunday => "Sun".to_string(),
        }
    } else if date.year() == now.year() {
        format!("{} {}", date.month(), date.day())
    } else {
        format!("{}/{}/{}", date.year(), date.month() as u8, date.day())
    }
}

/// Determine if a message is outgoing based on RFC standards
/// A message is outgoing if:
/// 1. It's in the "Sent" mailbox, OR
/// 2. The sender matches the account's email address
pub fn is_outgoing_message(message: &Message) -> bool {
    // Check if message is in Sent mailbox
    if message.mailbox_id.to_string().contains("sent") {
        return true;
    }
    
    // For demo purposes, check if sender contains the demo email
    // In a real implementation, this would check against the account's email
    message.headers.from.iter().any(|addr| addr.email.contains("demo@example.com"))
}

/// Determine if a message is a reply based on RFC standards
/// A message is a reply if it has:
/// 1. In-Reply-To header, OR
/// 2. References header with content
pub fn is_reply_message(message: &Message) -> bool {
    message.headers.in_reply_to.is_some() || message.headers.references.as_ref().map_or(false, |r| !r.is_empty())
}
