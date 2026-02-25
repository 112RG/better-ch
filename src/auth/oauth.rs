//! OAuth2 client-credentials authentication for Anypoint Platform.

use chrono::{DateTime, Duration, Utc};
use oauth2::reqwest::async_http_client;
use oauth2::{
    ClientId, ClientSecret, EmptyExtraTokenFields, OAuth2, Scope, TokenResponse,
    TokenUrl,
};
use serde::{Deserialize, Serialize};

use crate::error::{ApiError, AuthError, Error};

/// OAuth2 authenticator for Anypoint Platform.
pub struct Authenticator {
    oauth: OAuth2<EmptyExtraTokenFields>,
    client_id: ClientId,
    client_secret: ClientSecret,
    token_url: TokenUrl,
    scopes: Vec<Scope>,
}

impl Authenticator {
    /// Create a new authenticator.
    pub fn new(
        platform_url: &str,
        client_id: impl Into<String>,
        client_secret: impl Into<String>,
    ) -> Result<Self, Error> {
        let client_id = ClientId::new(client_id.into());
        let client_secret = ClientSecret::new(client_secret.into());

        let token_url = TokenUrl::new(format!("{}/accounts/api/v2/oauth2/token", platform_url))
            .map_err(|e| Error::Auth(AuthError::TokenFetch(e.to_string())))?;

        let auth_url = format!("{}/accounts/api/v2/oauth2/authorize", platform_url);

        let oauth = OAuth2::new(
            oauth2::ClientId::new("cloudhub-api".to_string()),
            oauth2::ClientSecret::new("".to_string()),
            oauth2::AuthorizationUrl::new(auth_url).unwrap(),
            token_url.clone(),
        );

        let scopes = vec![
            Scope::new("read:applications".to_string()),
            Scope::new("write:applications".to_string()),
            Scope::new("read:alerts".to_string()),
        ];

        Ok(Self {
            oauth,
            client_id,
            client_secret,
            token_url,
            scopes,
        })
    }

    /// Exchange client credentials for an access token.
    pub async fn get_token(&self) -> Result<Token, Error> {
        let token_result = self
            .oauth
            .clone()
            .add_scopes(self.scopes.iter().cloned())
            .exchange_client_credentials()
            .set_client_id(&self.client_id)
            .set_client_secret(&self.client_secret)
            .request_async(async_http_client)
            .await
            .map_err(|e| Error::Auth(AuthError::TokenFetch(e.to_string())))?;

        let access_token = token_result
            .access_token()
            .secret()
            .to_string();

        let expires_in = token_result
            .expires_in()
            .map(|d| d.as_secs())
            .unwrap_or(3600);

        let expires_at = Utc::now() + Duration::seconds(expires_in as i64);

        Ok(Token {
            access_token,
            expires_at,
            token_type: "Bearer".to_string(),
        })
    }
}

/// OAuth2 access token with expiration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Token {
    /// The access token string.
    pub access_token: String,

    /// Token expiration time.
    pub expires_at: DateTime<Utc>,

    /// Token type (usually "Bearer").
    pub token_type: String,
}

impl Token {
    /// Check if the token is expired.
    pub fn is_expired(&self) -> bool {
        Utc::now() >= self.expires_at
    }

    /// Check if the token expires within the given duration.
    pub fn expires_soon(&self, duration: Duration) -> bool {
        Utc::now() + duration >= self.expires_at
    }

    /// Get the authorization header value.
    pub fn authorization(&self) -> String {
        format!("{} {}", self.token_type, self.access_token)
    }
}

/// Token response from Anypoint Platform API.
#[derive(Debug, Deserialize)]
struct TokenResponse {
    #[serde(rename = "access_token")]
    access_token: String,

    #[serde(rename = "token_type")]
    token_type: String,

    #[serde(rename = "expires_in")]
    expires_in: u64,
}