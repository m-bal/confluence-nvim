# Confluence Neovim Testing Strategy

Based on research of best practices from telescope.nvim, gitsigns.nvim, and mini.nvim.

---

## Testing Framework Decision

**Primary: Plenary.nvim + Custom Helpers**

**Rationale:**
- Most widely adopted in Neovim plugin ecosystem
- Works well with Lua + Rust (via RPC)
- Async support for API calls
- Familiar busted-style syntax
- Easy CI/CD integration

**Secondary: Rust unit tests**
- Use `cargo test` for pure Rust logic
- API client, parser, renderer functions
- Mock HTTP responses with `mockito`

---

## Test Categories

### 1. Unit Tests (Rust) - 60%

**What to test:**
- API client functions
- Retry logic and exponential backoff
- XHTML/storage format parsing
- Rendering functions (text → buffer lines)
- Cache operations (get, set, eviction, TTL)
- Configuration validation

**Location:** `src/*/tests` modules

**Example:**
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use mockito::{mock, server_url};

    #[tokio::test]
    async fn test_fetch_page_with_retry_on_429() {
        // First call: 429 with Retry-After
        let _m1 = mock("GET", "/rest/api/content/123")
            .with_status(429)
            .with_header("Retry-After", "1")
            .expect(1)
            .create();

        // Second call: success
        let _m2 = mock("GET", "/rest/api/content/123")
            .with_status(200)
            .with_body(r#"{"id":"123","title":"Test"}"#)
            .expect(1)
            .create();

        let client = ConfluenceClient::new(server_url(), auth);
        let page = client.fetch_page("123").await.unwrap();

        assert_eq!(page.id, "123");
        assert_eq!(page.title, "Test");
    }
}
```

---

### 2. Integration Tests (Lua + Rust) - 30%

**What to test:**
- End-to-end: API fetch → parse → render → buffer
- Neovim buffer creation and configuration
- Extmark and virtual text placement
- Highlight group application
- Command registration and execution

**Location:** `tests/integration/`

**Example:**
```lua
-- tests/integration/page_rendering_spec.lua
local helpers = require('tests.helpers')
local eq = assert.are.equal

describe('page rendering', function()
  before_each(function()
    helpers.setup_test_env()
  end)

  after_each(function()
    helpers.cleanup()
  end)

  it('renders page with correct buffer options', function()
    local page_id = '123456'
    vim.cmd('ConfluenceOpen ' .. page_id)

    local buf = vim.api.nvim_get_current_buf()

    eq('nofile', vim.api.nvim_buf_get_option(buf, 'buftype'))
    eq('confluence', vim.api.nvim_buf_get_option(buf, 'filetype'))
    eq(false, vim.api.nvim_buf_get_option(buf, 'modifiable'))
    eq(page_id, vim.b.confluence_page_id)
  end)

  it('renders headings with proper highlights', function()
    helpers.open_test_page('heading_test')

    local lines = vim.api.nvim_buf_get_lines(0, 0, -1, false)
    eq('# Test Heading', lines[1])

    -- Verify highlight was applied
    local ns = vim.api.nvim_create_namespace('confluence')
    local marks = vim.api.nvim_buf_get_extmarks(0, ns, 0, -1, {details = true})

    local heading_mark = helpers.find_mark_by_line(marks, 0)
    eq('ConfluenceHeading1', heading_mark.hl_group)
  end)

  it('renders info panel with box drawing', function()
    helpers.open_test_page('macro_test')

    local lines = vim.api.nvim_buf_get_lines(0, 0, -1, false)

    -- Verify box drawing characters
    assert.is_true(lines[1]:match('^┌─ ℹ INFO'))
    assert.is_true(lines[2]:match('^│'))
    assert.is_true(lines[3]:match('^└'))
  end)

  it('applies syntax highlighting to code blocks', function()
    helpers.open_test_page('code_block_test')

    local lines = vim.api.nvim_buf_get_lines(0, 0, -1, false)

    -- Find code block
    local code_start = helpers.find_line_matching(lines, '┌─ RUST')

    -- Verify syntax highlighting was injected
    -- (Check for treesitter or vim syntax)
    local has_highlight = vim.treesitter.get_parser(0, 'rust') ~= nil
    assert.is_true(has_highlight)
  end)
end)
```

---

### 3. Visual Regression Tests - 10%

**What to test:**
- Full page layouts don't regress
- Box-drawing alignment
- Table formatting
- Multi-line content wrapping

**Location:** `tests/visual/`

**Example:**
```lua
-- tests/visual/layout_spec.lua
local Screen = require('tests.screen')

