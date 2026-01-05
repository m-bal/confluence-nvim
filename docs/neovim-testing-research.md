# Comprehensive Guide to Testing Neovim Buffer Rendering and Visual Output

## 1. Headless Neovim Testing

### Overview
Neovim can run in headless mode to test buffer content, extmarks, virtual text, and highlights. The most robust pattern uses **child Neovim processes** to isolate tests and prevent side effects.

### Child Process Pattern (Recommended)

The `mini.test` framework provides the most mature child process testing:

```lua
local MiniTest = require('mini.test')
local expect = MiniTest.expect
local eq = MiniTest.expect.equality

local child = MiniTest.new_child_neovim()

-- Start child process with minimal config
child.restart({'-u', 'scripts/minimal_init.lua'})

-- Execute Lua code in child process
child.lua([[M = require('your_plugin')]])

-- Get results from child
local result = child.lua_get([[M.some_function()]])

-- Stop child when done
child.stop()
```

**Key Benefits:**
- RPC communication between current and child Neovim processes
- Full access to buffer API, extmarks, floating windows
- Complete isolation prevents test interference
- Can test visual rendering via screenshots

### Running Tests Headless

**From command line:**
```bash
# Using plenary.nvim
nvim --headless -c "PlenaryBustedDirectory tests/"

# Using mini.test
nvim --headless --noplugin -u ./scripts/minimal_init.lua -c "lua MiniTest.run()"

# Using busted with nlua
busted --lua nlua spec/
```

**From within Neovim:**
```vim
:PlenaryBustedFile %
:lua MiniTest.run_file()
```

---

## 2. Snapshot Testing (Golden File Testing)

### Mini.test Reference Screenshots

The most powerful snapshot testing for Neovim plugins uses `mini.test`'s reference screenshot feature:

```lua
local child = MiniTest.new_child_neovim()

T['renders page correctly'] = function()
  -- Setup
  child.restart({ '-u', 'scripts/minimal_init.lua' })
  child.o.lines, child.o.columns = 20, 80

  -- Render your content
  child.lua([[
    vim.api.nvim_buf_set_lines(0, 0, -1, true, {
      '# Title',
      '',
      'Content here'
    })
  ]])

  -- Compare with reference screenshot
  expect.reference_screenshot(child.get_screenshot())
end
```

**How it works:**
1. **First run:** Automatically creates reference screenshot in `tests/screenshots/`
2. **Subsequent runs:** Compares current output with reference
3. **On mismatch:** Throws detailed error showing differences

**Screenshot contains two layers:**
- `text`: 2D array of characters displayed at each cell
- `attr`: 2D array of visual attributes (colors, styles)

**Advanced options:**
```lua
expect.reference_screenshot(
  child.get_screenshot({ redraw = true }),
  {
    ignore_text = false,  -- Set true to only check attributes
    ignore_attr = false   -- Set true to only check text
  }
)
```

### Plenary.nvim Screen Testing (Used by Gitsigns)

Gitsigns uses plenary's Screen class for visual assertions:

```lua
local Screen = require('test.functional.ui.screen')
local screen = Screen.new(50, 20)  -- width, height

-- Define color attributes
local default_attrs = {
  [1] = { foreground = Screen.colors.DarkBlue, background = Screen.colors.WebGray },
  [2] = { foreground = Screen.colors.DodgerBlue },
  [3] = { background = Screen.colors.LightBlue },
}
screen:set_default_attr_ids(default_attrs)

-- Assert screen content with highlights
screen:expect({
  grid = [[
  ^{MATCH:Blame text {6: Author, %d second.}}|
  Normal text here              |
  {3:Highlighted line           }|
  ]],
})
```

**Features:**
- `{N:text}` applies attribute ID N to text
- `{MATCH:pattern}` allows regex matching for dynamic content
- Automatically waits for screen to stabilize

---

## 3. Real-World Plugin Examples

