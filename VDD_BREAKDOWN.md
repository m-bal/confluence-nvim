# VDD Breakdown: Confluence Neovim Plugin

**Epic**: Create a Neovim plugin in Rust that renders Confluence documentation in its original format within Neovim.

**Success Criteria**:
- Users can view Confluence pages in Neovim with proper rendering
- Authentication is secure and configurable
- Plugin integrates seamlessly with Neovim workflow
- Code passes adversarial review with comprehensive test coverage
- Zero unhandled edge cases in critical paths

---

## Epic 1: Project Foundation & Infrastructure
**Objective**: Establish Rust-based Neovim plugin architecture with proper tooling and CI/CD.

### Issue 1.1: Rust Project Scaffolding
**Acceptance Criteria**: Cargo project with proper dependencies and build configuration.

- **Sub-issue 1.1.1**: Initialize Cargo workspace with library crate
  - AC: `Cargo.toml` with workspace configuration
  - AC: Proper edition, version, and metadata

- **Sub-issue 1.1.2**: Add Neovim RPC dependencies
  - AC: `nvim-oxi` or `nvim-rs` crate integrated
  - AC: Async runtime (tokio) configured

- **Sub-issue 1.1.3**: Configure development tooling
  - AC: `rustfmt.toml` with project style rules
  - AC: `clippy.toml` with strict lints enabled
  - AC: `.editorconfig` for consistency

### Issue 1.2: Neovim Plugin Integration
**Acceptance Criteria**: Plugin can be loaded by Neovim and exposes basic RPC interface.

- **Sub-issue 1.2.1**: Create Lua plugin entry point
  - AC: `lua/confluence/init.lua` with setup function
  - AC: Plugin metadata in `plugin/confluence.lua`

- **Sub-issue 1.2.2**: Establish Rust ↔ Neovim RPC bridge
  - AC: MessagePack RPC communication working
  - AC: Basic command registration (`:ConfluenceTest`)
  - AC: Error handling from Rust → Neovim

- **Sub-issue 1.2.3**: Plugin installation documentation
  - AC: Instructions for lazy.nvim, packer, vim-plug
  - AC: Build requirements documented

### Issue 1.3: Testing Infrastructure
**Acceptance Criteria**: Comprehensive test framework for unit, integration, and E2E tests.

- **Sub-issue 1.3.1**: Unit test framework
  - AC: `#[cfg(test)]` modules in all source files
  - AC: Mock traits for external dependencies

- **Sub-issue 1.3.2**: Integration test setup
  - AC: `tests/` directory with integration harness
  - AC: Mock Confluence API server for testing

- **Sub-issue 1.3.3**: Neovim headless testing
  - AC: E2E tests using `nvim --headless`
  - AC: Automated test runner in CI

---

## Epic 2: Confluence API Client
**Objective**: Robust, secure client for Confluence REST API with comprehensive error handling.

### Issue 2.1: HTTP Client Foundation
**Acceptance Criteria**: Type-safe HTTP client with proper timeout and retry logic.

- **Sub-issue 2.1.1**: Configure `reqwest` with TLS
  - AC: `reqwest::Client` with connection pooling
  - AC: Default timeouts (connect: 10s, request: 30s)
  - AC: User-Agent header identifying the plugin

- **Sub-issue 2.1.2**: Implement exponential backoff retry
  - AC: Retry on 429 (rate limit) and 5xx errors
  - AC: Configurable max retries (default: 3)
  - AC: Exponential backoff: 1s, 2s, 4s

- **Sub-issue 2.1.3**: Request/response logging
  - AC: Debug-level logging of all requests
  - AC: Sanitize auth tokens in logs
  - AC: Log response status and latency

### Issue 2.2: Authentication System
**Acceptance Criteria**: Secure multi-method authentication with credential management.

- **Sub-issue 2.2.1**: API token authentication
  - AC: Support for Confluence Cloud API tokens
  - AC: Basic Auth with email + API token
  - AC: Token validation on startup

- **Sub-issue 2.2.2**: Personal Access Token (PAT) support
  - AC: Bearer token authentication for Data Center
  - AC: Token refresh logic (if applicable)

- **Sub-issue 2.2.3**: Credential storage strategy
  - AC: Never store plaintext credentials in config
  - AC: Support for environment variables
  - AC: Integration with system keychain (optional)
  - AC: Clear error messages for missing credentials

