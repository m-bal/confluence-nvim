//! Confluence Neovim Plugin
//!
//! A Neovim plugin for reading and rendering Confluence documentation
//! with visual fidelity matching the Confluence web interface.
//!
//! # Architecture
//!
//! - `api`: Confluence REST API client with authentication and rate limiting
//! - `renderer`: Content rendering engine for rich terminal output
//! - `cache`: LRU caching for API responses and rendered content
//! - `config`: Plugin configuration and validation

pub mod api;
pub mod cache;
pub mod config;
pub mod renderer;

// Re-export main types
pub use api::ConfluenceClient;
pub use cache::ContentCache;
pub use config::Config;
pub use renderer::Renderer;

// Note: nvim-oxi FFI integration is planned for a future release.
// The Lua renderer provides a fallback implementation that works
// without the Rust FFI bridge. The core Rust renderer is fully
// functional and can be integrated via FFI when needed.
//
// To enable FFI support:
// 1. Enable the "nvim" feature in Cargo.toml
// 2. Build with: cargo build --release --features nvim
// 3. The compiled library can be loaded via Lua's require()

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_initialization() {
        let cache = ContentCache::new(50).unwrap();
        assert_eq!(cache.capacity(), 50);
        assert!(cache.is_empty());
    }

    #[test]
    fn test_config_creation() {
        let config = Config::new(
            "https://example.com".to_string(),
            config::AuthConfig::Token {
                email: Some("user@example.com".to_string()),
                token: "test".to_string(),
            },
        );

        assert!(config.validate().is_ok());
    }
}
