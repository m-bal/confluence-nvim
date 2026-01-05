# Adversarial Review Findings

**Review Date**: 2026-01-05
**Reviewer**: Claude (Adversarial Mode)
**Scope**: Confluence Neovim Plugin MVP (75% complete)
**Methodology**: VDD Adversarial Review - Zero tolerance for code slop

---

## Executive Summary

**Total Findings**: 35
**Critical**: 12
**High**: 15
**Medium**: 6
**Low**: 2

**Verdict**: **FAIL** - Multiple critical security vulnerabilities, panic points, and VDD violations (TODOs in production code) make this codebase unshippable.

---

## Critical Severity Findings

### C-01: Panic in HTTP Client Constructor
**File**: `src/api/mod.rs:113`
**Category**: Reliability / Panic
**Severity**: CRITICAL

**Issue**:
```rust
let client = Client::builder()
    .timeout(Duration::from_secs(30))
    .connect_timeout(Duration::from_secs(10))
    .user_agent("confluence-nvim/0.1.0")
    .pool_max_idle_per_host(10)
    .build()
    .expect("Failed to build HTTP client");  // PANIC!
```

The constructor uses `.expect()` which panics on failure. Client::builder().build() can fail due to TLS backend issues, system configuration, or missing crypto libraries.

**Reproduction**:
1. Run on a system without OpenSSL/rustls properly configured
2. Plugin crashes Neovim with panic

**Impact**: Complete Neovim crash, data loss for user

**Fix**: Return `Result<Self, ConfigError>` from constructor:
```rust
pub fn new(base_url: String, auth: AuthConfig) -> Result<Self, ConfigError> {
    let client = Client::builder()
        .timeout(Duration::from_secs(30))
        .connect_timeout(Duration::from_secs(10))
        .user_agent("confluence-nvim/0.1.0")
        .pool_max_idle_per_host(10)
        .build()
        .map_err(|e| ConfigError::ParseError(format!("Failed to build HTTP client: {}", e)))?;

    Ok(Self { base_url: base_url.trim_end_matches('/').to_string(), auth, client, retry_policy: RetryPolicy::default() })
}
```

---

### C-02: URL Injection Vulnerability in Page Fetch
**File**: `src/api/mod.rs:125-128`
**Category**: Security / Injection
**Severity**: CRITICAL

**Issue**:
```rust
let url = format!(
    "{}/rest/api/content/{}?expand=body.storage,space",
    self.base_url, page_id
);
```

`page_id` is NOT sanitized. Attacker can inject path traversal or query parameters.

**Reproduction**:
```rust
client.fetch_page("../admin/config?secret=true&").await;
// Results in: https://confluence.com/rest/api/content/../admin/config?secret=true&?expand=...
```

**Impact**: Access to unauthorized pages, potential information disclosure

**Fix**: Validate and sanitize page_id:
```rust
pub async fn fetch_page(&self, page_id: &str) -> Result<Page, ApiError> {
    // Validate page_id contains only safe characters
    if !page_id.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
        return Err(ApiError::InvalidInput(format!("Invalid page ID: {}", page_id)));
    }

    let url = format!(
        "{}/rest/api/content/{}?expand=body.storage,space",
        self.base_url,
        urlencoding::encode(page_id)
    );
    // ...
}
```

---

### C-03: CQL Injection Vulnerability
**File**: `src/api/mod.rs:188-193`
**Category**: Security / Injection
**Severity**: CRITICAL

**Issue**:
```rust
pub async fn search(&self, cql: &str) -> Result<Vec<SearchResult>, ApiError> {
    let url = format!(
        "{}/rest/api/content/search?cql={}",
        self.base_url,
        urlencoding::encode(cql)  // NOT ENOUGH!
    );
```

URL encoding does NOT prevent CQL injection. CQL has operators like `AND`, `OR`, `type=`, `space=`. Attacker can craft queries to access unauthorized content.

**Reproduction**:
```rust
// User input: "test) OR (space=ADMIN_SECRET"
client.search("test) OR (space=ADMIN_SECRET").await;
// CQL becomes: cql=test) OR (space=ADMIN_SECRET
// Returns results from restricted ADMIN_SECRET space
```

**Impact**: Authorization bypass, access to restricted spaces and pages