### Gitsigns.nvim Testing Strategy

**Test Structure:** (`lewis6991/gitsigns.nvim/test/`)
- `gitsigns_spec.lua` - Core functionality
- `highlights_spec.lua` - Highlight group testing
- `hunk_spec.lua` - Git diff rendering
- `word_diff_spec.lua` - Inline diff virtual text
- `gs_helpers.lua` - Shared test utilities

**Testing Virtual Text and Highlights:**
```lua
-- From highlights_spec.lua
local default_attrs = {
  [1] = { foreground = Screen.colors.DarkBlue },
  [2] = { foreground = Screen.colors.NvimDarkCyan },
  [3] = { background = Screen.colors.LightBlue },
}
screen:set_default_attr_ids(default_attrs)

-- Test highlight derivation
expectf(function()
  match_dag({
    p('Deriving GitSignsAdd from DiffAdd'),
    p('Deriving GitSignsAddNr from GitSignsAdd'),
    p('Deriving GitSignsChangeLn from DiffChange'),
  })
end)

-- Enable visual testing modes
command('set termguicolors')
config.numhl = true    -- number line highlights
config.linehl = true   -- line highlights
```

**Testing Buffer State:**
```lua
-- Helper function pattern
check({
  status = { head = 'main', added = 1, changed = 0, removed = 0 },
  signs = { untracked = 1 },
})

-- Screen assertion with virtual text
screen:expect({
  grid = [[
  ^{MATCH:This {6: You, %d second.}}|
  is                  |
  ]],
})
```

**Repository:** https://github.com/lewis6991/gitsigns.nvim

### Telescope.nvim Testing

**Structure:** Uses plenary.nvim test harness
- `lua/tests/automated/` - Automated test suites
- `lua/tests/pickers/` - Picker-specific tests
- `lua/tests/fixtures/` - Test data
- `lua/tests/helpers.lua` - Test utilities

**Pattern:**
```lua
describe("telescope basics", function()
  before_each(function()
    -- Setup
  end)

  it("filters results correctly", function()
    assert.equals(expected, actual)
  end)
end)
```

**Repository:** https://github.com/nvim-telescope/telescope.nvim

### Mini.nvim Testing (Best Practice Example)

Comprehensive testing documentation at `TESTING.md`:

**Parametrized Testing:**
```lua
T['set_lines()'] = MiniTest.new_set({
  parametrize = {
    {},                    -- Test case 1: no args
    { 0, { 'a' } },       -- Test case 2
    { 0, { 1, 2, 3 } }    -- Test case 3
  }
})

T['set_lines()']['works'] = function(buf_id, lines)
  child.lua('M.set_lines(...)', { buf_id, lines })
  expect.reference_screenshot(child.get_screenshot())
end
```

**Test Hooks:**
```lua
local T = MiniTest.new_set({
  hooks = {
    pre_case = function()
      -- Run before each test
      child.restart({'-u', 'scripts/minimal_init.lua'})
      child.bo.readonly = false
    end,
    post_once = function()
      -- Run once after all tests
      child.stop()
    end,
  },
})
```

**Repository:** https://github.com/echasnovski/mini.nvim

---

## 4. Testing Frameworks

### A. Plenary.nvim Test Harness

**Best for:** Most Lua-based Neovim plugins (used by Telescope, many others)

**Setup:**
```lua
-- In test file: tests/my_test_spec.lua
local describe = require('plenary.busted').describe
local it = require('plenary.busted').it
local assert = require('luassert')

describe("my plugin", function()
  local my_plugin

  before_each(function()
    my_plugin = require('my_plugin')
  end)

  it("renders buffer correctly", function()
    local buf = vim.api.nvim_create_buf(false, true)
    my_plugin.render(buf)

    local lines = vim.api.nvim_buf_get_lines(buf, 0, -1, false)
    assert.equals("# Title", lines[1])
  end)
end)
```