### Issue 2.3: Confluence REST API Implementation
**Acceptance Criteria**: Type-safe API client with comprehensive coverage of needed endpoints.

- **Sub-issue 2.3.1**: Content retrieval endpoints
  - AC: GET `/rest/api/content/{id}` with expand params
  - AC: GET `/rest/api/content` with search/filtering
  - AC: Proper deserialization of JSON responses

- **Sub-issue 2.3.2**: Content search functionality
  - AC: CQL (Confluence Query Language) support
  - AC: Search by title, space, label
  - AC: Pagination handling for large result sets

- **Sub-issue 2.3.3**: Attachment and macro handling
  - AC: Fetch embedded images/attachments
  - AC: Detect and handle common macros (code, status, info)
  - AC: Graceful degradation for unsupported macros

- **Sub-issue 2.3.4**: Rate limiting and quota management
  - AC: Detect 429 responses and back off
  - AC: Track API quota usage (if provided in headers)
  - AC: User notification when approaching limits

---

## Epic 3: Content Rendering Engine
**Objective**: Convert Confluence storage format to Neovim-compatible rendered output.

### Issue 3.1: Storage Format Parsing
**Acceptance Criteria**: Parse Confluence XHTML storage format into structured AST.

- **Sub-issue 3.1.1**: XHTML parser integration
  - AC: Use `quick-xml` or `scraper` for parsing
  - AC: Build AST from storage format
  - AC: Handle malformed/invalid markup gracefully

- **Sub-issue 3.1.2**: Confluence-specific element handling
  - AC: Parse structured macros (code, info, warning, etc.)
  - AC: Extract table structures
  - AC: Handle nested lists and formatting

- **Sub-issue 3.1.3**: Link resolution
  - AC: Convert internal page links to navigable format
  - AC: Handle attachment links
  - AC: Preserve external URLs

### Issue 3.2: Markdown Conversion
**Acceptance Criteria**: Convert parsed Confluence content to GitHub Flavored Markdown.

- **Sub-issue 3.2.1**: Basic element conversion
  - AC: Headings (h1-h6) → Markdown headers
  - AC: Paragraphs, bold, italic, strikethrough
  - AC: Lists (ordered, unordered, nested)

- **Sub-issue 3.2.2**: Advanced formatting
  - AC: Tables → GFM tables with alignment
  - AC: Code blocks with language detection
  - AC: Blockquotes

- **Sub-issue 3.2.3**: Macro translation
  - AC: Info/Warning macros → Markdown admonitions
  - AC: Code macros → fenced code blocks
  - AC: Status macros → emoji or text indicators
  - AC: Unknown macros → clearly marked placeholders

### Issue 3.3: Neovim Buffer Rendering
**Acceptance Criteria**: Display rendered content in Neovim buffer with syntax highlighting.

- **Sub-issue 3.3.1**: Buffer creation and management
  - AC: Create scratch buffer for Confluence content
  - AC: Set proper `buftype` and `bufhidden` options
  - AC: Unique buffer naming scheme

- **Sub-issue 3.3.2**: Syntax highlighting
  - AC: Apply Markdown syntax highlighting
  - AC: Custom highlight groups for Confluence elements
  - AC: Syntax highlighting for code blocks

- **Sub-issue 3.3.3**: Buffer metadata and navigation
  - AC: Store Confluence page ID in buffer variable
  - AC: Implement `:ConfluenceRefresh` to reload
  - AC: Display page title and breadcrumbs

---

## Epic 4: User Interface & Commands
**Objective**: Intuitive command interface and navigation system for Confluence content.

### Issue 4.1: Core Commands
**Acceptance Criteria**: Essential commands for viewing and navigating Confluence.

- **Sub-issue 4.1.1**: `:ConfluenceOpen` command
  - AC: Accept page ID or URL
  - AC: Fetch and render in new buffer
  - AC: Handle invalid IDs with clear errors

- **Sub-issue 4.1.2**: `:ConfluenceSearch` command
  - AC: Interactive search prompt
  - AC: Display results in quickfix or floating window
  - AC: Select result to open page

- **Sub-issue 4.1.3**: `:ConfluenceSpace` command
  - AC: List pages in a given space
  - AC: Tree view of space hierarchy
  - AC: Navigate to pages from list

