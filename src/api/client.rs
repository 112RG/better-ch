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

    pub fn set_token(&mut self, token: Token) {
        self.token = Some(token);
    }

    pub fn token(&self) -> Option<&Token> {
        self.token.as_ref()
    }

    pub fn is_authenticated(&self) -> bool {
        self.token
            .as_ref()
            .map(|t| !t.is_expired())
            .unwrap_or(false)
    }

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

    // Applications API
    pub async fn list_applications(&self) -> Result<Vec<Application>, Error> {
        #[derive(Deserialize)]
        struct Response {
            data: Option<Vec<Application>>,
        }
        let response: Response = self.request(reqwest::Method::GET, "/applications").await?;
        Ok(response.data.unwrap_or_default())
    }

    pub async fn get_application(&self, name: &str) -> Result<Application, Error> {
        self.request(reqwest::Method::GET, &format!("/applications/{}", name))
            .await
    }

    pub async fn create_application(
        &self,
        request: CreateApplicationRequest,
    ) -> Result<Application, Error> {
        self.request_with_body(reqwest::Method::POST, "/applications", &request)
            .await
    }

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

    pub async fn delete_application(&self, name: &str) -> Result<(), Error> {
        let _response: StatusResponse = self
            .request(reqwest::Method::DELETE, &format!("/applications/{}", name))
            .await?;
        Ok(())
    }

    pub async fn start_application(&self, name: &str) -> Result<StatusResponse, Error> {
        self.request(
            reqwest::Method::POST,
            &format!("/applications/{}/start", name),
        )
        .await
    }

    pub async fn stop_application(&self, name: &str) -> Result<StatusResponse, Error> {
        self.request(
            reqwest::Method::POST,
            &format!("/applications/{}/stop", name),
        )
        .await
    }

    pub async fn restart_application(&self, name: &str) -> Result<StatusResponse, Error> {
        self.request(
            reqwest::Method::POST,
            &format!("/applications/{}/restart", name),
        )
        .await
    }

    // Statistics API
    pub async fn get_application_stats(&self, name: &str) -> Result<ApplicationStats, Error> {
        self.request(
            reqwest::Method::GET,
            &format!("/applications/{}/statistics", name),
        )
        .await
    }

    // Logs API
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
    pub async fn list_alerts(&self) -> Result<Vec<Alert>, Error> {
        #[derive(Deserialize)]
        struct Response {
            data: Option<Vec<Alert>>,
        }
        let response: Response = self.request(reqwest::Method::GET, "/alerts").await?;
        Ok(response.data.unwrap_or_default())
    }
}
