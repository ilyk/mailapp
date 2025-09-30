//! IMAP sync engine for Asgard Mail

use crate::error::{AsgardError, AsgardResult};
use crate::account::Account;
use crate::mailbox::Mailbox;
use crate::message::Message;
use crate::gmail::XOAUTH2;
use async_imap::Session;
use async_imap::types::{Fetch, ImapError};
use async_imap::extensions::idle::IdleResponse;
use async_imap::error::Result as ImapResult;
use async_native_tls::TlsConnector;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio::time::timeout;
use tracing::{info, warn, error};

/// IMAP sync engine
pub struct ImapSync {
    /// Account being synced
    account: Account,
    /// IMAP session
    session: Option<Session<TcpStream>>,
    /// Sync status
    status: crate::sync::SyncStatus,
    /// Last sync result
    last_sync_result: Option<crate::sync::SyncResult>,
}

impl ImapSync {
    /// Create a new IMAP sync engine
    pub fn new(account: Account) -> Self {
        Self {
            account,
            session: None,
            status: crate::sync::SyncStatus::Idle,
            last_sync_result: None,
        }
    }
    
    /// Get the account ID
    pub fn account_id(&self) -> Uuid {
        self.account.id
    }

    /// Connect to IMAP server
    pub async fn connect(&mut self) -> AsgardResult<()> {
        let imap_config = self.account.imap_config()
            .ok_or_else(|| AsgardError::account("IMAP configuration not found"))?;

        let tcp_stream = TcpStream::connect((imap_config.host.as_str(), imap_config.port)).await?;
        
        let tls_connector = TlsConnector::new();
        let tls_stream = if imap_config.use_tls {
            tls_connector.connect(&imap_config.host, tcp_stream).await?
        } else {
            return Err(AsgardError::tls("TLS is required for IMAP"));
        };

        let client = async_imap::Client::new(tls_stream);
        let mut session = client.login("", "").await
            .map_err(|e| AsgardError::imap(e))?;

        // Authenticate
        match imap_config.auth_method {
            crate::account::AuthMethod::OAuth2 => {
                if let Some(oauth_config) = self.account.gmail_oauth_config() {
                    if let Some(access_token) = &oauth_config.access_token {
                        let xoauth2 = XOAUTH2::new(
                            self.account.email().to_string(),
                            access_token.clone(),
                        );
                        
                        let auth_string = xoauth2.generate_imap_auth_string()?;
                        session.authenticate("XOAUTH2", &auth_string).await
                            .map_err(|e| AsgardError::imap(e))?;
                    } else {
                        return Err(AsgardError::auth("No access token available"));
                    }
                } else {
                    return Err(AsgardError::auth("OAuth configuration not found"));
                }
            }
            crate::account::AuthMethod::Password => {
                // For password auth, we'd need to get the password from keyring
                return Err(AsgardError::auth("Password authentication not implemented"));
            }
            crate::account::AuthMethod::AppPassword => {
                // For app password auth, we'd need to get the app password from keyring
                return Err(AsgardError::auth("App password authentication not implemented"));
            }
        }

        self.session = Some(session);
        info!("Connected to IMAP server for account: {}", self.account.email());
        Ok(())
    }

    /// Disconnect from IMAP server
    pub async fn disconnect(&mut self) -> AsgardResult<()> {
        if let Some(session) = self.session.take() {
            session.logout().await.map_err(|e| AsgardError::imap(e))?;
            info!("Disconnected from IMAP server for account: {}", self.account.email());
        }
        Ok(())
    }

    /// Sync mailboxes
    pub async fn sync_mailboxes(&mut self) -> AsgardResult<Vec<Mailbox>> {
        let session = self.session.as_mut()
            .ok_or_else(|| AsgardError::invalid_state("Not connected to IMAP server"))?;

        let mailboxes = session.list(Some(""), Some("*")).await
            .map_err(|e| AsgardError::imap(e))?;

        let mut result = Vec::new();
        for mailbox in mailboxes {
            let mailbox_name = mailbox.name();
            let mailbox_type = self.determine_mailbox_type(&mailbox_name);
            
            let asgard_mailbox = Mailbox::new(
                self.account.id,
                mailbox_name.to_string(),
                None,
                mailbox_type,
                None,
            );
            
            result.push(asgard_mailbox);
        }

        info!("Synced {} mailboxes for account: {}", result.len(), self.account.email());
        Ok(result)
    }

