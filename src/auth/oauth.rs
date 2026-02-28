//! OAuth2 authentication for Anypoint Platform.
//! Supports authorization code flow with PKCE for SSO login.

use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use chrono::{DateTime, Duration, Utc};
use rand::Rng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::error::{AuthError, Error};

/// OAuth2 authenticator for Anypoint Platform.
pub struct Authenticator {
    platform_url: String,
    client_id: String,
    client_secret: String,
    /// Local callback URL for OAuth redirect
    redirect_uri: String,
    /// Callback server port
    port: u16,
    /// Reusable HTTP client for token exchange and user info requests
    http_client: reqwest::Client,
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
            redirect_uri: "http://127.0.0.1:8082/callback".to_string(),
            port: 8082,
            http_client: reqwest::Client::new(),
        })
    }

    /// Set custom redirect URI (default: http://127.0.0.1:8082/callback)
    pub fn with_redirect_uri(mut self, redirect_uri: impl Into<String>) -> Self {
        self.redirect_uri = redirect_uri.into();
        self
    }

    /// Set custom port (default: 8082)
    pub fn with_port(mut self, port: u16) -> Self {
        self.port = port;
        // Keep the redirect URI in sync with the port when using a local loopback address.
        if self.redirect_uri.starts_with("http://127.0.0.1:") {
            self.redirect_uri = format!("http://127.0.0.1:{}/callback", port);
        }
        self
    }

    /// Generate PKCE code verifier and challenge
    fn generate_pkce() -> (String, String) {
        // Byte slice avoids O(n) UTF-8 scanning per character; all chars are ASCII.
        const CHARSET: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
        let mut rng = rand::thread_rng();
        let verifier: String = (0..32)
            .map(|_| CHARSET[rng.gen_range(0..CHARSET.len())] as char)
            .collect();

        // Derive code_challenge = BASE64URL(SHA256(code_verifier)) per RFC 7636.
        let mut hasher = Sha256::new();
        hasher.update(verifier.as_bytes());
        let hash = hasher.finalize();
        let challenge = URL_SAFE_NO_PAD.encode(hash);

        (verifier, challenge)
    }

    /// Generate a random state parameter for CSRF protection
    fn generate_state() -> String {
        const CHARSET: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
        let mut rng = rand::thread_rng();
        (0..32)
            .map(|_| CHARSET[rng.gen_range(0..CHARSET.len())] as char)
            .collect()
    }

    /// Build the authorization URL for SSO login
    /// User opens this in a browser to authenticate via their IdP
    pub fn build_authorization_url(&self) -> (String, String, String) {
        let (code_verifier, code_challenge) = Self::generate_pkce();
        let state = Self::generate_state();

        let params = [
            ("response_type", "code"),
            ("client_id", &self.client_id),
            ("redirect_uri", &self.redirect_uri),
            ("scope", "full"),
            ("state", &state),
            ("code_challenge", &code_challenge),
            ("code_challenge_method", "S256"),
        ];

        let query_string = params
            .iter()
            .map(|(k, v)| format!("{}={}", k, urlencoding::encode(v)))
            .collect::<Vec<_>>()
            .join("&");

        let auth_url = format!(
            "{}/accounts/api/v2/oauth2/authorize?{}",
            self.platform_url, query_string
        );

        (auth_url, code_verifier, state)
    }

    /// Start OAuth flow: opens browser for login and waits for callback.
    /// This is the main entry point for SSO authentication.
    ///
    /// Returns the access token and user info after successful authentication.
    pub async fn login_with_sso(&self) -> Result<(Token, User), Error> {
        let (auth_url, code_verifier, state) = self.build_authorization_url();

        tracing::info!("Opening browser for login...");
        tracing::info!("URL: {}", auth_url);

        // Open browser
        #[cfg(target_os = "windows")]
        {
            if let Err(e) = std::process::Command::new("cmd")
                .args(["/c", "start", "", &auth_url])
                .spawn()
            {
                tracing::warn!(
                    "Failed to open browser automatically: {}. Open this URL manually: {}",
                    e,
                    auth_url
                );
            }
        }
        #[cfg(target_os = "macos")]
        {
            if let Err(e) = std::process::Command::new("open").arg(&auth_url).spawn() {
                tracing::warn!(
                    "Failed to open browser automatically: {}. Open this URL manually: {}",
                    e,
                    auth_url
                );
            }
        }
        #[cfg(target_os = "linux")]
        {
            if let Err(e) = std::process::Command::new("xdg-open")
                .arg(&auth_url)
                .spawn()
            {
                tracing::warn!(
                    "Failed to open browser automatically: {}. Open this URL manually: {}",
                    e,
                    auth_url
                );
            }
        }

        let code = self.wait_for_callback(&state)?;

        tracing::info!("Received authorization code, exchanging for token...");

        let token = self.exchange_code_for_token(&code, &code_verifier).await?;
        let user = self.get_current_user(&token).await?;

        tracing::info!("Successfully logged in as: {}", user.display_name());

        Ok((token, user))
    }

    /// Wait for the OAuth callback on the local server.
    ///
    /// Validates the `state` parameter against `expected_state` to prevent CSRF attacks.
    pub fn wait_for_callback(&self, expected_state: &str) -> Result<String, Error> {
        use std::io::{BufRead, BufReader};
        use std::net::TcpListener;

        let addr = format!("127.0.0.1:{}", self.port);
        let listener = TcpListener::bind(&addr).map_err(|e| {
            Error::Auth(AuthError::TokenFetch(format!(
                "Failed to bind to port {}: {}",
                self.port, e
            )))
        })?;

        // Time out after 2 minutes if the browser never redirects.
        listener.set_nonblocking(true).map_err(|e| {
            Error::Auth(AuthError::TokenFetch(format!(
                "Failed to set non-blocking: {}",
                e
            )))
        })?;

        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(120);
        let (stream, _) = loop {
            match listener.accept() {
                Ok(conn) => break conn,
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    if std::time::Instant::now() >= deadline {
                        return Err(Error::Auth(AuthError::TokenFetch(
                            "OAuth callback timed out after 2 minutes".to_string(),
                        )));
                    }
                    std::thread::sleep(std::time::Duration::from_millis(100));
                }
                Err(e) => {
                    return Err(Error::Auth(AuthError::TokenFetch(format!(
                        "Failed to accept connection: {}",
                        e
                    ))));
                }
            }
        };

        let reader = BufReader::new(stream);
        let mut lines = reader.lines();

        // Read the HTTP request line
        let request_line = lines.next().ok_or_else(|| {
            Error::Auth(AuthError::TokenFetch("Failed to read request".to_string()))
        })??;

        tracing::info!("Received callback: {}", request_line);

        // Parse the URL to extract code and state parameters
        if let Some(query) = request_line.split_whitespace().nth(1)
            && query.starts_with("/callback?")
        {
            let query_string = query.trim_start_matches("/callback?");
            let mut code: Option<String> = None;
            let mut received_state: Option<String> = None;

            for param in query_string.split('&') {
                let parts: Vec<&str> = param.splitn(2, '=').collect();
                if parts.len() == 2 {
                    match parts[0] {
                        "code" => {
                            code = Some(
                                urlencoding::decode(parts[1])
                                    .map_err(|e| Error::Auth(AuthError::TokenFetch(e.to_string())))?
                                    .to_string(),
                            );
                        }
                        "state" => {
                            received_state = Some(
                                urlencoding::decode(parts[1])
                                    .map_err(|e| Error::Auth(AuthError::TokenFetch(e.to_string())))?
                                    .to_string(),
                            );
                        }
                        _ => {}
                    }
                }
            }

            // CSRF check: received state must match the state we generated
            if received_state.as_deref() != Some(expected_state) {
                return Err(Error::Auth(AuthError::TokenFetch(
                    "OAuth state mismatch: possible CSRF attack".to_string(),
                )));
            }

            if let Some(c) = code {
                return Ok(c);
            }
        }

        Err(Error::Auth(AuthError::TokenFetch(
            "No authorization code found in callback".to_string(),
        )))
    }

    /// Exchange authorization code for access token
    pub async fn exchange_code_for_token(
        &self,
        code: &str,
        code_verifier: &str,
    ) -> Result<Token, Error> {
        let token_url = format!("{}/accounts/api/v2/oauth2/token", self.platform_url);

        let params = [
            ("grant_type", "authorization_code"),
            ("client_id", &self.client_id),
            ("client_secret", &self.client_secret),
            ("code", code),
            ("redirect_uri", &self.redirect_uri),
            ("code_verifier", code_verifier),
        ];

        let response = self
            .http_client
            .post(&token_url)
            .form(&params)
            .send()
            .await
            .map_err(|e| Error::Auth(AuthError::TokenFetch(e.to_string())))?;

        if !response.status().is_success() {
            let status = response.status();
            let message = response.text().await.unwrap_or_default();
            return Err(Error::Auth(AuthError::TokenFetch(format!(
                "{}: {}",
                status, message
            ))));
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

    /// Exchange client credentials for an access token (non-SSO).
    pub async fn get_token(&self) -> Result<Token, Error> {
        let token_url = format!("{}/accounts/api/v2/oauth2/token", self.platform_url);

        let params = [
            ("grant_type", "client_credentials"),
            ("client_id", &self.client_id),
            ("client_secret", &self.client_secret),
        ];

        let response = self
            .http_client
            .post(&token_url)
            .form(&params)
            .send()
            .await
            .map_err(|e| Error::Auth(AuthError::TokenFetch(e.to_string())))?;

        if !response.status().is_success() {
            let status = response.status();
            let message = response.text().await.unwrap_or_default();
            return Err(Error::Auth(AuthError::TokenFetch(format!(
                "{}: {}",
                status, message
            ))));
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

    /// Authenticate with username and password (Resource Owner Password Grant).
    /// This does NOT require a Connected App - uses your Anypoint credentials directly.
    /// Note: This grant type may be disabled by your organization.
    pub async fn login_with_password(
        &self,
        username: &str,
        password: &str,
    ) -> Result<Token, Error> {
        let token_url = format!("{}/accounts/api/v2/oauth2/token", self.platform_url);

        let params = [
            ("grant_type", "password"),
            ("username", username),
            ("password", password),
            ("client_id", &self.client_id),
            ("client_secret", &self.client_secret),
        ];

        let response = self
            .http_client
            .post(&token_url)
            .form(&params)
            .send()
            .await
            .map_err(|e| Error::Auth(AuthError::TokenFetch(e.to_string())))?;

        if !response.status().is_success() {
            let status = response.status();
            let message = response.text().await.unwrap_or_default();
            return Err(Error::Auth(AuthError::TokenFetch(format!(
                "{}: {}",
                status, message
            ))));
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

    /// Get the current user info using the access token.
    /// This tells you WHO is using the app.
    pub async fn get_current_user(&self, token: &Token) -> Result<User, Error> {
        let user_url = format!("{}/accounts/api/me", self.platform_url);

        let response = self
            .http_client
            .get(&user_url)
            .header("Authorization", token.authorization())
            .send()
            .await
            .map_err(|e| Error::Auth(AuthError::TokenFetch(e.to_string())))?;

        if !response.status().is_success() {
            let status = response.status();
            let message = response.text().await.unwrap_or_default();
            return Err(Error::Auth(AuthError::TokenFetch(format!(
                "{}: {}",
                status, message
            ))));
        }

        let user: User = response
            .json()
            .await
            .map_err(|e| Error::Auth(AuthError::TokenFetch(e.to_string())))?;

        Ok(user)
    }
}

/// Current user information from Anypoint Platform.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    /// Unique user ID
    pub id: String,
    /// Anypoint username
    pub username: String,
    /// Email address
    pub email: String,
    /// First name
    pub first_name: Option<String>,
    /// Last name
    pub last_name: Option<String>,
    /// Organization ID
    pub organization_id: String,
    /// Full name (computed)
    #[serde(default)]
    pub full_name: Option<String>,
}

impl User {
    /// Get display name (full name or username)
    pub fn display_name(&self) -> &str {
        self.full_name
            .as_deref()
            .or(self.first_name.as_deref())
            .unwrap_or(&self.username)
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

/// Lightweight helpers used by tests to construct tokens.
impl Token {
    /// Create a token expiring `seconds_from_now` seconds from now.
    pub fn new_with_expiry(access_token: impl Into<String>, seconds_from_now: i64) -> Self {
        Token {
            access_token: access_token.into(),
            expires_at: Utc::now() + Duration::seconds(seconds_from_now),
            token_type: "Bearer".to_string(),
        }
    }

    /// Convenience: token not expired (1 hour)
    pub fn test_token_not_expired() -> Self {
        Self::new_with_expiry("test-token", 3600)
    }

    /// Convenience: expired token (-1 hour)
    pub fn test_token_expired() -> Self {
        Self::new_with_expiry("expired", -3600)
    }
}

#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
    token_type: String,
    expires_in: u64,
}
