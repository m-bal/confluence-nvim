//! Plugin configuration module

use serde::{Deserialize, Serialize};
use thiserror::Error;

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
        if self.confluence_url.is_empty() {
            return Err(ConfigError::MissingField("confluence_url".to_string()));
        }

        if !self.confluence_url.starts_with("http://") && !self.confluence_url.starts_with("https://") {
            return Err(ConfigError::InvalidValue {
                field: "confluence_url".to_string(),
                reason: "Must start with http:// or https://".to_string(),
            });
        }

        match &self.auth {
            AuthConfig::Token { token, .. } | AuthConfig::Pat { token } => {
                if token.is_empty() {
                    return Err(ConfigError::InvalidValue {
                        field: "auth.token".to_string(),
                        reason: "Token cannot be empty".to_string(),
                    });
                }
            }
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
}