**Fix**: Implement CQL sanitization or use parameterized queries:
```rust
fn sanitize_cql_value(value: &str) -> String {
    // Escape CQL special characters: quotes, backslashes, parentheses
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('(', "\\(")
        .replace(')', "\\)")
}

pub async fn search(&self, query: &str) -> Result<Vec<SearchResult>, ApiError> {
    let sanitized = sanitize_cql_value(query);
    let cql = format!("text ~ \"{}\"", sanitized);
    let url = format!(
        "{}/rest/api/content/search?cql={}",
        self.base_url,
        urlencoding::encode(&cql)
    );
    // ...
}
```

---

### C-04: Unbounded JSON Parsing (Memory Exhaustion)
**File**: `src/api/mod.rs:136,154,177,201`
**Category**: Security / DoS
**Severity**: CRITICAL

**Issue**:
```rust
response.json::<Page>().await.map_err(|e| {
    ApiError::ParseError(format!("Failed to parse page response: {}", e))
})
```

No size limit on response body. Malicious/compromised Confluence server can send infinite JSON stream and exhaust all memory.

**Reproduction**:
1. Set up malicious server that streams infinite JSON
2. Call any API method
3. Neovim runs out of memory and crashes

**Impact**: Denial of service, system instability

**Fix**: Add body size limit:
```rust
const MAX_RESPONSE_SIZE: usize = 10 * 1024 * 1024; // 10MB

async fn parse_json<T: DeserializeOwned>(&self, response: Response) -> Result<T, ApiError> {
    let bytes = response
        .bytes()
        .await
        .map_err(|e| ApiError::RequestFailed(e))?;

    if bytes.len() > MAX_RESPONSE_SIZE {
        return Err(ApiError::ParseError(format!(
            "Response too large: {} bytes (max: {})",
            bytes.len(),
            MAX_RESPONSE_SIZE
        )));
    }

    serde_json::from_slice(&bytes)
        .map_err(|e| ApiError::ParseError(format!("Failed to parse JSON: {}", e)))
}
```

---

### C-05: Retry Logic Returns Errors as Success
**File**: `src/api/retry.rs:52-55`
**Category**: Logic Error
**Severity**: CRITICAL

**Issue**:
```rust
// Retry on 429 (rate limit) and 5xx errors
if status.as_u16() == 429 || status.is_server_error() {
    if attempt >= self.max_retries {
        return Ok(resp);  // BUG: Returns error response as Ok!
    }
```

After max retries, returns a 5xx error response wrapped in `Ok()`. Caller expects success but gets failure.

**Reproduction**:
1. Confluence server returns 500
2. After 3 retries, returns `Ok(Response { status: 500 })`
3. Caller tries to parse as JSON, gets confusing error

**Impact**: Incorrect error handling, confusing error messages, potential data corruption

**Fix**:
```rust
if attempt >= self.max_retries {
    return Err(ApiError::ApiError {
        status: status.as_u16(),
        message: format!("Max retries exceeded after {} attempts", self.max_retries),
    });
}
```

---

### C-06: Multiple Panic Points in Parser (Selector::parse)
**File**: `src/renderer/ast.rs:158,168-170,204,215`
**Category**: Reliability / Panic
**Severity**: CRITICAL

**Issue**:
```rust
let li_selector = Selector::parse("li").unwrap();  // Line 158
let th_selector = Selector::parse("th").unwrap();  // Line 168
let tr_selector = Selector::parse("tbody tr, tr").unwrap();  // Line 169
let td_selector = Selector::parse("td").unwrap();  // Line 170
let param_selector = Selector::parse("ac\\:parameter").unwrap();  // Line 204
let body_selector = Selector::parse("ac\\:plain-text-body, ac\\:rich-text-body").unwrap();  // Line 215
```

Six unwrap() calls that panic if CSS selector parsing fails. While these selectors are hardcoded and unlikely to fail, library updates or platform differences could cause panics.

**Reproduction**:
1. Update scraper crate to version with stricter CSS parsing
2. Parser panics during page render

**Impact**: Neovim crash while viewing Confluence page

**Fix**: Use lazy_static or once_cell to parse selectors once at startup:
```rust
use once_cell::sync::Lazy;

static LI_SELECTOR: Lazy<Selector> = Lazy::new(|| {
    Selector::parse("li").expect("Invalid hardcoded selector 'li'")
});

static TH_SELECTOR: Lazy<Selector> = Lazy::new(|| {
    Selector::parse("th").expect("Invalid hardcoded selector 'th'")
});

// Then use &*LI_SELECTOR instead of parsing inline
fn parse_list(element: ElementRef, ordered: bool) -> AstNode {
    let items: Vec<Vec<AstNode>> = element
        .select(&*LI_SELECTOR)
        .map(|li| Self::parse_element_children(li))
        .collect();
    AstNode::List { ordered, items }
}
```

