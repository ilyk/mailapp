//! OAuth2 implementation for Asgard Mail

pub mod gmail_oauth;
pub mod token_manager;

pub use gmail_oauth::GmailOAuth;
pub use token_manager::TokenManager;

/// OAuth2 configuration
#[derive(Debug, Clone)]
pub struct OAuthConfig {
    /// Client ID
    pub client_id: String,
    /// Client secret
    pub client_secret: String,
    /// Redirect URI
    pub redirect_uri: String,
    /// Scopes
    pub scopes: Vec<String>,
}

/// OAuth2 token response
#[derive(Debug, Clone, serde::Deserialize)]
pub struct TokenResponse {
    /// Access token
    pub access_token: String,
    /// Token type
    pub token_type: String,
    /// Expires in seconds
    pub expires_in: Option<u64>,
    /// Refresh token
    pub refresh_token: Option<String>,
    /// Scope
    pub scope: Option<String>,
}

/// OAuth2 error response
#[derive(Debug, Clone, serde::Deserialize)]
pub struct OAuthError {
    /// Error code
    pub error: String,
    /// Error description
    pub error_description: Option<String>,
    /// Error URI
    pub error_uri: Option<String>,
}

/// OAuth2 authorization URL
#[derive(Debug, Clone)]
pub struct AuthorizationUrl {
    /// Authorization URL
    pub url: String,
    /// State parameter
    pub state: String,
    /// Code verifier (for PKCE)
    pub code_verifier: String,
}

/// OAuth2 authorization code
#[derive(Debug, Clone)]
pub struct AuthorizationCode {
    /// Authorization code
    pub code: String,
    /// State parameter
    pub state: String,
}

/// OAuth2 token
#[derive(Debug, Clone)]
pub struct OAuthToken {
    /// Access token
    pub access_token: String,
    /// Refresh token
    pub refresh_token: Option<String>,
    /// Token type
    pub token_type: String,
    /// Expires at
    pub expires_at: Option<time::OffsetDateTime>,
    /// Scope
    pub scope: Option<String>,
}

impl OAuthToken {
    /// Check if the token is expired
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            time::OffsetDateTime::now_utc() >= expires_at
        } else {
            false
        }
    }

    /// Check if the token will expire soon (within 5 minutes)
    pub fn expires_soon(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            let now = time::OffsetDateTime::now_utc();
            let five_minutes = time::Duration::minutes(5);
            expires_at - now <= five_minutes
        } else {
            false
        }
    }
}