**Running:**
```bash
nvim --headless -c "PlenaryBustedDirectory tests/"
```

**Features:**
- Busted-style syntax (`describe`, `it`, `before_each`)
- Runs in Neovim's Lua environment (has `vim` module access)
- Supports mocks, stubs, spies via luassert
- Async testing via coroutines

**Async Testing:**
```lua
it("handles async operations", function()
  local co = coroutine.running()

  vim.defer_fn(function()
    coroutine.resume(co)
  end, 100)

  coroutine.yield()
  -- Continues after deferred function executes
end)
```

**Resources:**
- https://github.com/nvim-lua/plenary.nvim
- https://github.com/nvim-lua/plenary.nvim/blob/master/TESTS_README.md

### B. Busted with nlua

**Best for:** Plugins that prefer standard Lua testing tools

**Setup:**

1. **Install nlua:**
```bash
luarocks --local install nlua
```

2. **Create `.busted` config:**
```lua
-- .busted
return {
  _all = {
    lua = "nlua"
  }
}
```

3. **Create rockspec with test dependencies:**
```lua
-- my-plugin-dev-1.rockspec
test_dependencies = {
  "nlua",
  "busted"
}

test = {
  type = "busted"
}
```

4. **Write tests in `spec/` directory:**
```lua
-- spec/my_plugin_spec.lua
describe("my plugin", function()
  it("works", function()
    local result = vim.api.nvim_eval("1 + 1")
    assert.equals(2, result)
  end)
end)
```

**Running:**
```bash
busted --lua nlua spec/
# Or with luarocks
luarocks test
```

**Resources:**
- https://github.com/mfussenegger/nlua
- https://mrcjkb.dev/posts/2023-06-06-luarocks-test.html
- https://hiphish.github.io/blog/2024/01/29/testing-neovim-plugins-with-busted/

### C. Mini.test

**Best for:** Plugin authors who want the most complete testing solution

**Features:**
- Child Neovim process testing
- Screenshot-based snapshot testing
- Parametrized tests
- Custom expectations
- Hierarchical test organization

**Setup:**

1. **Add mini.nvim as dependency:**
```bash
mkdir -p deps
git clone https://github.com/echasnovski/mini.nvim deps/mini.nvim
```

2. **Create minimal init script:**
```lua
-- scripts/minimal_init.lua
vim.cmd([[let &rtp.=','.getcwd()]])

if #vim.api.nvim_list_uis() == 0 then
  vim.cmd('set rtp+=deps/mini.nvim')
  require('mini.test').setup()
end
```

3. **Write tests:**
```lua
-- tests/test_my_plugin.lua
local MiniTest = require('mini.test')
local new_set = MiniTest.new_set
local expect, eq = MiniTest.expect, MiniTest.expect.equality

local child = MiniTest.new_child_neovim()

local T = new_set({
  hooks = {
    pre_case = function()
      child.restart({ '-u', 'scripts/minimal_init.lua' })
    end,
    post_once = child.stop,
  },
})

T['my feature'] = new_set()

T['my feature']['works'] = function()
  child.lua([[require('my_plugin').setup()]])
  eq(child.lua_get([[require('my_plugin').get_value()]]), 42)
end

return T
```

**Running:**
```bash
make test
# Or directly
nvim --headless --noplugin -u ./scripts/minimal_init.lua -c "lua MiniTest.run()"
```

**Resources:**
- https://github.com/echasnovski/mini.nvim/blob/main/TESTING.md
- https://github.com/echasnovski/mini.test

### D. nvim-oxi for Rust Plugins

**Best for:** Neovim plugins written in Rust

**Setup:**

1. **Add to `Cargo.toml`:**
```toml
[dependencies]
nvim-oxi = "0.5"

[dev-dependencies]
nvim-oxi = { version = "0.5", features = ["test"] }
```