---

### C-07: Stack Overflow via Unbounded Recursion
**File**: `src/renderer/ast.rs:55-74,150-155`
**Category**: Security / DoS
**Severity**: CRITICAL

**Issue**:
The parser recursively calls `parse_node` → `parse_element_ref` → `parse_element_children` → `parse_node` with NO depth limit.

**Reproduction**:
```html
<div><div><div><div>...(10,000 levels deep)...</div></div></div></div>
```

**Impact**: Stack overflow, Neovim crash

**Fix**: Add depth tracking:
```rust
const MAX_PARSE_DEPTH: usize = 100;

pub fn parse(html: &str) -> Result<Self, ParseError> {
    let document = Html::parse_fragment(html);
    let root = document.root_element();

    let mut nodes = Vec::new();
    for child in root.children() {
        if let Some(node) = Self::parse_node_with_depth(child, 0)? {
            nodes.push(node);
        }
    }

    Ok(Self { root: AstNode::Document(nodes) })
}

fn parse_node_with_depth(node: ego_tree::NodeRef<Node>, depth: usize) -> Result<Option<AstNode>, ParseError> {
    if depth > MAX_PARSE_DEPTH {
        return Err(ParseError::InvalidHtml(format!(
            "HTML nesting too deep (max: {})",
            MAX_PARSE_DEPTH
        )));
    }

    match node.value() {
        Node::Text(text) => {
            let content = text.trim();
            if content.is_empty() {
                Ok(None)
            } else {
                Ok(Some(AstNode::Text(content.to_string())))
            }
        }
        Node::Element(_) => {
            if let Some(element_ref) = ElementRef::wrap(node) {
                Self::parse_element_ref_with_depth(element_ref, depth + 1)
            } else {
                Ok(None)
            }
        }
        _ => Ok(None),
    }
}
```

---

### C-08: Cache Panic on Zero Capacity
**File**: `src/cache.rs:38-41`
**Category**: Reliability / Panic
**Severity**: CRITICAL

**Issue**:
```rust
pub fn new(capacity: usize) -> Self {
    Self {
        cache: LruCache::new(
            NonZeroUsize::new(capacity).expect("Cache capacity must be non-zero"),
        ),
```

Panics if capacity is 0. Config validation doesn't prevent this.

**Reproduction**:
```lua
require('confluence').setup({ cache_size = 0 })
```

**Impact**: Neovim crash on plugin initialization

**Fix**:
```rust
pub fn new(capacity: usize) -> Result<Self, CacheError> {
    let non_zero_capacity = NonZeroUsize::new(capacity)
        .ok_or_else(|| CacheError::InvalidCapacity("Capacity must be non-zero".to_string()))?;

    Ok(Self {
        cache: LruCache::new(non_zero_capacity),
        default_ttl: Duration::from_secs(900),
    })
}
```

---

### C-09: VDD Violation - TODO in Production Code (Telescope)
**File**: `lua/confluence/telescope/init.lua:14,64,135,179`
**Category**: VDD Violation
**Severity**: CRITICAL

**Issue**:
```lua
-- TODO: Fetch spaces from Rust API
local spaces = {
    { key = 'PROJ', name = 'Project Documentation', page_count = 156 },
    -- ... hardcoded test data
}
```

Four instances of `-- TODO` with hardcoded mock data. VDD explicitly forbids TODOs in production code. If this ships, users see fake data.

**Reproduction**:
1. Install plugin
2. Run `:Telescope confluence spaces`
3. See fake "PROJ" and "ENG" spaces that don't exist

**Impact**: Broken functionality, misleading UX, complete failure of primary features

**Fix**: Implement Rust-Lua bridge to fetch real data, or remove these functions entirely until implemented.

---

### C-10: Runtime Error - Undefined Function Call
**File**: `lua/confluence/telescope/init.lua:95,103,111,166,204`
**Category**: Logic Error
**Severity**: CRITICAL

**Issue**:
```lua
require('confluence').open_page(selection.value.id)
```

Calls `confluence.open_page()` which is NOT DEFINED anywhere in the codebase.

**Reproduction**:
1. Open telescope picker
2. Select any page
3. Error: `attempt to call field 'open_page' (a nil value)`

**Impact**: Complete failure of primary feature

**Fix**: Implement `open_page` function or use placeholder that shows clear error message.

---

### C-11: Weak URL Validation Allows Malicious URLs
**File**: `src/config.rs:100-105`
**Category**: Security
**Severity**: CRITICAL

