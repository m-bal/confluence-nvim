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

    #[error("Server error: {status} - {message}")]
    ServerError { status: u16, message: String },

    #[error("Failed to parse response: {0}")]
    ParseError(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Response too large: {0}")]
    ResponseTooLarge(String),
}

/// Confluence page content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Page {
    pub id: String,
    pub title: String,
    #[serde(rename = "type")]
    pub page_type: String,
    pub body: PageBody,
    #[serde(default)]
    pub space: Option<SpaceInfo>,
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

/// Confluence space
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Space {
    pub id: i64,
    pub key: String,
    pub name: String,
    #[serde(rename = "type")]
    pub space_type: String,
}

/// Space information embedded in pages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpaceInfo {
    pub key: String,
    pub name: String,
}

/// Search result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub id: String,
    pub title: String,
    #[serde(rename = "type")]
    pub result_type: String,
    #[serde(default)]
    pub excerpt: Option<String>,
    #[serde(default)]
    pub space: Option<SpaceInfo>,
}

/// Paginated response wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    pub results: Vec<T>,
    pub start: usize,
    pub limit: usize,
    pub size: usize,
}

/// Default pagination limit for API requests
const PAGINATION_LIMIT: usize = 100;

/// Confluence API client
pub struct ConfluenceClient {
    base_url: String,
    auth: AuthConfig,
    client: Client,
    retry_policy: RetryPolicy,
}