describe('visual layout', function()
  local screen

  before_each(function()
    screen = Screen.new(80, 40)
    screen:attach()
  end)

  after_each(function()
    screen:detach()
  end)

  it('renders info panel correctly', function()
    helpers.open_test_page('info_panel')

    screen:expect([[
      ┌─ ℹ INFO ────────────────────────────────────────┐
      │ This is important information                   │
      │ spanning multiple lines                         │
      └─────────────────────────────────────────────────┘
    ]])
  end)

  it('renders table with proper alignment', function()
    helpers.open_test_page('table_test')

    screen:expect([[
      ┌─────────────┬──────────────┐
      │ Name        │ Status       │
      ├─────────────┼──────────────┤
      │ Feature A   │ Done         │
      │ Feature B   │ In Progress  │
      └─────────────┴──────────────┘
    ]])
  end)
end)
```

---

## Test Helpers

### `tests/helpers.lua`

```lua
local M = {}

-- Mock Confluence API responses
M.mock_api = {
  page_response = function(page_id, title, content)
    return vim.fn.json_encode({
      id = page_id,
      title = title,
      body = {
        storage = {
          value = content,
          representation = 'storage'
        }
      }
    })
  end,

  search_response = function(results)
    return vim.fn.json_encode({
      results = results,
      size = #results
    })
  end
}

-- Setup test environment
M.setup_test_env = function()
  require('confluence').setup({
    confluence_url = 'http://localhost:8080',
    auth = { token = 'test-token' },
    cache_enabled = false,  -- Disable cache for tests
  })
end

-- Cleanup after tests
M.cleanup = function()
  -- Close all Confluence buffers
  for _, buf in ipairs(vim.api.nvim_list_bufs()) do
    if vim.api.nvim_buf_is_valid(buf) then
      local ft = vim.api.nvim_buf_get_option(buf, 'filetype')
      if ft == 'confluence' or ft == 'confluence-browser' then
        vim.api.nvim_buf_delete(buf, { force = true })
      end
    end
  end

  -- Clear cache
  require('confluence').clear_cache()
end

-- Open a test page (from fixtures)
M.open_test_page = function(fixture_name)
  local fixture_path = 'tests/fixtures/' .. fixture_name .. '.html'
  local content = M.read_fixture(fixture_path)

  -- Mock the API response
  local mock_response = M.mock_api.page_response('test-id', 'Test Page', content)

  -- Trigger page open (this will use mocked API)
  vim.cmd('ConfluenceOpen test-id')

  -- Wait for async rendering to complete
  vim.wait(1000, function()
    return vim.bo.filetype == 'confluence'
  end)
end

-- Read fixture file
M.read_fixture = function(path)
  local file = io.open(path, 'r')
  if not file then
    error('Fixture not found: ' .. path)
  end
  local content = file:read('*all')
  file:close()
  return content
end

-- Find extmark by line number
M.find_mark_by_line = function(marks, line)
  for _, mark in ipairs(marks) do
    if mark[2] == line then
      return mark[4]  -- Return extmark details
    end
  end
  return nil
end

-- Find line matching pattern
M.find_line_matching = function(lines, pattern)
  for i, line in ipairs(lines) do
    if line:match(pattern) then
      return i - 1  -- Return 0-indexed line number
    end
  end
  return nil
end