**Issue**:
```rust
if !self.confluence_url.starts_with("http://") && !self.confluence_url.starts_with("https://") {
    return Err(ConfigError::InvalidValue {
        field: "confluence_url".to_string(),
        reason: "Must start with http:// or https://".to_string(),
    });
}
```

Accepts malformed URLs like:
- `http://evil.com@good.com` (credentials in URL)
- `http://localhost:99999` (invalid port)
- `http://` (no host)

**Impact**: Credentials leak, unexpected behavior, crashes

**Fix**: Use url crate for proper validation:
```rust
use url::Url;

pub fn validate(&self) -> Result<(), ConfigError> {
    let parsed_url = Url::parse(&self.confluence_url).map_err(|e| {
        ConfigError::InvalidValue {
            field: "confluence_url".to_string(),
            reason: format!("Invalid URL: {}", e),
        }
    })?;

    if parsed_url.scheme() != "https" && parsed_url.scheme() != "http" {
        return Err(ConfigError::InvalidValue {
            field: "confluence_url".to_string(),
            reason: "URL must use http or https scheme".to_string(),
        });
    }

    if parsed_url.host_str().is_none() {
        return Err(ConfigError::InvalidValue {
            field: "confluence_url".to_string(),
            reason: "URL must have a host".to_string(),
        });
    }

    // ... rest of validation
}
```

---

### C-12: Credentials Sent Over Unencrypted HTTP
**File**: `src/config.rs:100`
**Category**: Security
**Severity**: CRITICAL

**Issue**:
Config validation allows `http://` URLs, which send credentials in plaintext.

**Impact**: Credential theft via man-in-the-middle attack

**Fix**: Enforce HTTPS:
```rust
if parsed_url.scheme() != "https" {
    return Err(ConfigError::InvalidValue {
        field: "confluence_url".to_string(),
        reason: "Only HTTPS URLs are allowed for security".to_string(),
    });
}
```

---

## High Severity Findings

### H-01: Silent Data Loss - Whitespace Trimming
**File**: `src/renderer/ast.rs:58-63`
**Category**: Logic Error
**Severity**: HIGH

**Issue**:
```rust
Node::Text(text) => {
    let content = text.trim();
    if content.is_empty() {
        None
    } else {
        Some(AstNode::Text(content.to_string()))
    }
}
```

Removes ALL whitespace-only text nodes. In HTML, `<p>word   word</p>` has semantic meaning - the spaces matter for rendering.

**Impact**: Incorrect rendering, words run together, code blocks lose indentation

**Fix**: Preserve whitespace in certain contexts (code blocks, pre tags):
```rust
fn parse_node_in_context(node: ego_tree::NodeRef<Node>, preserve_ws: bool) -> Option<AstNode> {
    match node.value() {
        Node::Text(text) => {
            let content = if preserve_ws {
                text.to_string()
            } else {
                let trimmed = text.trim();
                if trimmed.is_empty() {
                    return None;
                }
                trimmed.to_string()
            };
            Some(AstNode::Text(content))
        }
        // ...
    }
}
```

---

### H-02: Unknown HTML Elements Wrapped as Paragraphs (Security Risk)
**File**: `src/renderer/ast.rs:137-146`
**Category**: Security / Logic Error
**Severity**: HIGH

**Issue**:
```rust
_ => {
    let children = Self::parse_element_children(element);
    if children.len() == 1 {
        Some(children.into_iter().next().unwrap())
    } else if !children.is_empty() {
        Some(AstNode::Paragraph(children))
    } else {
        None
    }
}
```

Unknown elements (including `<script>`, `<style>`, `<iframe>`) are parsed as paragraphs. While Confluence shouldn't return these, a compromised server could inject malicious HTML.

**Impact**: Potential XSS if malicious tags are rendered

**Fix**: Explicitly blacklist dangerous tags:
```rust
_ => {
    // Blacklist dangerous tags
    if ["script", "style", "iframe", "object", "embed"].contains(&tag) {
        tracing::warn!("Filtered out dangerous HTML tag: {}", tag);
        return None;
    }

    // Unknown safe elements - parse children
    let children = Self::parse_element_children(element);
    // ...
}
```

---

### H-03: Integer Overflow in Retry-After Header
**File**: `src/api/mod.rs:241-246`
**Category**: Logic Error
**Severity**: HIGH

**Issue**:
```rust
let retry_after = response
    .headers()
    .get("Retry-After")
    .and_then(|v| v.to_str().ok())
    .and_then(|v| v.parse::<u64>().ok())
    .unwrap_or(60);
```

Server can send `Retry-After: 999999999` causing excessive wait times (31 years).

