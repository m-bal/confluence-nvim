//! Plugin configuration module

use serde::{Deserialize, Serialize};
use thiserror::Error;
use url::Url;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Missing required field: {0}")]
    MissingField(String),

    #[error("Invalid field value: {field} - {reason}")]
    InvalidValue { field: String, reason: String },

    #[error("Failed to parse configuration: {0}")]
    ParseError(String),
}

/// Authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum AuthConfig {
    /// API token authentication (Confluence Cloud)
    Token {
        email: Option<String>,
        token: String,
    },
    /// Personal Access Token (Confluence Data Center)
    Pat {
        token: String,
    },
}

/// Plugin configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Base URL of Confluence instance
    pub confluence_url: String,

    /// Authentication configuration
    pub auth: AuthConfig,

    /// Enable response caching
    #[serde(default = "default_cache_enabled")]
    pub cache_enabled: bool,

    /// Cache TTL in seconds
    #[serde(default = "default_cache_ttl")]
    pub cache_ttl: u64,

    /// Maximum cache size (number of pages)
    #[serde(default = "default_cache_size")]
    pub cache_size: usize,
}

fn default_cache_enabled() -> bool {
    true
}

fn default_cache_ttl() -> u64 {
    900 // 15 minutes
}

fn default_cache_size() -> usize {
    50
}

impl Default for Config {
    fn default() -> Self {
        Self {
            confluence_url: String::new(),
            auth: AuthConfig::Token {
                email: None,
                token: String::new(),
            },
            cache_enabled: default_cache_enabled(),
            cache_ttl: default_cache_ttl(),
            cache_size: default_cache_size(),
        }
    }
}

impl Config {
    /// Create a new configuration
    pub fn new(confluence_url: String, auth: AuthConfig) -> Self {
        Self {
            confluence_url,
            auth,
            cache_enabled: default_cache_enabled(),
            cache_ttl: default_cache_ttl(),
            cache_size: default_cache_size(),
        }
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<(), ConfigError> {
        // Validate URL is not empty
        if self.confluence_url.is_empty() {
            return Err(ConfigError::MissingField("confluence_url".to_string()));
        }

        // Validate URL is properly formed using url crate (C-11)
        let parsed_url = Url::parse(&self.confluence_url).map_err(|e| {
            ConfigError::InvalidValue {
                field: "confluence_url".to_string(),
                reason: format!("Invalid URL: {}", e),
            }
        })?;

        // Enforce HTTPS only for security (C-12)
        if parsed_url.scheme() != "https" {
            return Err(ConfigError::InvalidValue {
                field: "confluence_url".to_string(),
                reason: "Only HTTPS URLs are allowed for security. HTTP sends credentials in plaintext.".to_string(),
            });
        }

        // Validate URL has a host
        if parsed_url.host_str().is_none() {
            return Err(ConfigError::InvalidValue {
                field: "confluence_url".to_string(),
                reason: "URL must have a host".to_string(),
            });
        }

        // Validate authentication token (H-04)
        match &self.auth {
            AuthConfig::Token { token, .. } | AuthConfig::Pat { token } => {
                if token.trim().is_empty() {
                    return Err(ConfigError::InvalidValue {
                        field: "auth.token".to_string(),
                        reason: "Token cannot be empty or whitespace-only".to_string(),
                    });
                }
            }
        }

        // Validate cache configuration (H-05)
        const MIN_CACHE_TTL: u64 = 60; // 1 minute
        const MAX_CACHE_TTL: u64 = 86400; // 24 hours
        const MIN_CACHE_SIZE: usize = 1;
        const MAX_CACHE_SIZE: usize = 1000;

        if self.cache_ttl < MIN_CACHE_TTL || self.cache_ttl > MAX_CACHE_TTL {
            return Err(ConfigError::InvalidValue {
                field: "cache_ttl".to_string(),
                reason: format!(
                    "Must be between {} and {} seconds ({} min to {} hours)",
                    MIN_CACHE_TTL, MAX_CACHE_TTL, MIN_CACHE_TTL / 60, MAX_CACHE_TTL / 3600
                ),
            });
        }

        if self.cache_size < MIN_CACHE_SIZE || self.cache_size > MAX_CACHE_SIZE {
            return Err(ConfigError::InvalidValue {
                field: "cache_size".to_string(),
                reason: format!(
                    "Must be between {} and {} pages",
                    MIN_CACHE_SIZE, MAX_CACHE_SIZE
                ),
            });
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.cache_enabled, true);
        assert_eq!(config.cache_ttl, 900);
        assert_eq!(config.cache_size, 50);
    }

    #[test]
    fn test_validate_empty_url() {
        let config = Config {
            confluence_url: String::new(),
            ..Default::default()
        };

        assert!(matches!(
            config.validate(),
            Err(ConfigError::MissingField(_))
        ));
    }

    #[test]
    fn test_validate_empty_token() {
        let config = Config {
            confluence_url: "https://example.com".to_string(),
            auth: AuthConfig::Token {
                email: None,
                token: String::new(),
            },
            ..Default::default()
        };

        assert!(matches!(
            config.validate(),
            Err(ConfigError::InvalidValue { .. })
        ));
    }

    #[test]
    fn test_validate_http_url_rejected() {
        let config = Config {
            confluence_url: "http://example.com".to_string(),
            auth: AuthConfig::Token {
                email: None,
                token: "test".to_string(),
            },
            ..Default::default()
        };

        assert!(matches!(
            config.validate(),
            Err(ConfigError::InvalidValue { .. })
        ));
    }

    #[test]
    fn test_validate_whitespace_token_rejected() {
        let config = Config {
            confluence_url: "https://example.com".to_string(),
            auth: AuthConfig::Token {
                email: None,
                token: "   ".to_string(),
            },
            ..Default::default()
        };

        assert!(matches!(
            config.validate(),
            Err(ConfigError::InvalidValue { .. })
        ));
    }

    #[test]
    fn test_validate_invalid_cache_ttl() {
        let config = Config {
            confluence_url: "https://example.com".to_string(),
            auth: AuthConfig::Token {
                email: None,
                token: "test".to_string(),
            },
            cache_enabled: true,
            cache_ttl: 0, // Too low
            cache_size: 50,
        };

        assert!(matches!(
            config.validate(),
            Err(ConfigError::InvalidValue { .. })
        ));
    }

    #[test]
    fn test_validate_invalid_cache_size() {
        let config = Config {
            confluence_url: "https://example.com".to_string(),
            auth: AuthConfig::Token {
                email: None,
                token: "test".to_string(),
            },
            cache_enabled: true,
            cache_ttl: 900,
            cache_size: 0, // Too low
        };

        assert!(matches!(
            config.validate(),
            Err(ConfigError::InvalidValue { .. })
        ));
    }
}