2. **Write tests:**
```rust
use nvim_oxi::api;

#[nvim_oxi::test]
fn test_buffer_operations() {
    let buf = api::create_buf(false, true).unwrap();

    api::buf_set_lines(buf, 0, -1, false, vec!["line 1", "line 2"])
        .unwrap();

    let lines = api::buf_get_lines(buf, 0, -1, false).unwrap();
    assert_eq!(lines, vec!["line 1", "line 2"]);
}

#[nvim_oxi::test]
fn test_extmarks() {
    let buf = api::create_buf(false, true).unwrap();
    let ns = api::create_namespace("test").unwrap();

    let opts = SetExtmarkOpts::builder()
        .virt_text(vec![("virtual text", "Comment")])
        .build();

    let mark_id = api::buf_set_extmark(buf, ns, 0, 0, &opts).unwrap();
    assert!(mark_id > 0);
}
```

**Running:**
```bash
cargo test
```

**How it works:**
- `#[nvim_oxi::test]` macro spawns Neovim process with `nvim` from `$PATH`
- Tests run using Rust's standard test framework
- Must run `cargo build` before `cargo test` after code changes

**Resources:**
- https://github.com/noib3/nvim-oxi
- https://crates.io/crates/nvim-oxi

---

## 5. What to Test - Best Practices

### A. Testing Buffer Content Lines

**Pattern:**
```lua
-- Set content
vim.api.nvim_buf_set_lines(buf, 0, -1, false, {
  '# Heading',
  '',
  'Content line 1',
  'Content line 2'
})

-- Retrieve and assert
local lines = vim.api.nvim_buf_get_lines(buf, 0, -1, false)
assert.are.same({
  '# Heading',
  '',
  'Content line 1',
  'Content line 2'
}, lines)
```

**With child process:**
```lua
child.api.nvim_buf_set_lines(0, 0, -1, true, { 'test line' })
eq(child.api.nvim_buf_get_lines(0, 0, -1, true), { 'test line' })
```

**Important notes:**
- Indices are 0-based
- `-1` means end of buffer
- `strict_indexing` parameter controls out-of-bounds behavior

### B. Testing Extmarks and Properties

**Setting extmarks:**
```lua
local ns_id = vim.api.nvim_create_namespace('test')
local buf = vim.api.nvim_get_current_buf()

local extmark_id = vim.api.nvim_buf_set_extmark(buf, ns_id, 0, 0, {
  virt_text = {{"Virtual text", "Comment"}},
  virt_text_pos = 'eol',
  hl_group = 'Search',
  priority = 100,
})
```

**Retrieving and testing extmarks:**
```lua
-- Get all extmarks in namespace
local marks = vim.api.nvim_buf_get_extmarks(
  buf,
  ns_id,
  0,      -- start line
  -1,     -- end line (-1 = end of buffer)
  { details = true }
)

-- marks format: { { id, row, col, details }, ... }
assert.equals(1, #marks)
assert.equals(extmark_id, marks[1][1])
assert.equals(0, marks[1][2])  -- row
assert.equals(0, marks[1][3])  -- col

-- Check details
local details = marks[1][4]
assert.equals('eol', details.virt_text_pos)
assert.equals(100, details.priority)
```

**Testing specific extmark properties:**
```lua
-- Get specific extmark
local mark = vim.api.nvim_buf_get_extmark_by_id(
  buf,
  ns_id,
  extmark_id,
  { details = true }
)

-- mark format: { row, col, details }
assert.equals(0, mark[1])
assert.equals(0, mark[2])
assert.is_not_nil(mark[3].virt_text)
```

### C. Testing Virtual Text Placement

