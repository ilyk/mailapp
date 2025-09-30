//! SMTP sending for Asgard Mail

use crate::error::{AsgardError, AsgardResult};
use crate::account::Account;
use crate::message::Message;
use crate::gmail::XOAUTH2;
use lettre::{
    message::{header, Mailbox as LettreMailbox, MultiPart, SinglePart},
    transport::smtp::authentication::Credentials,
    AsyncSmtpTransport, AsyncTransport, Message as LettreMessage, Tokio1Executor,
};
use std::sync::Arc;

/// SMTP sending engine
pub struct SmtpSend {
    /// Account for sending
    account: Account,
    /// SMTP transport
    transport: Option<AsyncSmtpTransport<Tokio1Executor>>,
}

impl SmtpSend {
    /// Create a new SMTP sending engine
    pub fn new(account: Account) -> Self {
        Self {
            account,
            transport: None,
        }
    }

    /// Connect to SMTP server
    pub async fn connect(&mut self) -> AsgardResult<()> {
        let smtp_config = self.account.smtp_config()
            .ok_or_else(|| AsgardError::account("SMTP configuration not found"))?;

        let mut builder = AsyncSmtpTransport::<Tokio1Executor>::relay(&smtp_config.host)?;

        if smtp_config.use_starttls {
            use lettre::transport::smtp::client::{Tls, TlsParameters};
            let tls_params = TlsParameters::new(smtp_config.host.clone()).unwrap();
            builder = builder.tls(Tls::Required(tls_params));
        }

        let transport = builder.build();

        self.transport = Some(transport);
        Ok(())
    }

    /// Send a message
    pub async fn send_message(&self, message: &Message) -> AsgardResult<()> {
        let transport = self.transport.as_ref()
            .ok_or_else(|| AsgardError::invalid_state("Not connected to SMTP server"))?;

        let smtp_config = self.account.smtp_config()
            .ok_or_else(|| AsgardError::account("SMTP configuration not found"))?;

        // Build email message
        let mut email_builder = LettreMessage::builder()
            .from(self.parse_email_address(&message.headers.from[0])?)
            .subject(&message.headers.subject);

        // Add recipients
        for to_addr in &message.headers.to {
            email_builder = email_builder.to(self.parse_email_address(to_addr)?);
        }

        for cc_addr in &message.headers.cc {
            email_builder = email_builder.cc(self.parse_email_address(cc_addr)?);
        }

        for bcc_addr in &message.headers.bcc {
            email_builder = email_builder.bcc(self.parse_email_address(bcc_addr)?);
        }

        // Add message body - simplified for now
        let body = if let Some(text_content) = message.text_content() {
            String::from_utf8_lossy(text_content).to_string()
        } else if let Some(html_content) = message.html_content() {
            String::from_utf8_lossy(html_content).to_string()
        } else {
            "".to_string()
        };

        let email = email_builder.body(body)?;

        // Send email
        match smtp_config.auth_method {
            crate::account::AuthMethod::OAuth2 => {
                if let Some(oauth_config) = self.account.gmail_oauth_config() {
                    if let Some(access_token) = &oauth_config.access_token {
                        let xoauth2 = XOAUTH2::new(
                            self.account.email().to_string(),
                            access_token.clone(),
                        );
                        
                        let auth_string = xoauth2.generate_smtp_auth_string()?;
                        // Note: XOAUTH2 for SMTP would require additional implementation
                        // This is a simplified version
                        transport.send(email).await?;
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

        Ok(())
    }

    // Helper methods

    fn parse_email_address(&self, addr: &crate::message::EmailAddress) -> AsgardResult<LettreMailbox> {
        if let Some(name) = &addr.name {
            Ok(LettreMailbox::new(Some(name.clone()), addr.email.parse()?))
        } else {
            Ok(LettreMailbox::new(None, addr.email.parse()?))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::account::{Account, GmailOAuthConfig};

    #[test]
    fn test_smtp_send_creation() {
        let oauth_config = GmailOAuthConfig {
            client_id: "test-client-id".to_string(),
            client_secret: "test-client-secret".to_string(),
            access_token: Some("test-access-token".to_string()),
            refresh_token: None,
            token_expires_at: None,
            scopes: vec!["https://www.googleapis.com/auth/gmail.send".to_string()],
        };

        let account = Account::new_gmail(
            "test@gmail.com".to_string(),
            Some("Test Account".to_string()),
            oauth_config,
        ).unwrap();

        let smtp_send = SmtpSend::new(account);
        assert!(smtp_send.transport.is_none());
    }
}
