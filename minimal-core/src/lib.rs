//! Minimal core library for Asgard Mail

use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

/// Basic account structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub id: Uuid,
    pub email: String,
    pub display_name: String,
    pub created_at: OffsetDateTime,
}

impl Account {
    pub fn new(email: String, display_name: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            email,
            display_name,
            created_at: OffsetDateTime::now_utc(),
        }
    }
}

/// Basic message structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: Uuid,
    pub account_id: Uuid,
    pub subject: String,
    pub from: String,
    pub body: String,
    pub created_at: OffsetDateTime,
}

impl Message {
    pub fn new(account_id: Uuid, subject: String, from: String, body: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            account_id,
            subject,
            from,
            body,
            created_at: OffsetDateTime::now_utc(),
        }
    }
}

/// Basic mailbox structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mailbox {
    pub id: Uuid,
    pub account_id: Uuid,
    pub name: String,
    pub created_at: OffsetDateTime,
}

impl Mailbox {
    pub fn new(account_id: Uuid, name: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            account_id,
            name,
            created_at: OffsetDateTime::now_utc(),
        }
    }
}