### Issue 4.2: Navigation Features
**Acceptance Criteria**: Seamless navigation between Confluence pages within Neovim.

- **Sub-issue 4.2.1**: Link following
  - AC: `gx` or custom mapping to follow links
  - AC: Detect Confluence URLs in buffer
  - AC: Open linked page in new buffer

- **Sub-issue 4.2.2**: Breadcrumb navigation
  - AC: Display page ancestors in statusline
  - AC: Navigate to parent pages

- **Sub-issue 4.2.3**: History and jump list
  - AC: Maintain navigation history
  - AC: `:ConfluenceBack` and `:ConfluenceForward`
  - AC: Integration with Neovim jumplist

### Issue 4.3: Configuration System
**Acceptance Criteria**: User-friendly configuration with sensible defaults.

- **Sub-issue 4.3.1**: Lua configuration API
  - AC: `require('confluence').setup({ ... })` function
  - AC: Validate configuration on setup
  - AC: Merge user config with defaults

- **Sub-issue 4.3.2**: Configuration options
  - AC: `confluence_url` (base URL)
  - AC: `auth` (method and credentials reference)
  - AC: `cache_enabled` and `cache_ttl`
  - AC: `render_options` (custom rendering preferences)

- **Sub-issue 4.3.3**: Configuration validation
  - AC: Check required fields on setup
  - AC: Validate URL format
  - AC: Test authentication on first use
  - AC: Clear error messages for misconfig

---

## Epic 5: Performance & Caching
**Objective**: Optimize performance with intelligent caching and lazy loading.

### Issue 5.1: Response Caching
**Acceptance Criteria**: Minimize API calls with smart caching strategy.

- **Sub-issue 5.1.1**: In-memory cache implementation
  - AC: LRU cache for API responses
  - AC: Configurable cache size (default: 50 pages)
  - AC: TTL-based expiration (default: 15 minutes)

- **Sub-issue 5.1.2**: Cache invalidation
  - AC: Manual cache clear command
  - AC: Respect Cache-Control headers
  - AC: Invalidate on known updates (if push available)

- **Sub-issue 5.1.3**: Persistent cache (optional)
  - AC: Disk-based cache in XDG_CACHE_HOME
  - AC: Serialize cached responses
  - AC: Cache versioning for compatibility

### Issue 5.2: Async Operations
**Acceptance Criteria**: Non-blocking UI during network operations.

- **Sub-issue 5.2.1**: Async content fetching
  - AC: All API calls use async/await
  - AC: Neovim UI remains responsive during fetch
  - AC: Loading indicator in statusline

- **Sub-issue 5.2.2**: Progressive rendering
  - AC: Show partial content as it arrives
  - AC: Render headers/outline first
  - AC: Stream large content into buffer

- **Sub-issue 5.2.3**: Background prefetching
  - AC: Prefetch linked pages on idle
  - AC: Respect rate limits during prefetch
  - AC: User-configurable prefetch behavior

---

## Epic 6: Error Handling & Resilience
**Objective**: Comprehensive error handling with graceful degradation.

### Issue 6.1: Network Error Handling
**Acceptance Criteria**: Robust handling of all network failure modes.

- **Sub-issue 6.1.1**: Connection failures
  - AC: Detect timeout, DNS, and connection refused
  - AC: User-friendly error messages
  - AC: Suggest troubleshooting steps

- **Sub-issue 6.1.2**: HTTP error responses
  - AC: Handle 401/403 (auth errors) with re-auth prompt
  - AC: Handle 404 (not found) with helpful message
  - AC: Handle 429 (rate limit) with wait time
  - AC: Handle 5xx with retry suggestion

- **Sub-issue 6.1.3**: Partial failure handling
  - AC: Render page even if attachments fail
  - AC: Show placeholder for failed macros
  - AC: Log errors without crashing plugin

### Issue 6.2: Data Validation
**Acceptance Criteria**: Validate all external data to prevent crashes.

- **Sub-issue 6.2.1**: API response validation
  - AC: Validate JSON schema before deserialization
  - AC: Handle missing expected fields
  - AC: Type-safe deserialization with serde

- **Sub-issue 6.2.2**: Content sanitization
  - AC: Escape or strip dangerous markup
  - AC: Validate URLs before rendering
  - AC: Limit content size to prevent DoS

