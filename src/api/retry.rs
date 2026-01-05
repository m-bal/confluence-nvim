//! Retry policy for API requests

use super::ApiError;
use reqwest::{RequestBuilder, Response};
use std::time::Duration;
use tokio::time::sleep;

/// Retry policy configuration
#[derive(Debug, Clone)]
pub struct RetryPolicy {
    /// Maximum number of retries
    pub max_retries: u32,

    /// Initial backoff duration
    pub initial_backoff: Duration,

    /// Maximum backoff duration
    pub max_backoff: Duration,

    /// Backoff multiplier
    pub backoff_multiplier: u32,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_backoff: Duration::from_secs(1),
            max_backoff: Duration::from_secs(60),
            backoff_multiplier: 2,
        }
    }
}

impl RetryPolicy {
    /// Execute a request with retry logic
    pub async fn execute<F>(&self, mut request_fn: F) -> Result<Response, ApiError>
    where
        F: FnMut() -> RequestBuilder,
    {
        let mut attempt = 0;
        let mut backoff = self.initial_backoff;

        loop {
            let response = request_fn().send().await;

            match response {
                Ok(resp) => {
                    let status = resp.status();

                    // Retry on 429 (rate limit) and 5xx errors
                    if status.as_u16() == 429 || status.is_server_error() {
                        if attempt >= self.max_retries {
                            // C-05: Return error instead of Ok(error_response)
                            return Err(ApiError::ServerError {
                                status: status.as_u16(),
                                message: format!(
                                    "Max retries ({}) exceeded. Last status: {}",
                                    self.max_retries,
                                    status
                                ),
                            });
                        }

                        // Extract Retry-After header for rate limiting
                        let wait_duration = if status.as_u16() == 429 {
                            resp.headers()
                                .get("Retry-After")
                                .and_then(|v| v.to_str().ok())
                                .and_then(|v| v.parse::<u64>().ok())
                                .map(Duration::from_secs)
                                .unwrap_or(backoff)
                        } else {
                            backoff
                        };

                        tracing::warn!(
                            "Request failed with status {}, retrying in {:?} (attempt {}/{})",
                            status,
                            wait_duration,
                            attempt + 1,
                            self.max_retries
                        );

                        sleep(wait_duration).await;

                        attempt += 1;
                        backoff = std::cmp::min(
                            backoff * self.backoff_multiplier,
                            self.max_backoff,
                        );

                        continue;
                    }

                    // Success or non-retryable error
                    return Ok(resp);
                }
                Err(e) => {
                    // Retry on network errors
                    if attempt >= self.max_retries {
                        return Err(ApiError::RequestFailed(e));
                    }

                    tracing::warn!(
                        "Network error: {}, retrying in {:?} (attempt {}/{})",
                        e,
                        backoff,
                        attempt + 1,
                        self.max_retries
                    );

                    sleep(backoff).await;

                    attempt += 1;
                    backoff = std::cmp::min(
                        backoff * self.backoff_multiplier,
                        self.max_backoff,
                    );
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_retry_policy_defaults() {
        let policy = RetryPolicy::default();

        assert_eq!(policy.max_retries, 3);
        assert_eq!(policy.initial_backoff, Duration::from_secs(1));
        assert_eq!(policy.max_backoff, Duration::from_secs(60));
        assert_eq!(policy.backoff_multiplier, 2);
    }

    #[test]
    fn test_backoff_calculation() {
        let policy = RetryPolicy::default();

        let mut backoff = policy.initial_backoff;

        // First retry: 1s
        assert_eq!(backoff, Duration::from_secs(1));

        // Second retry: 2s
        backoff = backoff * policy.backoff_multiplier;
        assert_eq!(backoff, Duration::from_secs(2));

        // Third retry: 4s
        backoff = backoff * policy.backoff_multiplier;
        assert_eq!(backoff, Duration::from_secs(4));

        // Should not exceed max_backoff
        for _ in 0..10 {
            backoff = std::cmp::min(
                backoff * policy.backoff_multiplier,
                policy.max_backoff,
            );
        }
        assert_eq!(backoff, policy.max_backoff);
    }
}
