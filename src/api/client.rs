//! HTTP client for CloudHub API.

use reqwest::Client;
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
    /// Create a new CloudHub API client.
    pub fn new(base_url: &str) -> Self {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("failed to create HTTP client");

        Self {
            client,
            base_url: base_url.to_string(),
            token: None,
        }
    }

    /// Set the authentication token.
    pub fn set_token(&mut self, token: Token) {
        self.token = Some(token);
    }

    /// Get the current token.
    pub fn token(&self) -> Option<&Token> {
        self.token.as_ref()
    }

    /// Check if authenticated.
    pub fn is_authenticated(&self) -> bool {
        self.token
            .as_ref()
            .map(|t| !t.is_expired())
            .unwrap_or(false)
    }

    /// Make an authenticated request.
    async fn request<T: DeserializeOwned>(
        &self,
        method: reqwest::Method,
        path: &str,
    ) -> Result<T, Error> {
        let token = self
            .token
            .as_ref()
            .ok_or_else(|| Error::Auth(crate::error::AuthError::TokenExpired))?;

        let url = format!("{}{}", self.base_url, path);

        let response = self
            .client
            .request(method, &url)
            .header("Authorization", token.authorization())
            .header("Content-Type", "application/json")
            .send()
            .await
            .map_err(|e| Error::Api(ApiError::Request(e.to_string())))?;

        let status = response.status();

        if status.is_success() {
            response
                .json()
                .await
                .map_err(|e| Error::Api(ApiError::Json(e)))
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

    /// Make a request with body.
    async fn request_with_body<T: DeserializeOwned, B: serde::Serialize>(
        &self,
        method: reqwest::Method,
        path: &str,
        body: &B,
    ) -> Result<T, Error> {
        let token = self
            .token
            .as_ref()
            .ok_or_else(|| Error::Auth(crate::error::AuthError::TokenExpired))?;

        let url = format!("{}{}", self.base_url, path);

        let response = self
            .client
            .request(method, &url)
            .header("Authorization", token.authorization())
            .header("Content-Type", "application/json")
            .json(body)
            .send()
            .await
            .map_err(|e| Error::Api(ApiError::Request(e.to_string())))?;

        let status = response.status();

        if status.is_success() {
            response
                .json()
                .await
                .map_err(|e| Error::Api(ApiError::Json(e)))
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

    // ==================== Applications API ====================

    /// List all applications.
    pub async fn list_applications(&self) -> Result<Vec<Application>, Error> {
        #[derive(Deserialize)]
        struct Response {
            #[serde(rename = "data")]
            data: Option<Vec<Application>>,
        }

        let response: Response = self
            .request(reqwest::Method::GET, "/applications")
            .await?;

        Ok(response.data.unwrap_or_default())
    }

    /// Get application by name.
    pub async fn get_application(&self, name: &str) -> Result<Application, Error> {
        self.request(reqwest::Method::GET, &format!("/applications/{}", name))
            .await
    }

    /// Create a new application.
    pub async fn create_application(
        &self,
        request: CreateApplicationRequest,
    ) -> Result<Application, Error> {
        self.request_with_body(reqwest::Method::POST, "/applications", &request)
            .await
    }

    /// Update an application.
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

    /// Delete an application.
    pub async fn delete_application(&self, name: &str) -> Result<(), Error> {
        let _response: StatusResponse = self
            .request(reqwest::Method::DELETE, &format!("/applications/{}", name))
            .await?;
        Ok(())
    }

    /// Start an application.
    pub async fn start_application(&self, name: &str) -> Result<StatusResponse, Error> {
        self.request(
            reqwest::Method::POST,
            &format!("/applications/{}/start", name),
        )
        .await
    }

    /// Stop an application.
    pub async fn stop_application(&self, name: &str) -> Result<StatusResponse, Error> {
        self.request(
            reqwest::Method::POST,
            &format!("/applications/{}/stop", name),
        )
        .await
    }

    /// Restart an application.
    pub async fn restart_application(&self, name: &str) -> Result<StatusResponse, Error> {
        self.request(
            reqwest::Method::POST,
            &format!("/applications/{}/restart", name),
        )
        .await
    }

    // ==================== Statistics API ====================

    /// Get application statistics.
    pub async fn get_application_stats(
        &self,
        name: &str,
    ) -> Result<ApplicationStats, Error> {
        self.request(
            reqwest::Method::GET,
            &format!("/applications/{}/statistics", name),
        )
        .await
    }

    // ==================== Logs API ====================

    /// Get application logs.
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
            #[serde(rename = "data")]
            data: Option<Vec<LogEntry>>,
        }

        let response: Response = self.request(reqwest::Method::GET, &path).await?;

        Ok(response.data.unwrap_or_default())
    }

    // ==================== Deployments API ====================

    /// List deployments for an application.
    pub async fn list_deployments(
        &self,
        name: &str,
    ) -> Result<Vec<Deployment>, Error> {
        #[derive(Deserialize)]
        struct Response {
            #[serde(rename = "data")]
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

    /// Get deployment by ID.
    pub async fn get_deployment(
        &self,
        name: &str,
        deployment_id: &str,
    ) -> Result<Deployment, Error> {
        self.request(
            reqwest::Method::GET,
            &format!("/applications/{}/deployments/{}", name, deployment_id),
        )
        .await
    }

    // ==================== Instances API ====================

    /// List instances for an application.
    pub async fn list_instances(&self, name: &str) -> Result<Vec<Instance>, Error> {
        #[derive(Deserialize)]
        struct Response {
            #[serde(rename = "data")]
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

    // ==================== Alerts API ====================

    /// List alerts.
    pub async fn list_alerts(&self) -> Result<Vec<Alert>, Error> {
        #[derive(Deserialize)]
        struct Response {
            #[serde(rename = "data")]
            data: Option<Vec<Alert>>,
        }

        let response: Response = self
            .request(reqwest::Method::GET, "/alerts")
            .await?;

        Ok(response.data.unwrap_or_default())
    }

    /// Get alert by ID.
    pub async fn get_alert(&self, alert_id: &str) -> Result<Alert, Error> {
        self.request(
            reqwest::Method::GET,
            &format!("/alerts/{}", alert_id),
        )
        .await
    }
}