- **Sub-issue 6.2.3**: Input validation
  - AC: Validate user-provided page IDs
  - AC: Sanitize search queries
  - AC: Reject invalid configuration values

---

## Epic 7: Documentation & Examples
**Objective**: Comprehensive documentation for users and contributors.

### Issue 7.1: User Documentation
**Acceptance Criteria**: Clear, example-driven documentation for end users.

- **Sub-issue 7.1.1**: README.md
  - AC: Feature overview with screenshots
  - AC: Installation instructions (all plugin managers)
  - AC: Quick start guide
  - AC: Configuration examples

- **Sub-issue 7.1.2**: Neovim help docs
  - AC: `doc/confluence.txt` with full command reference
  - AC: Configuration options documented
  - AC: Troubleshooting section

- **Sub-issue 7.1.3**: Example configurations
  - AC: Minimal config example
  - AC: Advanced config with all options
  - AC: Common use cases documented

### Issue 7.2: Developer Documentation
**Acceptance Criteria**: Enable contributors to understand and extend the codebase.

- **Sub-issue 7.2.1**: Architecture documentation
  - AC: `ARCHITECTURE.md` with module overview
  - AC: Data flow diagrams
  - AC: Plugin lifecycle documentation

- **Sub-issue 7.2.2**: API documentation
  - AC: Rustdoc comments on all public items
  - AC: `cargo doc` generates complete docs
  - AC: Code examples in rustdoc

- **Sub-issue 7.2.3**: Contributing guide
  - AC: `CONTRIBUTING.md` with setup instructions
  - AC: Code style guidelines
  - AC: VDD workflow for contributors

---

## Verification Checklist (Pre-Adversarial Review)

### Functional Verification
- [ ] Plugin loads in Neovim without errors
- [ ] Can authenticate with Confluence Cloud
- [ ] Can fetch and render a simple page
- [ ] Can fetch and render a complex page (tables, macros, code blocks)
- [ ] Can search for pages
- [ ] Can navigate between linked pages
- [ ] Error messages are clear and actionable

### Test Coverage
- [ ] Unit tests for all API client functions (>80% coverage)
- [ ] Unit tests for all rendering functions (>80% coverage)
- [ ] Integration tests for RPC communication
- [ ] E2E tests with mock Confluence API
- [ ] E2E tests in headless Neovim

### Security Verification
- [ ] Credentials never logged or stored in plaintext
- [ ] All network calls use HTTPS
- [ ] Input validation prevents injection attacks
- [ ] Dependencies audited with `cargo audit`

### Performance Verification
- [ ] Page load time < 2s for cached content
- [ ] Page load time < 5s for uncached content (normal network)
- [ ] No UI blocking during network operations
- [ ] Memory usage reasonable (<100MB for 50 cached pages)

### Code Quality
- [ ] No `unwrap()` or `expect()` in production paths
- [ ] All errors propagated with proper context
- [ ] No clippy warnings with strict lints
- [ ] All public APIs documented with rustdoc
- [ ] No TODOs or FIXMEs in committed code

---

## Adversarial Review Focus Areas

When ready for adversarial review, the following areas should be stressed:

1. **Authentication Security**: Token leakage, credential exposure, session management
2. **API Error Handling**: All HTTP status codes, network failures, malformed responses
3. **Rendering Edge Cases**: Malicious markup, deeply nested structures, massive pages
4. **Concurrency Issues**: Race conditions, deadlocks, async state management
5. **Resource Exhaustion**: Memory leaks, connection leaks, cache overflow
6. **Configuration Errors**: Missing required fields, invalid values, conflicting options
7. **Plugin Lifecycle**: Load/unload behavior, cleanup, state persistence

---

## Dependencies Audit

Critical dependencies to be locked and audited:

- `nvim-oxi` or `nvim-rs`: Neovim RPC bridge
- `reqwest`: HTTP client
- `tokio`: Async runtime
- `serde` / `serde_json`: Serialization
- `quick-xml` or `scraper`: XHTML parsing
- `pulldown-cmark` or similar: Markdown generation

All dependencies must:
- Have active maintenance
- Pass `cargo audit` security checks
- Have acceptable license (MIT/Apache-2.0)
- Be pinned to specific versions in production