**Different virtual text positions:**
```lua
local test_cases = {
  { pos = 'eol', desc = 'End of line' },
  { pos = 'overlay', desc = 'Overlay existing text' },
  { pos = 'right_align', desc = 'Right-aligned in window' },
  { pos = 'inline', desc = 'Inline with text' },
}

for _, case in ipairs(test_cases) do
  it(case.desc, function()
    local mark_id = vim.api.nvim_buf_set_extmark(buf, ns, 0, 0, {
      virt_text = {{"Test", "Comment"}},
      virt_text_pos = case.pos,
    })

    local mark = vim.api.nvim_buf_get_extmark_by_id(
      buf, ns, mark_id, { details = true }
    )

    assert.equals(case.pos, mark[3].virt_text_pos)
  end)
end
```

**Note:** `nvim_buf_get_extmarks` returns extmark position metadata but not the actual rendered virtual text content. For visual verification, use screenshot testing.

### D. Testing Highlight Group Application

**Modern approach using extmarks:**
```lua
-- Create namespace
local ns = vim.api.nvim_create_namespace('my_plugin_highlight')

-- Apply highlight via extmark
vim.api.nvim_buf_set_extmark(buf, ns, 0, 0, {
  end_col = 10,
  hl_group = 'Search',
})

-- Retrieve and verify
local marks = vim.api.nvim_buf_get_extmarks(
  buf, ns, 0, -1, { details = true }
)

assert.equals('Search', marks[1][4].hl_group)
```

**Testing custom highlight groups:**
```lua
-- Define custom highlight
vim.api.nvim_set_hl(0, 'MyCustomHighlight', {
  fg = '#ff0000',
  bg = '#000000',
  bold = true,
})

-- Apply it
vim.api.nvim_buf_set_extmark(buf, ns, 0, 0, {
  end_col = 5,
  hl_group = 'MyCustomHighlight',
})

-- Verify highlight definition
local hl = vim.api.nvim_get_hl(0, { name = 'MyCustomHighlight' })
assert.equals('#ff0000', hl.fg)
assert.equals('#000000', hl.bg)
assert.is_true(hl.bold)
```

**Visual testing with screenshots:**
```lua
-- Setup highlights
vim.cmd('highlight TestHighlight guifg=#ff0000')

-- Apply to buffer
child.lua([[
  local ns = vim.api.nvim_create_namespace('test')
  vim.api.nvim_buf_set_extmark(0, ns, 0, 0, {
    end_col = 10,
    hl_group = 'TestHighlight',
  })
]])

-- Verify visually
expect.reference_screenshot(child.get_screenshot())
```

### E. Testing Buffer Options

**Testing buftype, filetype, modifiable:**
```lua
it("creates scratch buffer with correct options", function()
  local buf = vim.api.nvim_create_buf(false, true)

  -- Set options
  vim.api.nvim_buf_set_option(buf, 'buftype', 'nofile')
  vim.api.nvim_buf_set_option(buf, 'filetype', 'confluence')
  vim.api.nvim_buf_set_option(buf, 'modifiable', false)
  vim.api.nvim_buf_set_option(buf, 'bufhidden', 'wipe')

  -- Verify
  assert.equals('nofile', vim.api.nvim_buf_get_option(buf, 'buftype'))
  assert.equals('confluence', vim.api.nvim_buf_get_option(buf, 'filetype'))
  assert.is_false(vim.api.nvim_buf_get_option(buf, 'modifiable'))
  assert.equals('wipe', vim.api.nvim_buf_get_option(buf, 'bufhidden'))
end)
```

**With child process:**
```lua
child.lua([[
  local buf = vim.api.nvim_create_buf(false, true)
  vim.api.nvim_buf_set_option(buf, 'filetype', 'confluence')
  _G.test_buf = buf
]])

local ft = child.lua_get([[vim.api.nvim_buf_get_option(_G.test_buf, 'filetype')]])
eq('confluence', ft)
```

### F. Complete Example: Testing Confluence Page Rendering