**Impact**: Plugin hangs indefinitely, unresponsive UI

**Fix**: Add max limit:
```rust
const MAX_RETRY_AFTER: u64 = 300; // 5 minutes

let retry_after = response
    .headers()
    .get("Retry-After")
    .and_then(|v| v.to_str().ok())
    .and_then(|v| v.parse::<u64>().ok())
    .map(|v| std::cmp::min(v, MAX_RETRY_AFTER))
    .unwrap_or(60);
```

---

### H-04: Token Validation Insufficient (Whitespace Tokens)
**File**: `src/config.rs:109`
**Category**: Validation
**Severity**: HIGH

**Issue**:
```rust
if token.is_empty() {
    return Err(ConfigError::InvalidValue {
        field: "auth.token".to_string(),
        reason: "Token cannot be empty".to_string(),
    });
}
```

Accepts `token = "   "` (whitespace-only) which is useless.

**Fix**:
```rust
if token.trim().is_empty() {
    return Err(ConfigError::InvalidValue {
        field: "auth.token".to_string(),
        reason: "Token cannot be empty or whitespace-only".to_string(),
    });
}
```

---

### H-05: No Validation on Cache Configuration
**File**: `src/config.rs`
**Category**: Validation
**Severity**: HIGH

**Issue**:
No validation on `cache_ttl` or `cache_size`. Accepts nonsensical values:
- `cache_ttl = 0` (instant expiration, cache useless)
- `cache_ttl = u64::MAX` (292 billion years)
- `cache_size = 0` (causes panic, see C-08)
- `cache_size = usize::MAX` (infinite memory)

**Fix**:
```rust
const MIN_CACHE_TTL: u64 = 60; // 1 minute
const MAX_CACHE_TTL: u64 = 86400; // 24 hours
const MIN_CACHE_SIZE: usize = 1;
const MAX_CACHE_SIZE: usize = 1000;

pub fn validate(&self) -> Result<(), ConfigError> {
    // ... existing checks

    if self.cache_ttl < MIN_CACHE_TTL || self.cache_ttl > MAX_CACHE_TTL {
        return Err(ConfigError::InvalidValue {
            field: "cache_ttl".to_string(),
            reason: format!("Must be between {} and {} seconds", MIN_CACHE_TTL, MAX_CACHE_TTL),
        });
    }

    if self.cache_size < MIN_CACHE_SIZE || self.cache_size > MAX_CACHE_SIZE {
        return Err(ConfigError::InvalidValue {
            field: "cache_size".to_string(),
            reason: format!("Must be between {} and {}", MIN_CACHE_SIZE, MAX_CACHE_SIZE),
        });
    }

    Ok(())
}
```

---

### H-06: Expensive Clone on Every Cache Get
**File**: `src/cache.rs:55`
**Category**: Performance
**Severity**: HIGH

**Issue**:
```rust
Some(entry.value.clone())
```

Clones entire page HTML (potentially 100KB+) on every cache hit.

**Impact**: Excessive memory allocation, GC pressure, poor performance

**Fix**: Use Arc for shared ownership:
```rust
use std::sync::Arc;

pub struct ContentCache {
    cache: LruCache<String, CacheEntry<Arc<String>>>,
    default_ttl: Duration,
}

pub fn insert(&mut self, key: String, value: String) {
    let entry = CacheEntry::new(Arc::new(value), self.default_ttl);
    self.cache.put(key, entry);
}

pub fn get(&mut self, key: &str) -> Option<Arc<String>> {
    if let Some(entry) = self.cache.get(key) {
        if entry.is_expired() {
            self.cache.pop(key);
            None
        } else {
            Some(Arc::clone(&entry.value))
        }
    } else {
        None
    }
}
```

---

### H-07: No Telescope Availability Check
**File**: `lua/confluence/telescope/init.lua:2`
**Category**: Error Handling
**Severity**: HIGH

**Issue**:
```lua
local pickers = require('telescope.pickers')
```

Assumes telescope is installed. If not, errors immediately.

**Fix**:
```lua
local has_telescope, pickers = pcall(require, 'telescope.pickers')
if not has_telescope then
  error('confluence-nvim requires telescope.nvim to be installed')
  return {}
end
```

---

### H-08: No Error Handling in Telescope Pickers
**File**: `lua/confluence/telescope/init.lua` (all functions)
**Category**: Error Handling
**Severity**: HIGH

**Issue**:
All picker functions assume success. No error handling for:
- API failures
- Malformed data
- Missing fields