return M
```

---

## Test Fixtures

### `tests/fixtures/`

Store sample Confluence HTML for different test cases:

**info_panel.html:**
```html
<ac:structured-macro ac:name="info">
  <ac:rich-text-body>
    <p>This is important information</p>
    <p>spanning multiple lines</p>
  </ac:rich-text-body>
</ac:structured-macro>
```

**code_block.html:**
```html
<ac:structured-macro ac:name="code">
  <ac:parameter ac:name="language">rust</ac:parameter>
  <ac:plain-text-body><![CDATA[
fn main() {
    println!("Hello, world!");
}
  ]]></ac:plain-text-body>
</ac:structured-macro>
```

**table.html:**
```html
<table>
  <thead>
    <tr><th>Name</th><th>Status</th></tr>
  </thead>
  <tbody>
    <tr><td>Feature A</td><td>Done</td></tr>
    <tr><td>Feature B</td><td>In Progress</td></tr>
  </tbody>
</table>
```

---

## Mock Confluence API Server

For integration tests, use a mock HTTP server.

### `tests/mock_server.lua`

```lua
local M = {}

local responses = {}

-- Register a mock response
M.register = function(method, path, response)
  local key = method .. ' ' .. path
  responses[key] = response
end

-- Start mock server (uses external http server process)
M.start = function()
  -- Option 1: Use Lua http library
  -- Option 2: Use simple Python/Node server
  -- Option 3: Use wiremock (Rust)

  os.execute('tests/mock_server.py &')
  vim.wait(500)  -- Wait for server to start
end

-- Stop mock server
M.stop = function()
  os.execute('pkill -f mock_server.py')
end

-- Reset all registered responses
M.reset = function()
  responses = {}
end

return M
```

### `tests/mock_server.py`

```python
#!/usr/bin/env python3
from http.server import BaseHTTPRequestHandler, HTTPServer
import json

class MockConfluenceHandler(BaseHTTPRequestHandler):
    def do_GET(self):
        if '/rest/api/content/' in self.path:
            page_id = self.path.split('/')[-1].split('?')[0]

            self.send_response(200)
            self.send_header('Content-type', 'application/json')
            self.end_headers()

            response = {
                'id': page_id,
                'title': f'Test Page {page_id}',
                'body': {
                    'storage': {
                        'value': '<p>Test content</p>',
                        'representation': 'storage'
                    }
                }
            }

            self.wfile.write(json.dumps(response).encode())
        else:
            self.send_error(404)

if __name__ == '__main__':
    server = HTTPServer(('localhost', 8080), MockConfluenceHandler)
    print('Mock Confluence server running on port 8080')
    server.serve_forever()
```

---

## Running Tests

### All Tests

```bash
make test
```

### Unit Tests Only (Rust)

```bash
cargo test --lib
```

### Integration Tests Only (Lua)

```bash
nvim --headless -c "PlenaryBustedDirectory tests/integration/ {minimal_init = 'tests/minimal_init.lua'}"
```

### Visual Tests Only

```bash
nvim --headless -c "PlenaryBustedDirectory tests/visual/ {minimal_init = 'tests/minimal_init.lua'}"
```

### With Coverage

```bash
cargo tarpaulin --out Html --output-dir coverage
```

### Watch Mode (for development)

```bash
cargo watch -x test
```

---

## CI/CD Integration

### GitHub Actions

```yaml
name: Tests

on: [push, pull_request]

jobs:
  rust-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - name: Run Rust tests
        run: cargo test --all

  lua-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Install Neovim
        run: |
          sudo add-apt-repository ppa:neovim-ppa/unstable
          sudo apt-get update
          sudo apt-get install neovim
      - name: Install plenary.nvim
        run: |
          git clone https://github.com/nvim-lua/plenary.nvim ~/.local/share/nvim/site/pack/vendor/start/plenary.nvim
      - name: Run Lua tests
        run: |
          nvim --headless -c "PlenaryBustedDirectory tests/ {minimal_init = 'tests/minimal_init.lua'}"

  coverage:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - name: Install tarpaulin
        run: cargo install cargo-tarpaulin
      - name: Generate coverage
        run: cargo tarpaulin --out Xml
      - name: Upload to codecov
        uses: codecov/codecov-action@v3