```lua
-- tests/test_page_renderer.lua
local MiniTest = require('mini.test')
local new_set = MiniTest.new_set
local expect, eq = MiniTest.expect, MiniTest.expect.equality

local child = MiniTest.new_child_neovim()

local T = new_set({
  hooks = {
    pre_case = function()
      child.restart({ '-u', 'scripts/minimal_init.lua' })
      child.lua([[M = require('confluence-nvim.renderer')]])
    end,
    post_once = child.stop,
  },
})

T['page rendering'] = new_set()

T['page rendering']['creates buffer with correct options'] = function()
  child.lua([[
    local buf = M.create_page_buffer()
    _G.page_buf = buf
  ]])

  -- Test buffer options
  eq('nofile', child.lua_get([[vim.bo[_G.page_buf].buftype]]))
  eq('confluence', child.lua_get([[vim.bo[_G.page_buf].filetype]]))
  eq(false, child.lua_get([[vim.bo[_G.page_buf].modifiable]]))
end

T['page rendering']['renders title correctly'] = function()
  child.lua([[
    local content = {
      title = "Test Page",
      body = "Content here"
    }
    M.render_page(_G.page_buf, content)
  ]])

  local lines = child.lua_get([[
    return vim.api.nvim_buf_get_lines(_G.page_buf, 0, 3, false)
  ]])

  eq('# Test Page', lines[1])
  eq('', lines[2])
  eq('Content here', lines[3])
end

T['page rendering']['applies correct highlights'] = function()
  child.lua([[
    M.render_page(_G.page_buf, { title = "Test", body = "Content" })
  ]])

  -- Get extmarks for title highlighting
  local marks = child.lua_get([[
    local ns = vim.api.nvim_create_namespace('confluence')
    return vim.api.nvim_buf_get_extmarks(
      _G.page_buf, ns, 0, -1, { details = true }
    )
  ]])

  -- Verify title highlight exists
  expect.truthy(#marks > 0)
  expect.truthy(marks[1][4].hl_group)
end

T['page rendering']['visual snapshot matches'] = function()
  child.o.lines, child.o.columns = 25, 80

  child.lua([[
    local content = {
      title = "Confluence Page",
      metadata = { space = "DOCS", id = "12345" },
      body = "Page content with **bold** and _italic_"
    }

    local buf = M.create_page_buffer()
    M.render_page(buf, content)
    vim.api.nvim_set_current_buf(buf)
  ]])

  -- Visual regression test
  expect.reference_screenshot(child.get_screenshot())
end

T['page rendering']['renders metadata as virtual text'] = function()
  child.lua([[
    local content = {
      title = "Test",
      metadata = { author = "John Doe", updated = "2025-01-05" },
      body = "Content"
    }
    M.render_page(_G.page_buf, content)
  ]])

  -- Get extmarks with virtual text
  local marks = child.lua_get([[
    local ns = vim.api.nvim_create_namespace('confluence')
    return vim.api.nvim_buf_get_extmarks(
      _G.page_buf, ns, 0, -1, { details = true }
    )
  ]])

  -- Find virtual text extmark
  local has_virt_text = false
  for _, mark in ipairs(marks) do
    if mark[4].virt_text then
      has_virt_text = true
      break
    end
  end

  expect.truthy(has_virt_text)
end

return T
```

---

## 6. Recommended Testing Strategy for Confluence Plugin

### Suggested Approach

1. **Use mini.test as primary framework**
   - Most comprehensive for buffer rendering
   - Screenshot testing catches visual regressions
   - Child process isolation prevents interference

2. **Test pyramid:**
   - **Unit tests** (60%): Individual rendering functions, parsers
   - **Integration tests** (30%): Buffer creation + rendering pipeline
   - **Screenshot tests** (10%): Full page rendering visual verification

3. **What to test:**
   - ✅ Buffer creation with correct options
   - ✅ Markdown rendering to buffer lines
   - ✅ Confluence storage format parsing
   - ✅ Syntax highlighting application
   - ✅ Virtual text for metadata (author, date, version)
   - ✅ Extmarks for special elements (macros, attachments)
   - ✅ Link rendering and concealment
   - ✅ Table rendering
   - ✅ Code block formatting
   - ✅ Full page layout (snapshot test)