**Fix**: Wrap API calls in pcall and show errors:
```lua
function M.spaces(opts)
  local ok, spaces_or_err = pcall(function()
    return require('confluence.api').list_spaces()
  end)

  if not ok then
    vim.notify('Failed to fetch Confluence spaces: ' .. tostring(spaces_or_err), vim.log.levels.ERROR)
    return
  end

  -- ... rest of function
end
```

---

### H-09: Logging May Expose Sensitive Data
**File**: `src/api/retry.rs:69-75,97-103`
**Category**: Security
**Severity**: HIGH

**Issue**:
```rust
tracing::warn!(
    "Request failed with status {}, retrying in {:?} (attempt {}/{})",
    status,
    wait_duration,
    attempt + 1,
    self.max_retries
);
```

Logs request failures, but request URL may contain auth tokens or sensitive query parameters.

**Fix**: Redact URLs in logs:
```rust
fn redact_url(url: &str) -> String {
    // Replace query parameters with [REDACTED]
    if let Some(pos) = url.find('?') {
        format!("{}?[REDACTED]", &url[..pos])
    } else {
        url.to_string()
    }
}

tracing::warn!(
    "Request to {} failed with status {}, retrying in {:?}",
    redact_url(&url),
    status,
    wait_duration
);
```

---

### H-10: No Size Limit on HTML Parsing
**File**: `src/renderer/ast.rs:39-52`
**Category**: Security / DoS
**Severity**: HIGH

**Issue**:
Parser accepts HTML of any size. A 1GB HTML document will be parsed into memory.

**Fix**:
```rust
const MAX_HTML_SIZE: usize = 5 * 1024 * 1024; // 5MB

pub fn parse(html: &str) -> Result<Self, ParseError> {
    if html.len() > MAX_HTML_SIZE {
        return Err(ParseError::InvalidHtml(format!(
            "HTML too large: {} bytes (max: {})",
            html.len(),
            MAX_HTML_SIZE
        )));
    }

    // ... rest of parsing
}
```

---

### H-11: Empty href Attribute Creates Useless Links
**File**: `src/renderer/ast.rs:119`
**Category**: Logic Error
**Severity**: HIGH

**Issue**:
```rust
let url = element.value().attr("href").unwrap_or("").to_string();
```

Missing href becomes empty string. A link to "" is valid HTML but meaningless.

**Fix**:
```rust
"a" => {
    let url = element.value().attr("href")
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string());

    match url {
        Some(url) => Some(AstNode::Link {
            url,
            text: Self::parse_element_children(element),
        }),
        None => {
            // Link with no href - just return text content
            let children = Self::parse_element_children(element);
            if children.len() == 1 {
                Some(children.into_iter().next().unwrap())
            } else {
                Some(AstNode::Paragraph(children))
            }
        }
    }
}
```

---

### H-12: Unnecessary String Allocations in Hot Path
**File**: `src/cache.rs:50,54,58`
**Category**: Performance
**Severity**: HIGH

**Issue**:
```rust
tracing::debug!("Cache entry expired for key: {}", key);
tracing::debug!("Cache hit for key: {}", key);
tracing::debug!("Cache miss for key: {}", key);
```

Creates string allocations on EVERY cache access, even when debug logging is disabled.

**Fix**: Use structured logging:
```rust
tracing::debug!(key = %key, "Cache entry expired");
tracing::debug!(key = %key, "Cache hit");
tracing::debug!(key = %key, "Cache miss");
```

Or disable debug logs in hot paths.

---

### H-13: Race Condition in Cache Expiration Check
**File**: `src/cache.rs:48-56`
**Category**: Logic Error
**Severity**: MEDIUM (upgraded to HIGH due to data consistency impact)

**Issue**:
```rust
if let Some(entry) = self.cache.get(key) {
    if entry.is_expired() {
        tracing::debug!("Cache entry expired for key: {}", key);
        self.cache.pop(key);
        None
    }
```

Time passes between `is_expired()` check and usage. While not a real race in single-threaded code, the pattern is fragile and could break if cache becomes thread-safe later.

**Fix**: Check expiration inline:
```rust
pub fn get(&mut self, key: &str) -> Option<Arc<String>> {
    let should_remove = self.cache
        .peek(key)
        .map(|entry| entry.is_expired())
        .unwrap_or(false);

    if should_remove {
        self.cache.pop(key);
        return None;
    }

    self.cache.get(key).map(|entry| Arc::clone(&entry.value))
}
```

---

### H-14: No Request Cancellation Support
**File**: `src/api/mod.rs` (all async functions)
**Category**: Resource Management
**Severity**: HIGH