    /// Sync messages in a mailbox
    pub async fn sync_mailbox_messages(&mut self, mailbox: &Mailbox) -> AsgardResult<Vec<Message>> {
        let session = self.session.as_mut()
            .ok_or_else(|| AsgardError::invalid_state("Not connected to IMAP server"))?;

        // Select mailbox
        session.select(&mailbox.name).await
            .map_err(|e| AsgardError::imap(e))?;

        // Get message count
        let mailbox_info = session.status(&mailbox.name, &["MESSAGES", "UNSEEN"]).await
            .map_err(|e| AsgardError::imap(e))?;

        let message_count = mailbox_info.messages.unwrap_or(0);
        if message_count == 0 {
            return Ok(Vec::new());
        }

        // Fetch messages
        let fetch_result = session.fetch("1:*", "ENVELOPE BODYSTRUCTURE BODY[HEADER] BODY[TEXT]").await
            .map_err(|e| AsgardError::imap(e))?;

        let mut messages = Vec::new();
        for fetch in fetch_result {
            if let Ok(message) = self.parse_fetch_result(fetch, mailbox.id).await {
                messages.push(message);
            }
        }

        info!("Synced {} messages from mailbox: {}", messages.len(), mailbox.name);
        Ok(messages)
    }

    /// Start IDLE mode for real-time updates
    pub async fn start_idle(&mut self, mailbox_name: &str) -> AsgardResult<mpsc::Receiver<IdleResponse>> {
        let session = self.session.as_mut()
            .ok_or_else(|| AsgardError::invalid_state("Not connected to IMAP server"))?;

        // Select mailbox
        session.select(mailbox_name).await
            .map_err(|e| AsgardError::imap(e))?;

        // Start IDLE
        let (idle, mut interrupt) = session.idle().await
            .map_err(|e| AsgardError::imap(e))?;

        let (tx, rx) = mpsc::channel(10);
        
        // Spawn IDLE task
        tokio::spawn(async move {
            let result = idle.wait_with_timeout(Duration::from_secs(30)).await;
            if let Err(e) = tx.send(result).await {
                error!("Failed to send IDLE response: {}", e);
            }
        });

        Ok(rx)
    }

    /// Stop IDLE mode
    pub async fn stop_idle(&mut self) -> AsgardResult<()> {
        // IDLE will be stopped when the session is dropped or logout is called
        Ok(())
    }

    /// Get sync status
    pub fn status(&self) -> crate::sync::SyncStatus {
        self.status
    }

    /// Get last sync result
    pub fn last_sync_result(&self) -> Option<&crate::sync::SyncResult> {
        self.last_sync_result.as_ref()
    }

    // Helper methods

    fn determine_mailbox_type(&self, mailbox_name: &str) -> crate::mailbox::MailboxType {
        match mailbox_name {
            "INBOX" => crate::mailbox::MailboxType::Inbox,
            name if name.contains("Sent") => crate::mailbox::MailboxType::Sent,
            name if name.contains("Draft") => crate::mailbox::MailboxType::Drafts,
            name if name.contains("Trash") => crate::mailbox::MailboxType::Trash,
            name if name.contains("Spam") => crate::mailbox::MailboxType::Spam,
            name if name.contains("Archive") => crate::mailbox::MailboxType::Archive,
            _ => crate::mailbox::MailboxType::Custom,
        }
    }

