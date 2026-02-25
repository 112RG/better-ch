//! OAuth2 client-credentials authentication for Anypoint Platform.

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

use crate::error::{AuthError, Error};

/// OAuth2 authenticator for Anypoint Platform.
pub struct Authenticator {
    platform_url: String,
    client_id: String,
    client_secret: String,
}

impl Authenticator {
    pub fn new(
        platform_url: &str,
        client_id: impl Into<String>,
        client_secret: impl Into<String>,
    ) -> Result<Self, Error> {
        Ok(Self {
            platform_url: platform_url.to_string(),
            client_id: client_id.into(),
            client_secret: client_secret.into(),
        })
    }

    /// Exchange client credentials for an access token.
    pub async fn get_token(&self) -> Result<Token, Error> {
        let client = reqwest::Client::new();
        let token_url = format!("{}/accounts/api/v2/oauth2/token", self.platform_url);

        let params = [
            ("grant_type", "client_credentials"),
            ("client_id", &self.client_id),
            ("client_secret", &self.client_secret),
        ];

        let response = client
            .post(&token_url)
            .form(&params)
            .send()
            .await
            .map_err(|e| Error::Auth(AuthError::TokenFetch(e.to_string())))?;

        if !response.status().is_success() {
            let status = response.status();
            let message = response.text().await.unwrap_or_default();
            return Err(Error::Auth(AuthError::TokenFetch(format!("{}: {}", status, message))));
        }

        let token_resp: TokenResponse = response
            .json()
            .await
            .map_err(|e| Error::Auth(AuthError::TokenFetch(e.to_string())))?;

        let expires_at = Utc::now() + Duration::seconds(token_resp.expires_in as i64);

        Ok(Token {
            access_token: token_resp.access_token,
            expires_at,
            token_type: token_resp.token_type,
        })
    }
}

/// OAuth2 access token with expiration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Token {
    pub access_token: String,
    pub expires_at: DateTime<Utc>,
    pub token_type: String,
}

impl Token {
    pub fn is_expired(&self) -> bool {
        Utc::now() >= self.expires_at
    }

    pub fn expires_soon(&self, duration: Duration) -> bool {
        Utc::now() + duration >= self.expires_at
    }

    pub fn authorization(&self) -> String {
        format!("{} {}", self.token_type, self.access_token)
    }
}

#[derive(Debug, Deserialize)]
struct TokenResponse {
    #[serde(rename = "access_token")]
    access_token: String,
    #[serde(rename = "token_type")]
    token_type: String,
    #[serde(rename = "expires_in")]
    expires_in: u64,
}