**Issue**:
All async operations use `.await` with no cancellation mechanism. If user closes buffer, request keeps running.

**Impact**: Wasted bandwidth, server resources, potential memory leaks

**Fix**: Use tokio::select with cancellation token:
```rust
use tokio::sync::oneshot;

pub struct ConfluenceClient {
    // ... existing fields
    cancel_tx: Option<oneshot::Sender<()>>,
}

pub async fn fetch_page(&self, page_id: &str) -> Result<Page, ApiError> {
    let (cancel_tx, mut cancel_rx) = oneshot::channel();

    let url = format!(/* ... */);

    tokio::select! {
        result = self.retry_policy.execute(|| self.get(&url)) => {
            // Normal flow
            let response = result?;
            // ...
        }
        _ = &mut cancel_rx => {
            Err(ApiError::Cancelled)
        }
    }
}
```

---

### H-15: No Rate Limiting on Outbound Requests
**File**: `src/api/mod.rs`
**Category**: Resource Management / Security
**Severity**: HIGH

**Issue**:
Client has retry logic but NO rate limiting. Tight loop can send thousands of requests.

**Impact**: IP ban, Confluence server overload, abuse

**Fix**: Add token bucket rate limiter:
```rust
use std::time::{Duration, Instant};

pub struct RateLimiter {
    tokens: f64,
    max_tokens: f64,
    refill_rate: f64, // tokens per second
    last_refill: Instant,
}

impl RateLimiter {
    pub fn new(requests_per_second: f64) -> Self {
        Self {
            tokens: requests_per_second,
            max_tokens: requests_per_second,
            refill_rate: requests_per_second,
            last_refill: Instant::now(),
        }
    }

    pub async fn acquire(&mut self) {
        // Refill tokens based on time elapsed
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill).as_secs_f64();
        self.tokens = (self.tokens + elapsed * self.refill_rate).min(self.max_tokens);
        self.last_refill = now;

        // If no tokens available, wait
        if self.tokens < 1.0 {
            let wait_time = ((1.0 - self.tokens) / self.refill_rate);
            tokio::time::sleep(Duration::from_secs_f64(wait_time)).await;
            self.tokens = 0.0;
        } else {
            self.tokens -= 1.0;
        }
    }
}

pub struct ConfluenceClient {
    // ... existing fields
    rate_limiter: RateLimiter,
}

pub async fn fetch_page(&self, page_id: &str) -> Result<Page, ApiError> {
    self.rate_limiter.acquire().await;
    // ... rest of function
}
```

---

## Medium Severity Findings

### M-01: Inconsistent Error Types
**File**: `src/api/mod.rs:28`
**Category**: Code Quality
**Severity**: MEDIUM

**Issue**:
```rust
#[error("API error: {status} - {message}")]
ApiError { status: u16, message: String },
```

Variant named `ApiError` within enum `ApiError` is confusing.

**Fix**: Rename to `ServerError`:
```rust
#[error("Server error: {status} - {message}")]
ServerError { status: u16, message: String },
```

---

### M-02: Magic Numbers in Retry Logic
**File**: `src/api/retry.rs:27-30`
**Category**: Code Quality
**Severity**: MEDIUM

**Issue**:
```rust
Self {
    max_retries: 3,
    initial_backoff: Duration::from_secs(1),
    max_backoff: Duration::from_secs(60),
    backoff_multiplier: 2,
}
```

Magic numbers without constants or documentation explaining why these values were chosen.

**Fix**:
```rust
// Retry 3 times with exponential backoff: 1s, 2s, 4s
const DEFAULT_MAX_RETRIES: u32 = 3;
const DEFAULT_INITIAL_BACKOFF_SECS: u64 = 1;
const DEFAULT_MAX_BACKOFF_SECS: u64 = 60;
const DEFAULT_BACKOFF_MULTIPLIER: u32 = 2;

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_retries: DEFAULT_MAX_RETRIES,
            initial_backoff: Duration::from_secs(DEFAULT_INITIAL_BACKOFF_SECS),
            max_backoff: Duration::from_secs(DEFAULT_MAX_BACKOFF_SECS),
            backoff_multiplier: DEFAULT_BACKOFF_MULTIPLIER,
        }
    }
}
```

---

### M-03: HTTP Client Pool Configuration Not Tunable
**File**: `src/api/mod.rs:111`
**Category**: Configurability
**Severity**: MEDIUM

**Issue**:
```rust
.pool_max_idle_per_host(10)
```

Hardcoded connection pool size. Users with high/low usage have no control.

