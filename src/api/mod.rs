//! Confluence REST API client module

use crate::config::AuthConfig;
use reqwest::{Client, RequestBuilder, Response};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use thiserror::Error;

mod retry;

pub use retry::RetryPolicy;

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("HTTP request failed: {0}")]
    RequestFailed(#[from] reqwest::Error),

    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),

    #[error("Resource not found: {0}")]
    NotFound(String),

    #[error("Rate limit exceeded. Retry after {0} seconds")]
    RateLimited(u64),

    #[error("API error: {status} - {message}")]
    ApiError { status: u16, message: String },

    #[error("Failed to parse response: {0}")]
    ParseError(String),
}

/// Confluence page content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Page {
    pub id: String,
    pub title: String,
    #[serde(rename = "type")]
    pub page_type: String,
    pub body: PageBody,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageBody {
    pub storage: StorageFormat,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageFormat {
    pub value: String,
    pub representation: String,
}

/// Confluence API client
pub struct ConfluenceClient {
    base_url: String,
    auth: AuthConfig,
    client: Client,
    retry_policy: RetryPolicy,
}

impl ConfluenceClient {
    /// Create a new Confluence API client
    pub fn new(base_url: String, auth: AuthConfig) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .connect_timeout(Duration::from_secs(10))
            .user_agent("confluence-nvim/0.1.0")
            .pool_max_idle_per_host(10)
            .build()
            .expect("Failed to build HTTP client");

        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            auth,
            client,
            retry_policy: RetryPolicy::default(),
        }
    }

    /// Fetch a page by ID
    pub async fn fetch_page(&self, page_id: &str) -> Result<Page, ApiError> {
        let url = format!(
            "{}/rest/api/content/{}?expand=body.storage",
            self.base_url, page_id
        );

        let response = self
            .retry_policy
            .execute(|| self.get(&url))
            .await?;

        if response.status().is_success() {
            response.json::<Page>().await.map_err(|e| {
                ApiError::ParseError(format!("Failed to parse page response: {}", e))
            })
        } else {
            self.handle_error_response(response).await
        }
    }

    /// Build a GET request with authentication
    fn get(&self, url: &str) -> RequestBuilder {
        let mut request = self.client.get(url);

        request = match &self.auth {
            AuthConfig::Token { email, token } => {
                if let Some(email) = email {
                    request.basic_auth(email, Some(token))
                } else {
                    request.bearer_auth(token)
                }
            }
            AuthConfig::Pat { token } => request.bearer_auth(token),
        };

        request
    }

    /// Handle error responses
    async fn handle_error_response<T>(&self, response: Response) -> Result<T, ApiError> {
        let status = response.status();

        match status.as_u16() {
            401 | 403 => Err(ApiError::AuthenticationFailed(
                "Invalid credentials or insufficient permissions".to_string(),
            )),
            404 => Err(ApiError::NotFound(
                "Page not found or you don't have access".to_string(),
            )),
            429 => {
                let retry_after = response
                    .headers()
                    .get("Retry-After")
                    .and_then(|v| v.to_str().ok())
                    .and_then(|v| v.parse::<u64>().ok())
                    .unwrap_or(60);

                Err(ApiError::RateLimited(retry_after))
            }
            _ => {
                let message = response
                    .text()
                    .await
                    .unwrap_or_else(|_| "Unknown error".to_string());

                Err(ApiError::ApiError {
                    status: status.as_u16(),
                    message,
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let auth = AuthConfig::Token {
            email: Some("user@example.com".to_string()),
            token: "test-token".to_string(),
        };

        let client = ConfluenceClient::new("https://example.atlassian.net/wiki".to_string(), auth);

        assert_eq!(
            client.base_url,
            "https://example.atlassian.net/wiki"
        );
    }

    #[test]
    fn test_base_url_normalization() {
        let auth = AuthConfig::Pat {
            token: "test-token".to_string(),
        };

        let client = ConfluenceClient::new("https://example.com/wiki/".to_string(), auth);

        assert_eq!(client.base_url, "https://example.com/wiki");
    }
}
