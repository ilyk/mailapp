//! XOAUTH2 implementation for Gmail

use crate::error::{AsgardError, AsgardResult};
use base64::{Engine as _, engine::general_purpose};

/// XOAUTH2 authentication for Gmail IMAP/SMTP
pub struct XOAUTH2 {
    /// Email address
    email: String,
    /// OAuth2 access token
    access_token: String,
}

impl XOAUTH2 {
    /// Create a new XOAUTH2 instance
    pub fn new(email: String, access_token: String) -> Self {
        Self {
            email,
            access_token,
        }
    }

    /// Generate XOAUTH2 SASL string for IMAP
    pub fn generate_imap_auth_string(&self) -> AsgardResult<String> {
        let auth_string = format!(
            "user={}\x01auth=Bearer {}\x01\x01",
            self.email, self.access_token
        );
        
        let encoded = general_purpose::STANDARD.encode(auth_string.as_bytes());
        Ok(format!("AUTHENTICATE XOAUTH2 {}", encoded))
    }

    /// Generate XOAUTH2 SASL string for SMTP
    pub fn generate_smtp_auth_string(&self) -> AsgardResult<String> {
        let auth_string = format!(
            "user={}\x01auth=Bearer {}\x01\x01",
            self.email, self.access_token
        );
        
        let encoded = general_purpose::STANDARD.encode(auth_string.as_bytes());
        Ok(encoded)
    }

    /// Parse XOAUTH2 response from server
    pub fn parse_server_response(response: &str) -> AsgardResult<XOAUTH2Response> {
        if response == "+" {
            Ok(XOAUTH2Response::Continue)
        } else if response.starts_with("+ ") {
            // Server sent base64-encoded challenge
            let challenge = &response[2..];
            let decoded = general_purpose::STANDARD.decode(challenge)
                .map_err(|_| AsgardError::auth("Invalid base64 challenge"))?;
            
            let challenge_str = String::from_utf8(decoded)
                .map_err(|_| AsgardError::auth("Invalid UTF-8 challenge"))?;
            
            Ok(XOAUTH2Response::Challenge(challenge_str))
        } else if response.starts_with("+OK") {
            Ok(XOAUTH2Response::Success)
        } else if response.starts_with("+NO") {
            Ok(XOAUTH2Response::Failure)
        } else {
            Err(AsgardError::auth("Unknown XOAUTH2 response"))
        }
    }

    /// Validate XOAUTH2 token
    pub fn validate_token(&self) -> bool {
        !self.access_token.is_empty() && !self.email.is_empty()
    }

    /// Get the email address
    pub fn email(&self) -> &str {
        &self.email
    }

    /// Get the access token
    pub fn access_token(&self) -> &str {
        &self.access_token
    }
}

/// XOAUTH2 server response types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum XOAUTH2Response {
    /// Continue with authentication
    Continue,
    /// Server challenge
    Challenge(String),
    /// Authentication successful
    Success,
    /// Authentication failed
    Failure,
}

/// XOAUTH2 error codes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum XOAUTH2ErrorCode {
    /// Invalid request
    InvalidRequest,
    /// Invalid client
    InvalidClient,
    /// Invalid grant
    InvalidGrant,
    /// Unauthorized client
    UnauthorizedClient,
    /// Unsupported grant type
    UnsupportedGrantType,
    /// Invalid scope
    InvalidScope,
    /// Access denied
    AccessDenied,
    /// Server error
    ServerError,
    /// Temporarily unavailable
    TemporarilyUnavailable,
}

impl XOAUTH2ErrorCode {
    /// Parse error code from XOAUTH2 response
    pub fn from_response(response: &str) -> Option<Self> {
        if response.contains("invalid_request") {
            Some(Self::InvalidRequest)
        } else if response.contains("invalid_client") {
            Some(Self::InvalidClient)
        } else if response.contains("invalid_grant") {
            Some(Self::InvalidGrant)
        } else if response.contains("unauthorized_client") {
            Some(Self::UnauthorizedClient)
        } else if response.contains("unsupported_grant_type") {
            Some(Self::UnsupportedGrantType)
        } else if response.contains("invalid_scope") {
            Some(Self::InvalidScope)
        } else if response.contains("access_denied") {
            Some(Self::AccessDenied)
        } else if response.contains("server_error") {
            Some(Self::ServerError)
        } else if response.contains("temporarily_unavailable") {
            Some(Self::TemporarilyUnavailable)
        } else {
            None
        }
    }

    /// Get error description
    pub fn description(&self) -> &'static str {
        match self {
            Self::InvalidRequest => "The request is missing a required parameter, includes an invalid parameter value, or is otherwise malformed",
            Self::InvalidClient => "Client authentication failed",
            Self::InvalidGrant => "The provided authorization grant is invalid, expired, or revoked",
            Self::UnauthorizedClient => "The authenticated client is not authorized to use this authorization grant type",
            Self::UnsupportedGrantType => "The authorization grant type is not supported by the authorization server",
            Self::InvalidScope => "The requested scope is invalid, unknown, or malformed",
            Self::AccessDenied => "The resource owner or authorization server denied the request",
            Self::ServerError => "The authorization server encountered an unexpected condition",
            Self::TemporarilyUnavailable => "The authorization server is currently unable to handle the request",
        }
    }
}

