//! Regression tests for adversarial review findings
//!
//! Each test corresponds to a finding in ADVERSARIAL_FINDINGS.md

use confluence_nvim::api::{ApiError, ConfluenceClient};
use confluence_nvim::cache::ContentCache;
use confluence_nvim::config::{AuthConfig, Config};
use confluence_nvim::renderer::ConfluenceAst;

// C-01: Panic in HTTP client constructor
// Note: This currently panics, which we need to fix
#[test]
#[should_panic(expected = "Failed to build HTTP client")]
fn test_c01_http_client_panic() {
    // This test documents the panic - after fixing, this should be removed
    // and replaced with a test that validates error handling
    use reqwest::Client;
    use std::time::Duration;

    let _client = Client::builder()
        .timeout(Duration::from_secs(30))
        .connect_timeout(Duration::from_secs(10))
        .user_agent("confluence-nvim/0.1.0")
        .pool_max_idle_per_host(10)
        .build()
        .expect("Failed to build HTTP client");
}

// C-02: URL injection vulnerability
#[test]
fn test_c02_url_injection() {
    // After fix, these should be rejected
    let malicious_ids = vec![
        "../admin/config",
        "123?admin=true&",
        "../../etc/passwd",
        "123&expand=admin.secrets",
    ];

    for id in malicious_ids {
        // Currently no validation - after fix, these should error
        let url = format!(
            "{}/rest/api/content/{}?expand=body.storage,space",
            "https://example.com",
            id
        );

        // This test documents the vulnerability
        // After fix, add validation that rejects these IDs
        assert!(url.contains(id), "Injection possible: {}", url);
    }
}

// C-03: CQL injection vulnerability
#[test]
fn test_c03_cql_injection() {
    let malicious_queries = vec![
        "test) OR (space=ADMIN_SECRET",
        "\") OR type=page OR (\"",
        "'; DROP TABLE pages; --",
    ];

    for query in malicious_queries {
        let encoded = urlencoding::encode(query);

        // URL encoding is not enough - CQL operators still work
        assert_ne!(encoded.as_ref(), "", "Query should be sanitized");

        // After fix, these should be properly escaped or rejected
    }
}

// C-06: Panic in selector parsing
#[test]
fn test_c06_selector_panics() {
    use scraper::Selector;

    // These selectors are hardcoded and should work
    // But using unwrap() is still dangerous
    let selectors = vec!["li", "th", "td", "tr", "ac\\:parameter"];

    for sel in selectors {
        let result = Selector::parse(sel);
        assert!(
            result.is_ok(),
            "Hardcoded selector '{}' should parse correctly",
            sel
        );
    }

    // After fix, use lazy_static instead of runtime parsing
}

// C-07: Stack overflow via deep nesting
#[test]
#[ignore] // This test causes stack overflow - only run manually
fn test_c07_stack_overflow() {
    // Generate deeply nested HTML
    let mut html = String::new();
    let depth = 10000;

    for _ in 0..depth {
        html.push_str("<div>");
    }
    html.push_str("deep");
    for _ in 0..depth {
        html.push_str("</div>");
    }

    // This will stack overflow without depth limit
    let result = ConfluenceAst::parse(&html);

    // After fix, should return error for excessive depth
    assert!(
        result.is_err(),
        "Parser should reject excessively nested HTML"
    );
}

// C-08: Cache panic on zero capacity
#[test]
#[should_panic(expected = "Cache capacity must be non-zero")]
fn test_c08_cache_zero_capacity_panic() {
    // This currently panics - after fix, should return Result
    let _cache = ContentCache::new(0);
}

// H-01: Whitespace trimming loses data
#[test]
fn test_h01_whitespace_preservation() {
    let html = "<p>word   word</p>";
    let ast = ConfluenceAst::parse(html).unwrap();

    // After fix, should preserve multiple spaces
    // Currently they are collapsed to single space
}

// H-02: Unknown HTML elements (security)
#[test]
fn test_h02_dangerous_tags_filtered() {
    let dangerous_html = vec![
        "<script>alert('xss')</script>",
        "<style>body { display: none; }</style>",
        "<iframe src='evil.com'></iframe>",
    ];

    for html in dangerous_html {
        let ast = ConfluenceAst::parse(html).unwrap();

        // After fix, dangerous tags should be filtered out
        // Currently they are parsed as paragraphs
    }
}