```

---

## Makefile

```makefile
.PHONY: test test-rust test-lua test-visual test-all coverage watch

# Run all tests
test: test-rust test-lua

# Rust unit tests
test-rust:
	cargo test --lib

# Lua integration tests
test-lua:
	nvim --headless -c "PlenaryBustedDirectory tests/integration/ {minimal_init = 'tests/minimal_init.lua'}"

# Visual regression tests
test-visual:
	nvim --headless -c "PlenaryBustedDirectory tests/visual/ {minimal_init = 'tests/minimal_init.lua'}"

# All tests including visual
test-all: test test-visual

# Coverage report
coverage:
	cargo tarpaulin --out Html --output-dir coverage
	open coverage/index.html

# Watch mode for development
watch:
	cargo watch -x test

# Lint
lint:
	cargo clippy -- -D warnings
	luacheck lua/ tests/

# Format
fmt:
	cargo fmt
	stylua lua/ tests/
```

---

## Test Coverage Goals

| Component | Target Coverage | Priority |
|-----------|----------------|----------|
| API Client | 85%+ | High |
| Parser | 90%+ | High |
| Renderer | 85%+ | High |
| Cache | 90%+ | Medium |
| Config | 85%+ | Medium |
| UI/Commands | 70%+ | Low |

---

## VDD Testing Workflow

### Before Adversarial Review

1. ✅ All unit tests passing (23+)
2. ✅ Integration tests for core workflows
3. ✅ Visual tests for key layouts
4. ✅ Coverage > 80% on critical paths
5. ✅ No known crashes or panics
6. ✅ Manual testing on real Confluence instance

### During Adversarial Review

1. Add regression tests for each discovered issue
2. Test edge cases suggested by adversary
3. Stress test with large/malformed pages
4. Security audit with malicious input

### After Adversarial Refinement

1. All regression tests passing
2. No new warnings or errors
3. Coverage maintained or improved
4. Documentation updated

---

## Key Testing Principles

1. **Test What Matters**: Focus on rendering correctness, not implementation details
2. **Use Real Data**: Test with actual Confluence HTML, not simplified mocks
3. **Fast Feedback**: Unit tests run in <1s, integration tests in <5s
4. **Isolated Tests**: Each test is independent, no shared state
5. **Clear Failures**: Test failures pinpoint exact problem
6. **Visual Regression**: Catch layout breaks early
7. **Edge Cases**: Test empty content, huge pages, malformed HTML
8. **Security**: Test XSS, injection, token leakage

---

## Example Test Suite Structure

```
tests/
├── integration/
│   ├── page_rendering_spec.lua
│   ├── navigation_spec.lua
│   ├── search_spec.lua
│   └── cache_spec.lua
├── visual/
│   ├── layout_spec.lua
│   ├── tables_spec.lua
│   └── code_blocks_spec.lua
├── fixtures/
│   ├── simple_page.html
│   ├── complex_page.html
│   ├── info_panel.html
│   ├── code_block.html
│   ├── table.html
│   └── nested_lists.html
├── helpers.lua
├── screen.lua
├── mock_server.lua
├── mock_server.py
└── minimal_init.lua
```

---

## Next Steps

1. **Set up plenary.nvim in dev dependencies**
2. **Create `tests/minimal_init.lua`**
3. **Implement `tests/helpers.lua`**
4. **Write first integration test** (page rendering)
5. **Set up mock API server**
6. **Add Makefile** for test runners
7. **Configure GitHub Actions**

This testing strategy ensures:
- ✅ Rendering correctness
- ✅ Visual regressions caught early
- ✅ Fast feedback loop
- ✅ High confidence before adversarial review
- ✅ Easy to add regression tests after adversary findings
