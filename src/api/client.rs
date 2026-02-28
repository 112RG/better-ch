//! HTTP client for CloudHub API.

use reqwest::Client;
use serde::Deserialize;
use serde::de::DeserializeOwned;

use crate::api::models::*;
use crate::auth::oauth::Token;
use crate::error::{ApiError, Error};

/// CloudHub API client.
pub struct CloudHubClient {
    client: Client,
    base_url: String,
    token: Option<Token>,
}

impl CloudHubClient {
    /// Create a new `CloudHubClient` targeting the given base URL.
    ///
    /// # Errors
    ///
    /// Returns `Error::Api` if the underlying HTTP client cannot be constructed
    /// (e.g. TLS initialization failure).
    pub fn new(base_url: &str) -> Result<Self, Error> {
        /// Default HTTP request timeout in seconds.
        const HTTP_REQUEST_TIMEOUT_SECS: u64 = 30;
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(HTTP_REQUEST_TIMEOUT_SECS))
            .build()
            .map_err(|e| Error::Api(ApiError::Request(e.to_string())))?;
        Ok(Self {
            client,
            base_url: base_url.to_string(),
            token: None,
        })
    }

    /// Set the authentication token used for subsequent API requests.
    pub fn set_token(&mut self, token: Token) {
        self.token = Some(token);
    }

    /// Return a reference to the current token, if any.
    pub fn token(&self) -> Option<&Token> {
        self.token.as_ref()
    }

    /// Return `true` if a non-expired token is present.
    pub fn is_authenticated(&self) -> bool {
        self.token
            .as_ref()
            .map(|t| !t.is_expired())
            .unwrap_or(false)
    }

    /// Send a pre-built request and deserialize the response body.
    ///
    /// # Errors
    ///
    /// Returns `Error::Api` for HTTP failures, non-success status codes, or deserialization errors.
    async fn execute_request<T: DeserializeOwned>(
        &self,
        builder: reqwest::RequestBuilder,
        path: &str,
    ) -> Result<T, Error> {
        let response = builder
            .send()
            .await
            .map_err(|e| Error::Api(ApiError::Request(e.to_string())))?;

        let status = response.status();

        if status.is_success() {
            response
                .json()
                .await
                .map_err(|e| Error::Api(ApiError::Request(e.to_string())))
        } else if status.as_u16() == 404 {
            Err(Error::Api(ApiError::NotFound(path.to_string())))
        } else {
            let message = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            Err(Error::Api(ApiError::ServerError {
                status: status.as_u16(),
                message,
            }))
        }
    }

    /// Send an authenticated GET/DELETE/POST request without a body.
    async fn request<T: DeserializeOwned>(
        &self,
        method: reqwest::Method,
        path: &str,
    ) -> Result<T, Error> {
        let token = self
            .token
            .as_ref()
            .ok_or(Error::Auth(crate::error::AuthError::TokenExpired))?;
        let url = format!("{}{}", self.base_url, path);

        let builder = self
            .client
            .request(method, &url)
            .header("Authorization", token.authorization())
            .header("Content-Type", "application/json");

        self.execute_request(builder, path).await
    }

    /// Send an authenticated request with a JSON body.
    async fn request_with_body<T: DeserializeOwned, B: serde::Serialize>(
        &self,
        method: reqwest::Method,
        path: &str,
        body: &B,
    ) -> Result<T, Error> {
        let token = self
            .token
            .as_ref()
            .ok_or(Error::Auth(crate::error::AuthError::TokenExpired))?;
        let url = format!("{}{}", self.base_url, path);

        let builder = self
            .client
            .request(method, &url)
            .header("Authorization", token.authorization())
            .header("Content-Type", "application/json")
            .json(body);

        self.execute_request(builder, path).await
    }

    // Applications API

    /// List all deployed applications.
    ///
    /// # Errors
    ///
    /// Returns `Error::Auth` if not authenticated, or `Error::Api` on HTTP/JSON failure.
    pub async fn list_applications(&self) -> Result<Vec<Application>, Error> {
        #[derive(Deserialize)]
        struct Response {
            data: Option<Vec<Application>>,
        }
        let response: Response = self.request(reqwest::Method::GET, "/applications").await?;
        Ok(response.data.unwrap_or_default())
    }

    /// Retrieve a single application by name.
    ///
    /// # Errors
    ///
    /// Returns `Error::Api(ApiError::NotFound)` if the application does not exist.
    pub async fn get_application(&self, name: &str) -> Result<Application, Error> {
        self.request(reqwest::Method::GET, &format!("/applications/{}", name))
            .await
    }

    /// Deploy a new application.
    ///
    /// # Errors
    ///
    /// Returns `Error::Api` on HTTP or JSON failure.
    pub async fn create_application(
        &self,
        request: CreateApplicationRequest,
    ) -> Result<Application, Error> {
        self.request_with_body(reqwest::Method::POST, "/applications", &request)
            .await
    }

    /// Update an existing application's configuration.
    ///
    /// # Errors
    ///
    /// Returns `Error::Api(ApiError::NotFound)` if the application does not exist.
    pub async fn update_application(
        &self,
        name: &str,
        request: UpdateApplicationRequest,
    ) -> Result<Application, Error> {
        self.request_with_body(
            reqwest::Method::PUT,
            &format!("/applications/{}", name),
            &request,
        )
        .await
    }

    /// Undeploy and permanently delete an application.
    ///
    /// # Errors
    ///
    /// Returns `Error::Api(ApiError::NotFound)` if the application does not exist.
    pub async fn delete_application(&self, name: &str) -> Result<(), Error> {
        let _response: StatusResponse = self
            .request(reqwest::Method::DELETE, &format!("/applications/{}", name))
            .await?;
        Ok(())
    }

    /// Start a stopped application.
    ///
    /// # Errors
    ///
    /// Returns `Error::Api` on HTTP failure or if the application is not found.
    pub async fn start_application(&self, name: &str) -> Result<StatusResponse, Error> {
        self.request(
            reqwest::Method::POST,
            &format!("/applications/{}/start", name),
        )
        .await
    }

    /// Stop a running application.
    ///
    /// # Errors
    ///
    /// Returns `Error::Api` on HTTP failure or if the application is not found.
    pub async fn stop_application(&self, name: &str) -> Result<StatusResponse, Error> {
        self.request(
            reqwest::Method::POST,
            &format!("/applications/{}/stop", name),
        )
        .await
    }

    /// Restart a running application.
    ///
    /// # Errors
    ///
    /// Returns `Error::Api` on HTTP failure or if the application is not found.
    pub async fn restart_application(&self, name: &str) -> Result<StatusResponse, Error> {
        self.request(
            reqwest::Method::POST,
            &format!("/applications/{}/restart", name),
        )
        .await
    }

    // Statistics API

    /// Retrieve runtime statistics for an application.
    ///
    /// # Errors
    ///
    /// Returns `Error::Api` on HTTP failure or if the application is not found.
    pub async fn get_application_stats(&self, name: &str) -> Result<ApplicationStats, Error> {
        self.request(
            reqwest::Method::GET,
            &format!("/applications/{}/statistics", name),
        )
        .await
    }

    // Logs API

    /// Retrieve log entries for an application.
    ///
    /// # Arguments
    ///
    /// * `name` - Application name
    /// * `limit` - Maximum number of log entries to return; `None` uses the server default
    ///
    /// # Errors
    ///
    /// Returns `Error::Api` on HTTP failure or if the application is not found.
    pub async fn get_application_logs(
        &self,
        name: &str,
        limit: Option<u32>,
    ) -> Result<Vec<LogEntry>, Error> {
        let path = match limit {
            Some(n) => format!("/applications/{}/logs?limit={}", name, n),
            None => format!("/applications/{}/logs", name),
        };
        #[derive(Deserialize)]
        struct Response {
            data: Option<Vec<LogEntry>>,
        }
        let response: Response = self.request(reqwest::Method::GET, &path).await?;
        Ok(response.data.unwrap_or_default())
    }

    // Deployments API

    /// List all deployments for an application.
    ///
    /// # Errors
    ///
    /// Returns `Error::Api` on HTTP failure or if the application is not found.
    pub async fn list_deployments(&self, name: &str) -> Result<Vec<Deployment>, Error> {
        #[derive(Deserialize)]
        struct Response {
            data: Option<Vec<Deployment>>,
        }
        let response: Response = self
            .request(
                reqwest::Method::GET,
                &format!("/applications/{}/deployments", name),
            )
            .await?;
        Ok(response.data.unwrap_or_default())
    }

    // Instances API

    /// List all running instances of an application.
    ///
    /// # Errors
    ///
    /// Returns `Error::Api` on HTTP failure or if the application is not found.
    pub async fn list_instances(&self, name: &str) -> Result<Vec<Instance>, Error> {
        #[derive(Deserialize)]
        struct Response {
            data: Option<Vec<Instance>>,
        }
        let response: Response = self
            .request(
                reqwest::Method::GET,
                &format!("/applications/{}/instances", name),
            )
            .await?;
        Ok(response.data.unwrap_or_default())
    }

    // Alerts API

    /// List all configured alerts.
    ///
    /// # Errors
    ///
    /// Returns `Error::Api` on HTTP failure.
    pub async fn list_alerts(&self) -> Result<Vec<Alert>, Error> {
        #[derive(Deserialize)]
        struct Response {
            data: Option<Vec<Alert>>,
        }
        let response: Response = self.request(reqwest::Method::GET, "/alerts").await?;
        Ok(response.data.unwrap_or_default())
    }
}
