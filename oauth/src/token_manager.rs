//! Token management for OAuth2

use crate::OAuthToken;
use asgard_core::error::{AsgardError, AsgardResult};
use asgard_core::crypto::TokenEncryption;
use keyring::Entry;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Token manager for storing and retrieving OAuth2 tokens
pub struct TokenManager {
    /// Encrypted token storage
    token_encryption: TokenEncryption,
    /// In-memory token cache
    token_cache: Arc<RwLock<HashMap<Uuid, OAuthToken>>>,
}

impl TokenManager {
    /// Create a new token manager
    pub fn new() -> AsgardResult<Self> {
        // Create encryption key (in a real implementation, this would be derived from user credentials)
        let encryption_key = asgard_core::crypto::EncryptionKey::new();
        let token_encryption = TokenEncryption::new(encryption_key);

        Ok(Self {
            token_encryption,
            token_cache: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Store a token for an account
    pub async fn store_token(&self, account_id: Uuid, token: OAuthToken) -> AsgardResult<()> {
        // Encrypt access token
        let encrypted_access_token = self.token_encryption.encrypt_access_token(&token.access_token)?;
        
        // Encrypt refresh token if present
        let encrypted_refresh_token = if let Some(refresh_token) = &token.refresh_token {
            Some(self.token_encryption.encrypt_refresh_token(refresh_token)?)
        } else {
            None
        };

        // Store in keyring
        let access_token_entry = Entry::new("asgard-mail", &format!("access_token_{}", account_id))?;
        access_token_entry.set_password(&encrypted_access_token)?;

        if let Some(encrypted_refresh) = encrypted_refresh_token {
            let refresh_token_entry = Entry::new("asgard-mail", &format!("refresh_token_{}", account_id))?;
            refresh_token_entry.set_password(&encrypted_refresh)?;
        }

        // Store expiration time
        if let Some(expires_at) = token.expires_at {
            let expires_entry = Entry::new("asgard-mail", &format!("expires_at_{}", account_id))?;
            expires_entry.set_password(&expires_at.unix_timestamp().to_string())?;
        }

        // Store in cache
        {
            let mut cache = self.token_cache.write().await;
            cache.insert(account_id, token);
        }

        Ok(())
    }

    /// Retrieve a token for an account
    pub async fn get_token(&self, account_id: Uuid) -> AsgardResult<Option<OAuthToken>> {
        // Check cache first
        {
            let cache = self.token_cache.read().await;
            if let Some(token) = cache.get(&account_id) {
                return Ok(Some(token.clone()));
            }
        }

        // Load from keyring
        let access_token_entry = Entry::new("asgard-mail", &format!("access_token_{}", account_id))?;
        let encrypted_access_token = match access_token_entry.get_password() {
            Ok(token) => token,
            Err(_) => return Ok(None), // Token not found
        };

        let access_token = self.token_encryption.decrypt_access_token(&encrypted_access_token)?;

        // Load refresh token
        let refresh_token = {
            let refresh_token_entry = Entry::new("asgard-mail", &format!("refresh_token_{}", account_id))?;
            match refresh_token_entry.get_password() {
                Ok(encrypted_refresh) => {
                    Some(self.token_encryption.decrypt_refresh_token(&encrypted_refresh)?)
                }
                Err(_) => None,
            }
        };

        // Load expiration time
        let expires_at = {
            let expires_entry = Entry::new("asgard-mail", &format!("expires_at_{}", account_id))?;
            match expires_entry.get_password() {
                Ok(expires_str) => {
                    expires_str.parse::<i64>().ok()
                        .and_then(|timestamp| time::OffsetDateTime::from_unix_timestamp(timestamp).ok())
                }
                Err(_) => None,
            }
        };

        let token = OAuthToken {
            access_token,
            refresh_token,
            token_type: "Bearer".to_string(),
            expires_at,
            scope: None, // Would need to store this separately if needed
        };

        // Store in cache
        {
            let mut cache = self.token_cache.write().await;
            cache.insert(account_id, token.clone());
        }

        Ok(Some(token))
    }

    /// Remove a token for an account
    pub async fn remove_token(&self, account_id: Uuid) -> AsgardResult<()> {
        // Remove from keyring
        let access_token_entry = Entry::new("asgard-mail", &format!("access_token_{}", account_id))?;
        let _ = access_token_entry.delete_password();

        let refresh_token_entry = Entry::new("asgard-mail", &format!("refresh_token_{}", account_id))?;
        let _ = refresh_token_entry.delete_password();

        let expires_entry = Entry::new("asgard-mail", &format!("expires_at_{}", account_id))?;
        let _ = expires_entry.delete_password();

        // Remove from cache
        {
            let mut cache = self.token_cache.write().await;
            cache.remove(&account_id);
        }

        Ok(())
    }

    /// Check if a token exists for an account
    pub async fn has_token(&self, account_id: Uuid) -> bool {
        // Check cache first
        {
            let cache = self.token_cache.read().await;
            if cache.contains_key(&account_id) {
                return true;
            }
        }

        // Check keyring
        let access_token_entry = Entry::new("asgard-mail", &format!("access_token_{}", account_id));
        match access_token_entry {
            Ok(entry) => entry.get_password().is_ok(),
            Err(_) => false,
        }
    }

    /// Get all account IDs with stored tokens
    pub async fn get_account_ids(&self) -> AsgardResult<Vec<Uuid>> {
        // This is a simplified implementation
        // In a real implementation, you'd need to enumerate keyring entries
        let cache = self.token_cache.read().await;
        Ok(cache.keys().cloned().collect())
    }

    /// Clear all tokens
    pub async fn clear_all_tokens(&self) -> AsgardResult<()> {
        // Clear cache
        {
            let mut cache = self.token_cache.write().await;
            cache.clear();
        }

        // Note: Clearing all keyring entries would require enumeration
        // This is a simplified implementation
        Ok(())
    }

    /// Refresh a token if it's expired or about to expire
    pub async fn refresh_token_if_needed(&self, account_id: Uuid, oauth_client: &crate::GmailOAuth) -> AsgardResult<Option<OAuthToken>> {
        let token = match self.get_token(account_id).await? {
            Some(token) => token,
            None => return Ok(None),
        };

        if token.is_expired() || token.expires_soon() {
            if let Some(refresh_token) = &token.refresh_token {
                let new_token = oauth_client.refresh_token(refresh_token).await?;
                self.store_token(account_id, new_token.clone()).await?;
                Ok(Some(new_token))
            } else {
                // No refresh token available, remove the expired token
                self.remove_token(account_id).await?;
                Ok(None)
            }
        } else {
            Ok(Some(token))
        }
    }
}

impl Default for TokenManager {
    fn default() -> Self {
        Self::new().expect("Failed to create TokenManager")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_token_manager_creation() {
        let token_manager = TokenManager::new().unwrap();
        assert!(token_manager.token_cache.read().await.is_empty());
    }

    #[tokio::test]
    async fn test_token_storage_and_retrieval() {
        let token_manager = TokenManager::new().unwrap();
        let account_id = Uuid::new_v4();

        let token = OAuthToken {
            access_token: "test-access-token".to_string(),
            refresh_token: Some("test-refresh-token".to_string()),
            token_type: "Bearer".to_string(),
            expires_at: Some(time::OffsetDateTime::now_utc() + time::Duration::hours(1)),
            scope: Some("test-scope".to_string()),
        };

        // Store token
        token_manager.store_token(account_id, token.clone()).await.unwrap();

        // Retrieve token
        let retrieved_token = token_manager.get_token(account_id).await.unwrap().unwrap();
        assert_eq!(retrieved_token.access_token, token.access_token);
        assert_eq!(retrieved_token.refresh_token, token.refresh_token);
        assert_eq!(retrieved_token.token_type, token.token_type);

        // Check if token exists
        assert!(token_manager.has_token(account_id).await);

        // Remove token
        token_manager.remove_token(account_id).await.unwrap();
        assert!(!token_manager.has_token(account_id).await);
    }

    #[tokio::test]
    async fn test_token_expiration() {
        let token_manager = TokenManager::new().unwrap();
        let account_id = Uuid::new_v4();

        let expired_token = OAuthToken {
            access_token: "test-access-token".to_string(),
            refresh_token: Some("test-refresh-token".to_string()),
            token_type: "Bearer".to_string(),
            expires_at: Some(time::OffsetDateTime::now_utc() - time::Duration::hours(1)),
            scope: None,
        };

        token_manager.store_token(account_id, expired_token).await.unwrap();
        assert!(token_manager.get_token(account_id).await.unwrap().unwrap().is_expired());
    }
}