/// XOAUTH2 token refresh helper
pub struct XOAUTH2TokenRefresh {
    /// Email address
    email: String,
    /// Refresh token
    refresh_token: String,
    /// Client ID
    client_id: String,
    /// Client secret
    client_secret: String,
}

impl XOAUTH2TokenRefresh {
    /// Create a new token refresh helper
    pub fn new(
        email: String,
        refresh_token: String,
        client_id: String,
        client_secret: String,
    ) -> Self {
        Self {
            email,
            refresh_token,
            client_id,
            client_secret,
        }
    }

    /// Refresh the access token
    pub async fn refresh_token(&self) -> AsgardResult<String> {
        let client = reqwest::Client::new();
        
        let params = [
            ("grant_type", "refresh_token"),
            ("refresh_token", &self.refresh_token),
            ("client_id", &self.client_id),
            ("client_secret", &self.client_secret),
        ];
        
        let response = client
            .post("https://oauth2.googleapis.com/token")
            .form(&params)
            .send()
            .await?;
        
        if response.status().is_success() {
            let token_response: serde_json::Value = response.json().await?;
            
            if let Some(access_token) = token_response.get("access_token").and_then(|v| v.as_str()) {
                Ok(access_token.to_string())
            } else {
                Err(AsgardError::auth("No access token in response"))
            }
        } else {
            let error_text = response.text().await?;
            Err(AsgardError::auth(format!("Token refresh failed: {}", error_text)))
        }
    }

    /// Get the email address
    pub fn email(&self) -> &str {
        &self.email
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xoauth2_creation() {
        let xoauth2 = XOAUTH2::new(
            "test@gmail.com".to_string(),
            "test_access_token".to_string(),
        );
        
        assert_eq!(xoauth2.email(), "test@gmail.com");
        assert_eq!(xoauth2.access_token(), "test_access_token");
        assert!(xoauth2.validate_token());
    }

    #[test]
    fn test_xoauth2_imap_auth_string() {
        let xoauth2 = XOAUTH2::new(
            "test@gmail.com".to_string(),
            "test_access_token".to_string(),
        );
        
        let auth_string = xoauth2.generate_imap_auth_string().unwrap();
        assert!(auth_string.starts_with("AUTHENTICATE XOAUTH2 "));
    }

    #[test]
    fn test_xoauth2_smtp_auth_string() {
        let xoauth2 = XOAUTH2::new(
            "test@gmail.com".to_string(),
            "test_access_token".to_string(),
        );
        
        let auth_string = xoauth2.generate_smtp_auth_string().unwrap();
        assert!(!auth_string.is_empty());
    }

    #[test]
    fn test_parse_server_response() {
        assert_eq!(
            XOAUTH2::parse_server_response("+").unwrap(),
            XOAUTH2Response::Continue
        );
        
        assert_eq!(
            XOAUTH2::parse_server_response("+OK").unwrap(),
            XOAUTH2Response::Success
        );
        
        assert_eq!(
            XOAUTH2::parse_server_response("+NO").unwrap(),
            XOAUTH2Response::Failure
        );
    }

    #[test]
    fn test_xoauth2_error_codes() {
        assert_eq!(
            XOAUTH2ErrorCode::from_response("invalid_request"),
            Some(XOAUTH2ErrorCode::InvalidRequest)
        );
        
        assert_eq!(
            XOAUTH2ErrorCode::from_response("invalid_client"),
            Some(XOAUTH2ErrorCode::InvalidClient)
        );
        
        assert_eq!(
            XOAUTH2ErrorCode::from_response("access_denied"),
            Some(XOAUTH2ErrorCode::AccessDenied)
        );
        
        assert_eq!(
            XOAUTH2ErrorCode::from_response("unknown_error"),
            None
        );
    }

    #[test]
    fn test_xoauth2_error_descriptions() {
        assert!(!XOAUTH2ErrorCode::InvalidRequest.description().is_empty());
        assert!(!XOAUTH2ErrorCode::InvalidClient.description().is_empty());
        assert!(!XOAUTH2ErrorCode::AccessDenied.description().is_empty());
    }

    #[test]
    fn test_xoauth2_token_refresh_creation() {
        let refresh = XOAUTH2TokenRefresh::new(
            "test@gmail.com".to_string(),
            "test_refresh_token".to_string(),
            "test_client_id".to_string(),
            "test_client_secret".to_string(),
        );
        
        assert_eq!(refresh.email(), "test@gmail.com");
    }

    #[test]
    fn test_invalid_xoauth2() {
        let xoauth2 = XOAUTH2::new(
            "".to_string(),
            "".to_string(),
        );
        
        assert!(!xoauth2.validate_token());
    }
}
