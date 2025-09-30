use crate::message::Message;
use crate::types::MsgMeta;
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ThreadId(Uuid);

impl ThreadId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

#[derive(Debug, Clone)]
pub struct MessageThread {
    pub id: ThreadId,
    pub messages: Vec<Message>,
    pub subject: String,
    pub is_expanded: bool,
}

impl MessageThread {
    pub fn new() -> Self {
        Self {
            id: ThreadId::new(),
            messages: Vec::new(),
            subject: String::new(),
            is_expanded: false,
        }
    }
    
    pub fn add_message(&mut self, message: Message) {
        if self.subject.is_empty() {
            self.subject = message.subject().to_string();
        }
        self.messages.push(message);
        // Sort by date, newest first
        self.messages.sort_by(|a, b| b.date().cmp(&a.date()));
    }
    
    pub fn len(&self) -> usize {
        self.messages.len()
    }
    
    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }
    
    pub fn latest_message(&self) -> Option<&Message> {
        self.messages.first()
    }
}

#[derive(Debug, Clone)]
pub struct ThreadManager {
    threads: HashMap<ThreadId, MessageThread>,
    message_to_thread: HashMap<Uuid, ThreadId>,
}

impl ThreadManager {
    pub fn new() -> Self {
        Self {
            threads: HashMap::new(),
            message_to_thread: HashMap::new(),
        }
    }
    
    pub fn add_message(&mut self, message: Message) -> ThreadId {
        // For now, create a new thread for each message
        // In a real implementation, this would look at Message-ID, In-Reply-To, References headers
        let thread_id = ThreadId::new();
        
        let mut thread = MessageThread::new();
        thread.add_message(message.clone());
        
        self.threads.insert(thread_id.clone(), thread);
        self.message_to_thread.insert(message.id, thread_id.clone());
        
        thread_id
    }
    
    pub fn get_thread(&self, thread_id: &ThreadId) -> Option<&MessageThread> {
        self.threads.get(thread_id)
    }
    
    pub fn get_thread_mut(&mut self, thread_id: &ThreadId) -> Option<&mut MessageThread> {
        self.threads.get_mut(thread_id)
    }
    
    pub fn get_thread_for_message(&self, message_id: &Uuid) -> Option<&MessageThread> {
        let thread_id = self.message_to_thread.get(message_id)?;
        self.threads.get(thread_id)
    }
    
    pub fn all_threads(&self) -> Vec<&MessageThread> {
        self.threads.values().collect()
    }
    
    pub fn toggle_thread_expansion(&mut self, thread_id: &ThreadId) {
        if let Some(thread) = self.threads.get_mut(thread_id) {
            thread.is_expanded = !thread.is_expanded;
        }
    }
    
    pub fn clear(&mut self) {
        self.threads.clear();
        self.message_to_thread.clear();
    }
}