4. **Project structure:**
```
confluence-nvim/
├── deps/
│   └── mini.nvim/          # Testing dependency
├── lua/
│   └── confluence-nvim/
│       ├── renderer.lua
│       └── parser.lua
├── scripts/
│   └── minimal_init.lua    # Test initialization
├── tests/
│   ├── screenshots/        # Reference screenshots
│   ├── test_renderer.lua
│   ├── test_parser.lua
│   └── test_integration.lua
└── Makefile                # `make test` command
```

### Sample Makefile

```makefile
.PHONY: test
test:
	nvim --headless --noplugin -u ./scripts/minimal_init.lua \
		-c "lua MiniTest.run()" \
		&& echo "Tests passed!"

.PHONY: test-file
test-file:
	nvim -u ./scripts/minimal_init.lua -c "lua MiniTest.run_file()"

.PHONY: test-update-screenshots
test-update-screenshots:
	rm -rf tests/screenshots/
	$(MAKE) test
```

---

## 7. Key Resources

### Documentation
- [Mini.nvim Testing Guide](https://github.com/echasnovski/mini.nvim/blob/main/TESTING.md)
- [Plenary.nvim Tests README](https://github.com/nvim-lua/plenary.nvim/blob/master/TESTS_README.md)
- [Neovim API Documentation](https://neovim.io/doc/user/api.html)
- [Neovim Dev Testing](https://neovim.io/doc/user/dev_test.html)

### Blog Posts & Tutorials
- [Test your Neovim plugins with luarocks and busted](https://mrcjkb.dev/posts/2023-06-06-luarocks-test.html)
- [Testing Neovim plugins with Busted](https://hiphish.github.io/blog/2024/01/29/testing-neovim-plugins-with-busted/)
- [Testing Neovim LSP plugins](https://zignar.net/2022/10/26/testing-neovim-lsp-plugins/)
- [Using Virtual Text in Neovim](https://jdhao.github.io/2021/09/09/nvim_use_virtual_text/)

### Real Plugin Test Suites
- [gitsigns.nvim tests](https://github.com/lewis6991/gitsigns.nvim/tree/main/test) - Virtual text, highlights, extmarks
- [telescope.nvim tests](https://github.com/nvim-telescope/telescope.nvim/tree/master/lua/tests) - UI picker testing
- [mini.nvim tests](https://github.com/echasnovski/mini.nvim) - Comprehensive examples across all mini.* modules
- [Neovim core tests](https://github.com/neovim/neovim/tree/master/test/functional) - Screen testing examples

### Tools & Libraries
- [plenary.nvim](https://github.com/nvim-lua/plenary.nvim) - Test harness and utilities
- [mini.test](https://github.com/echasnovski/mini.test) - Comprehensive testing framework
- [nlua](https://github.com/mfussenegger/nlua) - Neovim as Lua interpreter for busted
- [nvim-oxi](https://github.com/noib3/nvim-oxi) - Rust bindings with test support
- [luassert](https://github.com/lunarmodules/luassert) - Assertion library (bundled with plenary)

---

## Summary

For testing Confluence page rendering in your Neovim plugin:

1. **Use mini.test** for comprehensive testing with child processes and screenshots
2. **Test buffer content** with `nvim_buf_get_lines()` assertions
3. **Test extmarks** with `nvim_buf_get_extmarks()` for virtual text and highlights
4. **Use reference screenshots** for visual regression testing of full pages
5. **Follow gitsigns.nvim patterns** for testing virtual text and highlight application
6. **Organize tests** into unit (functions), integration (buffer rendering), and snapshot (visual) tests

The combination of API assertions for structure and screenshot testing for visuals provides robust coverage for buffer rendering features.