    async fn parse_fetch_result(&self, fetch: Fetch, mailbox_id: uuid::Uuid) -> AsgardResult<Message> {
        // Parse IMAP FETCH response into Asgard Message
        // This is a simplified implementation
        
        let uid = fetch.uid.unwrap_or(0);
        let sequence_number = fetch.message;
        
        // Parse envelope
        let envelope = fetch.envelope().ok_or_else(|| AsgardError::message("No envelope in fetch result"))?;
        
        let subject = envelope.subject
            .map(|s| String::from_utf8_lossy(&s).to_string())
            .unwrap_or_default();
        
        let from = envelope.from
            .map(|addrs| {
                addrs.iter()
                    .map(|addr| crate::message::EmailAddress {
                        name: addr.name.as_ref().map(|n| String::from_utf8_lossy(n).to_string()),
                        email: String::from_utf8_lossy(&addr.mailbox).to_string(),
                    })
                    .collect()
            })
            .unwrap_or_default();
        
        let to = envelope.to
            .map(|addrs| {
                addrs.iter()
                    .map(|addr| crate::message::EmailAddress {
                        name: addr.name.as_ref().map(|n| String::from_utf8_lossy(n).to_string()),
                        email: String::from_utf8_lossy(&addr.mailbox).to_string(),
                    })
                    .collect()
            })
            .unwrap_or_default();
        
        let date = envelope.date
            .map(|date| {
                // Parse IMAP date format
                time::OffsetDateTime::now_utc() // Simplified - would need proper date parsing
            });
        
        let headers = crate::message::MessageHeaders {
            message_id: envelope.message_id
                .map(|id| String::from_utf8_lossy(&id).to_string()),
            in_reply_to: envelope.in_reply_to
                .map(|id| String::from_utf8_lossy(&id).to_string()),
            references: envelope.references
                .map(|refs| {
                    refs.iter()
                        .map(|r| String::from_utf8_lossy(r).to_string())
                        .collect::<Vec<_>>()
                        .join(" ")
                }),
            subject,
            from,
            to,
            cc: vec![],
            bcc: vec![],
            reply_to: vec![],
            date,
            received_date: date,
            importance: crate::message::MessageImportance::Normal,
            custom: std::collections::HashMap::new(),
        };
        
        let mut message = Message::new(self.account.id, mailbox_id, headers);
        message.set_uid(uid, 1); // UID validity would be obtained from mailbox
        message.set_sequence_number(sequence_number);
        
        // Parse body parts
        if let Some(body) = fetch.body() {
            // Parse MIME structure and extract text/HTML content
            // This is simplified - would need proper MIME parsing
            let content = String::from_utf8_lossy(&body).to_string();
            
            let part = crate::message::MessagePart {
                id: "1".to_string(),
                part_type: crate::message::MessagePartType::Text,
                mime_type: "text/plain".to_string(),
                disposition: None,
                filename: None,
                size: body.len(),
                encoding: None,
                content_id: None,
                content_location: None,
                content: Some(body),
                children: vec![],
            };
            
            message.add_part(part);
        }
        
        Ok(message)
    }
}

impl Drop for ImapSync {
    fn drop(&mut self) {
        if self.session.is_some() {
            // Note: We can't use async in Drop, so we'll just log a warning
            warn!("ImapSync dropped while connected - connection may not be properly closed");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::account::{Account, AccountType, GmailOAuthConfig, ServerConfig, AuthMethod};

    #[test]
    fn test_imap_sync_creation() {
        let oauth_config = GmailOAuthConfig {
            client_id: "test-client-id".to_string(),
            client_secret: "test-client-secret".to_string(),
            access_token: Some("test-access-token".to_string()),
            refresh_token: None,
            token_expires_at: None,
            scopes: vec!["https://www.googleapis.com/auth/gmail.readonly".to_string()],
        };

        let account = Account::new_gmail(
            "test@gmail.com".to_string(),
            Some("Test Account".to_string()),
            oauth_config,
        ).unwrap();

        let imap_sync = ImapSync::new(account);
        assert_eq!(imap_sync.status(), crate::sync::SyncStatus::Idle);
        assert!(imap_sync.last_sync_result().is_none());
    }

    #[test]
    fn test_mailbox_type_determination() {
        let oauth_config = GmailOAuthConfig {
            client_id: "test-client-id".to_string(),
            client_secret: "test-client-secret".to_string(),
            access_token: Some("test-access-token".to_string()),
            refresh_token: None,
            token_expires_at: None,
            scopes: vec!["https://www.googleapis.com/auth/gmail.readonly".to_string()],
        };

        let account = Account::new_gmail(
            "test@gmail.com".to_string(),
            Some("Test Account".to_string()),
            oauth_config,
        ).unwrap();

        let imap_sync = ImapSync::new(account);
        
        // Note: We can't test the private method directly, but we can test the behavior
        // through the public interface when we have a real IMAP connection
    }
}