impl ConfluenceClient {
    /// Create a new Confluence API client (C-01: Fixed panic)
    pub fn new(base_url: String, auth: AuthConfig) -> Result<Self, ApiError> {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .connect_timeout(Duration::from_secs(10))
            .user_agent("confluence-nvim/0.1.0")
            .pool_max_idle_per_host(10)
            .build()
            .map_err(|e| {
                ApiError::ServerError {
                    status: 0,
                    message: format!("Failed to build HTTP client: {}", e),
                }
            })?;

        Ok(Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            auth,
            client,
            retry_policy: RetryPolicy::default(),
        })
    }

    /// Validate page ID contains only safe characters (C-02)
    fn validate_page_id(page_id: &str) -> Result<(), ApiError> {
        if page_id.is_empty() {
            return Err(ApiError::InvalidInput("Page ID cannot be empty".to_string()));
        }

        // Allow only alphanumeric, hyphens, and underscores
        if !page_id.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
            return Err(ApiError::InvalidInput(format!(
                "Invalid page ID '{}': contains unsafe characters",
                page_id
            )));
        }

        Ok(())
    }

    /// Sanitize CQL search query (C-03)
    fn sanitize_cql_value(value: &str) -> String {
        // Escape CQL special characters
        value
            .replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('(', "\\(")
            .replace(')', "\\)")
    }

    /// Fetch a page by ID (C-02: Added validation, C-04: Added size limit)
    pub async fn fetch_page(&self, page_id: &str) -> Result<Page, ApiError> {
        // Validate page ID to prevent injection
        Self::validate_page_id(page_id)?;

        let url = format!(
            "{}/rest/api/content/{}?expand=body.storage,space",
            self.base_url, page_id
        );

        let response = self
            .retry_policy
            .execute(|| self.get(&url))
            .await?;

        if response.status().is_success() {
            self.parse_json_with_size_limit(response).await
        } else {
            self.handle_error_response(response).await
        }
    }

    /// Parse JSON response with size limit (C-04)
    async fn parse_json_with_size_limit<T: serde::de::DeserializeOwned>(
        &self,
        response: Response,
    ) -> Result<T, ApiError> {
        const MAX_RESPONSE_SIZE: usize = 10 * 1024 * 1024; // 10MB

        let bytes = response.bytes().await.map_err(ApiError::RequestFailed)?;

        if bytes.len() > MAX_RESPONSE_SIZE {
            return Err(ApiError::ResponseTooLarge(format!(
                "Response size {} bytes exceeds maximum of {} bytes ({}MB)",
                bytes.len(),
                MAX_RESPONSE_SIZE,
                MAX_RESPONSE_SIZE / (1024 * 1024)
            )));
        }

        serde_json::from_slice(&bytes).map_err(|e| {
            ApiError::ParseError(format!("Failed to parse JSON response: {}", e))
        })
    }

    /// Fetch all pages of results from a paginated endpoint
    async fn fetch_all_pages<T: serde::de::DeserializeOwned + Clone>(
        &self,
        base_url: &str,
    ) -> Result<Vec<T>, ApiError> {
        let mut all_results: Vec<T> = Vec::new();
        let mut start: usize = 0;

        loop {
            // Build URL with pagination parameters
            let separator = if base_url.contains('?') { '&' } else { '?' };
            let url = format!("{}{}limit={}&start={}", base_url, separator, PAGINATION_LIMIT, start);

            let response = self
                .retry_policy
                .execute(|| self.get(&url))
                .await?;

            if !response.status().is_success() {
                return self.handle_error_response(response).await;
            }

            let paginated: PaginatedResponse<T> = self.parse_json_with_size_limit(response).await?;

            // Append results
            all_results.extend(paginated.results);

            // Check if there are more pages
            if paginated.size < paginated.limit {
                // No more pages
                break;
            }

            // Move to next page
            start += paginated.limit;
        }

        Ok(all_results)
    }

    /// List all spaces with full pagination (C-04: Added size limit)
    pub async fn list_spaces(&self) -> Result<Vec<Space>, ApiError> {
        let url = format!("{}/rest/api/space", self.base_url);
        self.fetch_all_pages(&url).await
    }

    /// List pages in a space with full pagination (C-04: Added size limit)
    pub async fn list_pages(&self, space_key: &str) -> Result<Vec<Page>, ApiError> {
        let url = format!(
            "{}/rest/api/space/{}/content/page?expand=space",
            self.base_url, space_key
        );
        self.fetch_all_pages(&url).await
    }

    /// Search Confluence content with full pagination (C-03: Added CQL sanitization, C-04: Added size limit)
    pub async fn search(&self, query: &str) -> Result<Vec<SearchResult>, ApiError> {
        // Sanitize user query and wrap in text search
        let sanitized = Self::sanitize_cql_value(query);
        let cql = format!("text ~ \"{}\"", sanitized);

        let url = format!(
            "{}/rest/api/content/search?cql={}",
            self.base_url,
            urlencoding::encode(&cql)
        );
        self.fetch_all_pages(&url).await
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

    /// Handle error responses (H-03: Added retry-after cap)
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
                const MAX_RETRY_AFTER: u64 = 300; // 5 minutes

                let retry_after = response
                    .headers()
                    .get("Retry-After")
                    .and_then(|v| v.to_str().ok())
                    .and_then(|v| v.parse::<u64>().ok())
                    .map(|v| std::cmp::min(v, MAX_RETRY_AFTER))
                    .unwrap_or(60);

                Err(ApiError::RateLimited(retry_after))
            }
            _ => {
                let message = response
                    .text()
                    .await
                    .unwrap_or_else(|_| "Unknown error".to_string());

                Err(ApiError::ServerError {
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

        let client = ConfluenceClient::new("https://example.atlassian.net/wiki".to_string(), auth).unwrap();

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

        let client = ConfluenceClient::new("https://example.com/wiki/".to_string(), auth).unwrap();

        assert_eq!(client.base_url, "https://example.com/wiki");
    }

    #[test]
    fn test_validate_page_id_valid() {
        assert!(ConfluenceClient::validate_page_id("123456").is_ok());
        assert!(ConfluenceClient::validate_page_id("page-123").is_ok());
        assert!(ConfluenceClient::validate_page_id("page_123").is_ok());
    }

    #[test]
    fn test_validate_page_id_invalid() {
        assert!(ConfluenceClient::validate_page_id("").is_err());
        assert!(ConfluenceClient::validate_page_id("../admin").is_err());
        assert!(ConfluenceClient::validate_page_id("123?admin=true").is_err());
        assert!(ConfluenceClient::validate_page_id("123&test=1").is_err());
    }

    #[test]
    fn test_sanitize_cql() {
        let result = ConfluenceClient::sanitize_cql_value("test) OR (space=ADMIN");
        assert!(!result.contains(") OR ("));
        assert!(result.contains("\\("));
        assert!(result.contains("\\)"));
    }
}
