//! Gmail OAuth2 implementation

use crate::{OAuthConfig, AuthorizationUrl, AuthorizationCode, OAuthToken, TokenResponse};
use asgard_core::error::{AsgardError, AsgardResult};
use oauth2::{
    AuthorizationCode, ClientId, ClientSecret, CsrfToken, PkceCodeChallenge, PkceCodeVerifier,
    RedirectUrl, Scope, TokenResponse as OAuth2TokenResponse,
};
use oauth2::basic::BasicClient;
use oauth2::reqwest::async_http_client;
use rand::Rng;
use sha2::{Sha256, Digest};
use std::collections::HashMap;
use tokio::net::TcpListener;
use tokio::sync::oneshot;
use url::Url;

/// Gmail OAuth2 client
pub struct GmailOAuth {
    /// OAuth2 client
    client: BasicClient,
    /// Configuration
    config: OAuthConfig,
}

impl GmailOAuth {
    /// Create a new Gmail OAuth2 client
    pub fn new(config: OAuthConfig) -> Self {
        let client_id = ClientId::new(config.client_id.clone());
        let client_secret = ClientSecret::new(config.client_secret.clone());
        let redirect_url = RedirectUrl::new(config.redirect_uri.clone())
            .expect("Invalid redirect URI");

        let client = BasicClient::new(client_id, Some(client_secret), oauth2::AuthUrl::new("https://accounts.google.com/o/oauth2/v2/auth".to_string()).unwrap(), Some(oauth2::TokenUrl::new("https://oauth2.googleapis.com/token".to_string()).unwrap()))
            .set_redirect_uri(redirect_url);

        Self { client, config }
    }

    /// Generate authorization URL with PKCE
    pub fn get_authorization_url(&self) -> AsgardResult<AuthorizationUrl> {
        // Generate PKCE code verifier and challenge
        let code_verifier = PkceCodeVerifier::new_random();
        let code_challenge = PkceCodeChallenge::from_code_verifier_sha256(&code_verifier);

        // Generate state parameter
        let state = CsrfToken::new_random();

        // Build authorization URL
        let (auth_url, _) = self.client
            .authorize_url(CsrfToken::new(state.secret().clone()))
            .add_scope(Scope::new("https://www.googleapis.com/auth/gmail.readonly".to_string()))
            .add_scope(Scope::new("https://www.googleapis.com/auth/gmail.send".to_string()))
            .add_scope(Scope::new("https://www.googleapis.com/auth/gmail.modify".to_string()))
            .add_scope(Scope::new("https://www.googleapis.com/auth/gmail.labels".to_string()))
            .set_pkce_challenge(code_challenge)
            .url();

        Ok(AuthorizationUrl {
            url: auth_url.to_string(),
            state: state.secret().clone(),
            code_verifier: code_verifier.secret().clone(),
        })
    }

    /// Exchange authorization code for access token
    pub async fn exchange_code(&self, auth_code: &AuthorizationCode, code_verifier: &str) -> AsgardResult<OAuthToken> {
        let code = AuthorizationCode::new(auth_code.code.clone());
        let pkce_verifier = PkceCodeVerifier::new(code_verifier.to_string());

        let token_response = self.client
            .exchange_code(code)
            .set_pkce_verifier(pkce_verifier)
            .request_async(async_http_client)
            .await
            .map_err(|e| AsgardError::oauth(e))?;

        let access_token = token_response.access_token().secret().clone();
        let refresh_token = token_response.refresh_token().map(|t| t.secret().clone());
        let token_type = token_response.token_type().and_then(|t| t.as_str()).unwrap_or("Bearer").to_string();
        let expires_in = token_response.expires_in();
        let scope = token_response.scopes().map(|scopes| {
            scopes.iter()
                .map(|s| s.to_string())
                .collect::<Vec<_>>()
                .join(" ")
        });

        let expires_at = expires_in.map(|duration| {
            time::OffsetDateTime::now_utc() + time::Duration::seconds(duration.as_secs() as i64)
        });

        Ok(OAuthToken {
            access_token,
            refresh_token,
            token_type,
            expires_at,
            scope,
        })
    }

    /// Refresh access token
    pub async fn refresh_token(&self, refresh_token: &str) -> AsgardResult<OAuthToken> {
        let refresh_token = oauth2::RefreshToken::new(refresh_token.to_string());

        let token_response = self.client
            .exchange_refresh_token(&refresh_token)
            .request_async(async_http_client)
            .await
            .map_err(|e| AsgardError::oauth(e))?;

        let access_token = token_response.access_token().secret().clone();
        let new_refresh_token = token_response.refresh_token().map(|t| t.secret().clone());
        let token_type = token_response.token_type().and_then(|t| t.as_str()).unwrap_or("Bearer").to_string();
        let expires_in = token_response.expires_in();
        let scope = token_response.scopes().map(|scopes| {
            scopes.iter()
                .map(|s| s.to_string())
                .collect::<Vec<_>>()
                .join(" ")
        });

        let expires_at = expires_in.map(|duration| {
            time::OffsetDateTime::now_utc() + time::Duration::seconds(duration.as_secs() as i64)
        });

        Ok(OAuthToken {
            access_token,
            refresh_token: new_refresh_token.or(Some(refresh_token.secret().clone())),
            token_type,
            expires_at,
            scope,
        })
    }