**Fix**: Add to config:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    // ... existing fields

    #[serde(default = "default_connection_pool_size")]
    pub connection_pool_size: usize,
}

fn default_connection_pool_size() -> usize {
    10
}
```

---

### M-04: Missing Dependency on `url` Crate
**File**: `Cargo.toml`
**Category**: Dependencies
**Severity**: MEDIUM

**Issue**:
Config validation needs proper URL parsing (see C-11) but `url` crate is not in dependencies.

**Fix**: Add to Cargo.toml:
```toml
url = "2.5"
```

---

### M-05: Missing Dependency on `once_cell` Crate
**File**: `Cargo.toml`
**Category**: Dependencies
**Severity**: MEDIUM

**Issue**:
Parser fix for C-06 requires `once_cell` but it's not in dependencies.

**Fix**: Add to Cargo.toml:
```toml
once_cell = "1.19"
```

---

### M-06: Renderer Returns Placeholder Instead of Error
**File**: `src/renderer/mod.rs:92-96`
**Category**: Logic Error
**Severity**: MEDIUM

**Issue**:
```rust
// TODO: Implement actual rendering logic
// For now, return placeholder
lines.push(format!("# {}", title));
lines.push(String::new());
lines.push("Content rendering not yet implemented".to_string());
```

Renderer returns success with placeholder instead of NotImplemented error.

**Fix**:
```rust
Err(RenderError::UnsupportedElement("Full rendering not yet implemented".to_string()))
```

OR implement basic rendering (which is the proper fix).

---

## Low Severity Findings

### L-01: Unused Warning Suppressions Will Accumulate
**File**: `src/renderer/mod.rs:85,90`, `src/renderer/highlights.rs:27-48`
**Category**: Code Quality
**Severity**: LOW

**Issue**:
Multiple unused variable and dead code warnings. While expected in WIP code, these should be cleaned up before release.

**Fix**: Implement rendering logic to use these variables.

---

### L-02: Missing Documentation on Public APIs
**File**: Multiple files
**Category**: Documentation
**Severity**: LOW

**Issue**:
Many public functions lack doc comments explaining parameters, errors, and behavior.

**Fix**: Add comprehensive doc comments:
```rust
/// Fetch a Confluence page by ID.
///
/// # Arguments
///
/// * `page_id` - The numeric ID of the page to fetch
///
/// # Returns
///
/// * `Ok(Page)` - The page with body content and metadata
/// * `Err(ApiError::NotFound)` - Page doesn't exist or no access
/// * `Err(ApiError::AuthenticationFailed)` - Invalid credentials
/// * `Err(ApiError::RateLimited)` - Too many requests
///
/// # Example
///
/// ```no_run
/// let page = client.fetch_page("123456").await?;
/// println!("Title: {}", page.title);
/// ```
pub async fn fetch_page(&self, page_id: &str) -> Result<Page, ApiError> {
    // ...
}
```

---

## Summary of Required Changes

### Blocking (Must Fix Before Any Release)

1. **C-01**: Fix panic in HTTP client constructor
2. **C-02**: Fix URL injection in page fetch
3. **C-03**: Fix CQL injection in search
4. **C-04**: Add response size limits
5. **C-05**: Fix retry logic error handling
6. **C-06**: Fix selector parsing panics
7. **C-07**: Add recursion depth limit
8. **C-08**: Fix cache capacity panic
9. **C-09**: Remove TODOs, implement Rust-Lua bridge
10. **C-10**: Implement open_page function
11. **C-11**: Fix URL validation
12. **C-12**: Enforce HTTPS only

### High Priority (Fix Before MVP)

1. **H-01** through **H-15**: All high-severity issues

### Nice to Have (Fix Before 1.0)

1. **M-01** through **M-06**: Medium-severity issues
2. **L-01** through **L-02**: Low-severity issues

---

## Testing Requirements

For each finding, create a regression test:

1. **Unit tests**: For logic errors and validation
2. **Integration tests**: For API and cache behavior
3. **Fuzzing tests**: For parser with malformed input
4. **Security tests**: For injection vulnerabilities

**Test Coverage Target**: 90%+ after fixes

---

## Conclusion

The codebase shows solid architectural thinking but has **12 critical flaws** that make it unshippable:

- **Security vulnerabilities**: Injection attacks, credential leaks, DoS vectors
- **Reliability issues**: Multiple panic points, unbounded recursion
- **VDD violations**: TODOs in production code, broken functionality

**Recommendation**: Address all Critical findings before any testing or deployment. The codebase is salvageable but needs significant hardening.