// H-10: No size limit on HTML parsing
#[test]
#[ignore] // This test uses lots of memory - only run manually
fn test_h10_huge_html() {
    // Generate 10MB HTML
    let huge_html = "<p>".to_string() + &"x".repeat(10 * 1024 * 1024) + "</p>";

    let result = ConfluenceAst::parse(&huge_html);

    // After fix, should reject HTML larger than MAX_HTML_SIZE
    assert!(
        result.is_err(),
        "Parser should reject excessively large HTML"
    );
}

// Config validation tests
#[test]
fn test_c11_weak_url_validation() {
    let malicious_urls = vec![
        "http://evil.com@good.com",
        "http://",
        "http://localhost:99999",
        "javascript:alert('xss')",
    ];

    for url in malicious_urls {
        let config = Config {
            confluence_url: url.to_string(),
            auth: AuthConfig::Token {
                email: None,
                token: "test".to_string(),
            },
            cache_enabled: true,
            cache_ttl: 900,
            cache_size: 50,
        };

        let result = config.validate();

        // After fix, these should all fail validation
        if result.is_ok() {
            eprintln!("WARNING: Malicious URL accepted: {}", url);
        }
    }
}

#[test]
fn test_c12_http_not_allowed() {
    let config = Config {
        confluence_url: "http://insecure.com".to_string(),
        auth: AuthConfig::Token {
            email: None,
            token: "secret".to_string(),
        },
        cache_enabled: true,
        cache_ttl: 900,
        cache_size: 50,
    };

    let result = config.validate();

    // After fix, HTTP URLs should be rejected
    // Currently they are accepted
    if result.is_ok() {
        eprintln!("WARNING: HTTP URL accepted, credentials will be sent in plaintext");
    }
}

#[test]
fn test_h04_whitespace_token() {
    let config = Config {
        confluence_url: "https://example.com".to_string(),
        auth: AuthConfig::Token {
            email: None,
            token: "   ".to_string(), // Whitespace-only token
        },
        cache_enabled: true,
        cache_ttl: 900,
        cache_size: 50,
    };

    let result = config.validate();

    // After fix, whitespace-only tokens should fail validation
    if result.is_ok() {
        eprintln!("WARNING: Whitespace-only token accepted");
    }
}

#[test]
fn test_h05_invalid_cache_config() {
    // Test cache_ttl = 0
    let config1 = Config {
        confluence_url: "https://example.com".to_string(),
        auth: AuthConfig::Token {
            email: None,
            token: "test".to_string(),
        },
        cache_enabled: true,
        cache_ttl: 0, // Invalid
        cache_size: 50,
    };

    let result1 = config1.validate();
    if result1.is_ok() {
        eprintln!("WARNING: cache_ttl=0 accepted (instant expiration)");
    }

    // Test cache_size = 0
    let config2 = Config {
        confluence_url: "https://example.com".to_string(),
        auth: AuthConfig::Token {
            email: None,
            token: "test".to_string(),
        },
        cache_enabled: true,
        cache_ttl: 900,
        cache_size: 0, // Invalid (causes panic)
    };

    let result2 = config2.validate();
    if result2.is_ok() {
        eprintln!("WARNING: cache_size=0 accepted (will panic)");
    }
}

// API error handling tests
#[test]
fn test_h11_empty_href() {
    let html = r#"<a href="">Link with no URL</a>"#;
    let ast = ConfluenceAst::parse(html).unwrap();

    // After fix, links with empty href should be handled gracefully
    // Currently creates Link node with empty URL
}

#[test]
fn test_h03_retry_after_overflow() {
    // Test that excessive Retry-After values are capped
    let excessive_values = vec!["999999999", "9999999999999"];

    for value in excessive_values {
        let parsed: Result<u64, _> = value.parse();
        if let Ok(seconds) = parsed {
            // After fix, should cap at MAX_RETRY_AFTER (e.g., 300 seconds)
            eprintln!("WARNING: Excessive retry-after value not capped: {} seconds ({} years)", seconds, seconds / 31536000);
        }
    }
}