    /// Start local server to handle OAuth callback
    pub async fn start_callback_server(&self, port: u16) -> AsgardResult<oneshot::Receiver<AuthorizationCode>> {
        let listener = TcpListener::bind(format!("127.0.0.1:{}", port)).await?;
        let (tx, rx) = oneshot::channel();

        tokio::spawn(async move {
            if let Ok((stream, _)) = listener.accept().await {
                let mut stream = tokio::io::BufReader::new(stream);
                let mut request_line = String::new();
                
                if let Ok(_) = tokio::io::AsyncBufReadExt::read_line(&mut stream, &mut request_line).await {
                    if let Some((method, path)) = request_line.split_once(' ') {
                        if method == "GET" && path.starts_with("/?") {
                            let query_string = &path[2..];
                            let params: HashMap<String, String> = url::form_urlencoded::parse(query_string.as_bytes())
                                .into_owned()
                                .collect();

                            if let (Some(code), Some(state)) = (params.get("code"), params.get("state")) {
                                let auth_code = AuthorizationCode {
                                    code: code.clone(),
                                    state: state.clone(),
                                };

                                // Send success response
                                let response = "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\n\r\n<html><body><h1>Authorization successful!</h1><p>You can close this window and return to Asgard Mail.</p></body></html>";
                                let _ = stream.write_all(response.as_bytes()).await;

                                let _ = tx.send(auth_code);
                                return;
                            }
                        }
                    }
                }

                // Send error response
                let response = "HTTP/1.1 400 Bad Request\r\nContent-Type: text/html\r\n\r\n<html><body><h1>Authorization failed!</h1><p>Please try again.</p></body></html>";
                let _ = stream.write_all(response.as_bytes()).await;
            }
        });

        Ok(rx)
    }

    /// Open authorization URL in default browser
    pub fn open_authorization_url(&self, auth_url: &AuthorizationUrl) -> AsgardResult<()> {
        open::that(&auth_url.url)?;
        Ok(())
    }

    /// Complete OAuth flow
    pub async fn complete_oauth_flow(&self) -> AsgardResult<OAuthToken> {
        // Generate authorization URL
        let auth_url = self.get_authorization_url()?;

        // Start callback server
        let callback_rx = self.start_callback_server(8080).await?;

        // Open authorization URL in browser
        self.open_authorization_url(&auth_url)?;

        // Wait for callback
        let auth_code = callback_rx.await
            .map_err(|_| AsgardError::oauth("OAuth callback failed"))?;

        // Verify state parameter
        if auth_code.state != auth_url.state {
            return Err(AsgardError::oauth("Invalid state parameter"));
        }

        // Exchange code for token
        let token = self.exchange_code(&auth_code, &auth_url.code_verifier).await?;

        Ok(token)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gmail_oauth_creation() {
        let config = OAuthConfig {
            client_id: "test-client-id".to_string(),
            client_secret: "test-client-secret".to_string(),
            redirect_uri: "http://127.0.0.1:8080".to_string(),
            scopes: vec!["https://www.googleapis.com/auth/gmail.readonly".to_string()],
        };

        let oauth = GmailOAuth::new(config);
        assert!(oauth.client.client_id().secret() == "test-client-id");
    }

    #[test]
    fn test_authorization_url_generation() {
        let config = OAuthConfig {
            client_id: "test-client-id".to_string(),
            client_secret: "test-client-secret".to_string(),
            redirect_uri: "http://127.0.0.1:8080".to_string(),
            scopes: vec!["https://www.googleapis.com/auth/gmail.readonly".to_string()],
        };

        let oauth = GmailOAuth::new(config);
        let auth_url = oauth.get_authorization_url().unwrap();
        
        assert!(!auth_url.url.is_empty());
        assert!(!auth_url.state.is_empty());
        assert!(!auth_url.code_verifier.is_empty());
        assert!(auth_url.url.contains("accounts.google.com"));
    }

    #[test]
    fn test_oauth_token_expiration() {
        let token = OAuthToken {
            access_token: "test-token".to_string(),
            refresh_token: None,
            token_type: "Bearer".to_string(),
            expires_at: Some(time::OffsetDateTime::now_utc() + time::Duration::hours(1)),
            scope: None,
        };

        assert!(!token.is_expired());
        assert!(!token.expires_soon());

        let expired_token = OAuthToken {
            access_token: "test-token".to_string(),
            refresh_token: None,
            token_type: "Bearer".to_string(),
            expires_at: Some(time::OffsetDateTime::now_utc() - time::Duration::hours(1)),
            scope: None,
        };

        assert!(expired_token.is_expired());
    }
}
