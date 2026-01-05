# Confluence Neovim UI/UX Design

## Design Principles

1. **Vim-Native**: Use familiar Vim patterns (buffers, splits, motions)
2. **Telescope-First**: Leverage telescope.nvim for fuzzy finding when available
3. **Graceful Fallback**: Plain buffer UI if telescope not installed
4. **Keyboard-Driven**: All actions accessible via keybindings
5. **Minimal Context Switching**: Stay in Neovim, avoid external tools

---

## Entry Points

### Command-Based Entry

```vim
:Confluence              " Open space browser (main entry point)
:ConfluenceSpace <key>   " Open specific space (e.g., :ConfluenceSpace PROJ)
:ConfluenceSearch <q>    " Search all spaces
:ConfluenceRecent        " Show recent pages
:ConfluenceOpen <id>     " Open page by ID or URL
```

### Keymap-Based Entry (Optional User Config)

```lua
vim.keymap.set('n', '<leader>cc', ':Confluence<CR>', { desc = 'Confluence: Open browser' })
vim.keymap.set('n', '<leader>cs', ':ConfluenceSearch ', { desc = 'Confluence: Search' })
vim.keymap.set('n', '<leader>cr', ':ConfluenceRecent<CR>', { desc = 'Confluence: Recent pages' })
```

---

## UI Option 1: Telescope Picker (Preferred)

### Space Browser (`:Confluence`)

```
┌─ Confluence Spaces ─────────────────────────────────────────┐
│ > proj                                                       │
│                                                              │
│   🏢 PROJ · Project Documentation · 156 pages               │
│   📚 ENG  · Engineering Docs      · 89 pages                │
│   🎨 DESIGN · Design System       · 42 pages                │
│   📖 KB   · Knowledge Base        · 234 pages               │
│   🚀 PROD · Product Specs         · 67 pages                │
│                                                              │
│ [4/5]                                                        │
└──────────────────────────────────────────────────────────────┘

Actions:
  <CR>     - Open space page tree
  <C-s>    - Search within space
  <C-x>    - Open space homepage in split
  <C-v>    - Open space homepage in vsplit
  <C-t>    - Open space homepage in tab
```

**Implementation:**
```lua
require('telescope.pickers').new(opts, {
  prompt_title = 'Confluence Spaces',
  finder = require('telescope.finders').new_table({
    results = spaces,
    entry_maker = function(space)
      return {
        value = space,
        display = string.format('🏢 %s · %s · %d pages',
          space.key, space.name, space.page_count),
        ordinal = space.key .. ' ' .. space.name,
      }
    end,
  }),
  sorter = require('telescope.config').values.generic_sorter(opts),
  attach_mappings = function(prompt_bufnr, map)
    actions.select_default:replace(function()
      local selection = action_state.get_selected_entry()
      actions.close(prompt_bufnr)
      open_space_pages(selection.value.key)
    end)
    return true
  end,
}):find()
```

### Page Tree Picker (After selecting a space)

```
┌─ PROJ: Project Documentation ───────────────────────────────┐
│ > getting                                                    │
│                                                              │
│   📄 Getting Started Guide          · updated 2h ago        │
│   📁 Architecture                   · 12 child pages        │
│      📄 System Architecture         · updated 1d ago        │
│      📄 Database Schema             · updated 3d ago        │
│      📁 API Design                  · 5 child pages         │
│   📁 Development                    · 8 child pages         │
│   📄 Contributing Guidelines        · updated 1w ago        │
│   📁 Deployment                     · 6 child pages         │
│                                                              │
│ [4/156]                                                      │
└──────────────────────────────────────────────────────────────┘

Actions:
  <CR>     - Open page
  <C-s>    - Search within space
  <C-p>    - Go to parent page
  <Tab>    - Expand/collapse folder
  <C-x>    - Open in split
  <C-v>    - Open in vsplit
  <C-r>    - Refresh space
```

**Hierarchy Display:**
- Indent child pages with spaces
- Use icons to distinguish pages (📄) from parent pages (📁)
- Show metadata: last updated, comment count

### Search Results (`:ConfluenceSearch <query>`)

```
┌─ Search: "authentication" ──────────────────────────────────┐
│ > api                                                        │
│                                                              │
│   [PROJ] API Authentication Guide                           │
│   Overview of OAuth2 and JWT authentication methods...      │
│                                                              │
│   [ENG] Auth Service Architecture                           │
│   The authentication service handles user login and...      │
│                                                              │
│   [KB] Troubleshooting Auth Issues                          │
│   Common problems: token expiration, CORS errors...         │
│                                                              │
│ [3/12]                                                       │
└──────────────────────────────────────────────────────────────┘

Display:
  - [SPACE_KEY] Page Title
  - Excerpt with search term highlighted
  - Relevance score (if provided by API)
```

### Recent Pages (`:ConfluenceRecent`)

```
┌─ Recent Confluence Pages ───────────────────────────────────┐
│ > api                                                        │
│                                                              │
│   [PROJ] API Authentication Guide      · viewed 10m ago     │
│   [ENG] Database Schema                · viewed 1h ago      │
│   [PROD] Q1 Product Roadmap            · viewed 3h ago      │
│   [PROJ] Getting Started Guide         · viewed 1d ago      │
│   [KB] Onboarding Checklist            · viewed 2d ago      │
│                                                              │
│ [5/10]                                                       │
└──────────────────────────────────────────────────────────────┘

Persistence:
  - Store in ~/.local/share/nvim/confluence_history.json
  - Track: page_id, title, space_key, last_viewed_at
  - Limit to 50 most recent
```

---

## UI Option 2: Buffer-Based Browser (Fallback)

When telescope is not available, use a dedicated buffer with custom keybindings.

### Space Browser Buffer

```
┌──────────────────────────────────────────────────────────────┐
│ Confluence Spaces                                    [Help: ?]│
├──────────────────────────────────────────────────────────────┤
│                                                              │
│  🏢 PROJ      Project Documentation                156 pages │
│  📚 ENG       Engineering Docs                      89 pages │
│  🎨 DESIGN    Design System                         42 pages │
│  📖 KB        Knowledge Base                       234 pages │
│  🚀 PROD      Product Specs                         67 pages │
│                                                              │
│                                                              │
│                                                              │
└──────────────────────────────────────────────────────────────┘

Buffer properties:
  - buftype=nofile
  - bufhidden=wipe
  - modifiable=false
  - filetype=confluence-browser

Keybindings (buffer-local):
  <CR>     - Open space page tree
  s        - Search within space
  r        - Refresh space list
  ?        - Toggle help
  q        - Close browser
```

**Implementation:**
```lua
local function create_space_browser()
  local buf = vim.api.nvim_create_buf(false, true)
  vim.api.nvim_buf_set_name(buf, 'confluence://spaces')

  -- Set buffer options
  vim.api.nvim_buf_set_option(buf, 'buftype', 'nofile')
  vim.api.nvim_buf_set_option(buf, 'bufhidden', 'wipe')
  vim.api.nvim_buf_set_option(buf, 'modifiable', false)
  vim.api.nvim_buf_set_option(buf, 'filetype', 'confluence-browser')

  -- Render content
  local lines = {}
  table.insert(lines, ' Confluence Spaces' .. string.rep(' ', 50) .. '[Help: ?]')
  table.insert(lines, string.rep('─', 80))
  table.insert(lines, '')

  for _, space in ipairs(spaces) do
    table.insert(lines, string.format('  🏢 %-10s %-40s %3d pages',
      space.key, space.name, space.page_count))
  end

  vim.api.nvim_buf_set_option(buf, 'modifiable', true)
  vim.api.nvim_buf_set_lines(buf, 0, -1, false, lines)
  vim.api.nvim_buf_set_option(buf, 'modifiable', false)

  -- Set up keybindings
  local opts = { buffer = buf, silent = true }
  vim.keymap.set('n', '<CR>', function()
    local line = vim.api.nvim_get_current_line()
    local space_key = line:match('^%s+🏢%s+(%S+)')
    if space_key then
      open_space_pages(space_key)
    end
  end, opts)

  vim.keymap.set('n', 'q', ':close<CR>', opts)

  return buf
end
```

### Page Tree Buffer

```
┌──────────────────────────────────────────────────────────────┐
│ PROJ: Project Documentation                         [Help: ?]│
├──────────────────────────────────────────────────────────────┤
│                                                              │
│  📄 Getting Started Guide                      updated 2h ago│
│  📁 Architecture [+]                          12 child pages │
│  📁 Development [+]                            8 child pages │
│  📄 Contributing Guidelines                   updated 1w ago │
│  📁 Deployment [+]                             6 child pages │
│                                                              │
│                                                              │
│                                                              │
│                                                              │
└──────────────────────────────────────────────────────────────┘

Expandable tree:
  [+] collapsed folder
  [-] expanded folder

Keybindings:
  <CR>     - Open page or toggle folder
  <Tab>    - Toggle folder expansion
  o        - Open in current window
  s        - Open in horizontal split
  v        - Open in vertical split
  t        - Open in new tab
  u        - Go up to parent
  b        - Back to space browser
  /        - Search in space
  r        - Refresh tree
  ?        - Toggle help
  q        - Close browser
```

**Tree State Management:**
```lua
-- Track expanded folders
local expanded_folders = {}

local function toggle_folder(page_id)
  expanded_folders[page_id] = not expanded_folders[page_id]
  refresh_tree_buffer()
end

local function render_tree(pages, indent)
  local lines = {}
  for _, page in ipairs(pages) do
    local prefix = string.rep('  ', indent)
    local icon = page.has_children and '📁' or '📄'
    local expand_icon = ''

    if page.has_children then
      expand_icon = expanded_folders[page.id] and '[-]' or '[+]'
    end

    local line = string.format('%s%s %s %s',
      prefix, icon, expand_icon, page.title)
    table.insert(lines, line)

    -- Recursively render children if expanded
    if page.has_children and expanded_folders[page.id] then
      local children = fetch_child_pages(page.id)
      vim.list_extend(lines, render_tree(children, indent + 1))
    end
  end
  return lines
end
```

---

## Page View UI

### Main Content Buffer

```
┌──────────────────────────────────────────────────────────────┐
│ confluence://page/123456789                                  │
│                                                              │
│ # API Authentication Guide                                   │
│                                                              │
│ Space: PROJ · Author: Alice Smith · Updated: 2 hours ago    │
│ 💬 3 comments · 👁 124 views                                 │
│ ────────────────────────────────────────────────────────────  │
│                                                              │
│ ## Overview                                                  │
│                                                              │
│ This guide covers OAuth2 and JWT authentication...          │
│                                                              │
│ ┌─ ℹ INFO ───────────────────────────────────────────────┐  │
│ │ Authentication is required for all API endpoints       │  │
│ └────────────────────────────────────────────────────────┘  │
│                                                              │
│ ## Setup                                                     │
│                                                              │
│ ┌─ RUST ─────────────────────────────────────────────────┐  │
│ │  1  use oauth2::ClientId;                              │  │
│ │  2  let client = ClientId::new("your_client_id");      │  │
│ └────────────────────────────────────────────────────────┘  │
│                                                              │
└──────────────────────────────────────────────────────────────┘

Buffer properties:
  - buftype=nofile
  - bufhidden=hide
  - modifiable=false
  - filetype=confluence
  - b:confluence_page_id = "123456789"
  - b:confluence_space_key = "PROJ"
  - b:confluence_title = "API Authentication Guide"

Keybindings (buffer-local):
  gx       - Follow link under cursor
  <C-]>    - Follow Confluence link (open in new buffer)
  <C-t>    - Go back (like tag stack)
  <leader>r - Refresh page
  <leader>c - Show comments
  <leader>e - Edit page (if permissions allow)
  <leader>b - Show breadcrumbs (page hierarchy)
```

### Breadcrumb/Navigation Bar

Display at top of page buffer or in statusline:

```
Option 1 (Top of buffer):
│ 🏠 PROJ > Architecture > System Design > API Authentication │

Option 2 (Statusline):
%{b:confluence_breadcrumb}

Option 3 (Floating window on <leader>b):
┌─ Page Hierarchy ─────────────────────┐
│ 🏠 PROJ: Project Documentation       │
│   📁 Architecture                    │
│     📁 System Design                 │
│       📄 API Authentication Guide ◄──│
└──────────────────────────────────────┘
```

### Comment Sidebar (`:ConfluenceComments` or `<leader>c`)

```
┌─ Comments (3) ────────────────────────────┐
│                                           │
│ 👤 Alice Smith · 2 hours ago              │
│ Should we add examples for refresh        │
│ tokens?                                   │
│                                           │
│   ↳ 👤 Bob Jones · 1 hour ago             │
│     Good idea, I'll add that section      │
│                                           │
│ 👤 Charlie · Yesterday                    │
│ Great documentation! 👍                   │
│                                           │
│ ───────────────────────────────────────── │
│ Press <CR> to add comment                 │
└───────────────────────────────────────────┘

Display options:
  1. Floating window (default)
  2. Vertical split (:ConfluenceComments split)
  3. At bottom of page buffer (inline)
```

### Inline Comment Indicators

```
│ This guide covers OAuth2 and JWT authentication...   💬 2
│                                                         ▲
│                              Cursor here shows popup ──┘

Popup on CursorHold:
┌─ Inline Comments ────────────┐
│ 👤 Alice: Needs examples     │
│ 👤 Bob: Link to RFC 6749?    │
└──────────────────────────────┘
```

---

## Navigation Patterns

### Link Following

**Internal Confluence Links:**
```
[Related Page]  ← cursor here, press gx or <C-]>
    ↓
Opens: confluence://page/987654321
```

**External Links:**
```
[GitHub Repo](https://github.com/...) ← gx opens in browser
```

**Attachment Links:**
```
📎 diagram.png  ← gx downloads and opens with xdg-open
```

### Navigation History

Similar to Vim's tag stack:
```
<C-]>  - Follow link (push to stack)
<C-t>  - Go back
<C-i>  - Go forward

:ConfluenceHistory
┌─ Navigation History ─────────┐
│ 1. Getting Started Guide     │
│ 2. Architecture Overview     │
│ 3. API Authentication ◄──    │
└──────────────────────────────┘
```

---

## Search UI Enhancements

### Fuzzy Search with Preview

```
┌─ Search: "auth" ─────────────────────────┬─ Preview ───────┐
│ > api                                    │ # API Auth...   │
│                                          │                 │
│ [PROJ] API Authentication Guide          │ This guide...   │
│ [PROJ] Auth Service Architecture         │                 │
│ [KB] Troubleshooting Auth Issues         │                 │
│                                          │                 │
│ [3/12]                                   │                 │
└──────────────────────────────────────────┴─────────────────┘
```

### Filter by Space

```
:ConfluenceSearch --space PROJ authentication
   ↓
Only searches within PROJ space
```

### Advanced Search Syntax (Future)

```
:ConfluenceSearch type:page space:PROJ author:alice "authentication"
:ConfluenceSearch created:>2024-01-01 label:api
```

---

## Status Line Integration

### Custom Statusline Component

```lua
-- Show in statusline when viewing Confluence page
local function confluence_statusline()
  if vim.bo.filetype ~= 'confluence' then
    return ''
  end

  local page_id = vim.b.confluence_page_id
  local space = vim.b.confluence_space_key
  local title = vim.b.confluence_title

  return string.format('🏢 [%s] %s', space, title)
end

-- For lualine:
require('lualine').setup({
  sections = {
    lualine_c = { 'filename', confluence_statusline },
  }
})
```

---

## Accessibility Features

### Keyboard-Only Navigation

All features accessible without mouse:
- Fuzzy finding via Telescope
- Tree navigation with j/k
- Link following with gx/<C-]>
- Comment viewing with <leader>c

### Screen Reader Hints

```lua
-- Add aria-label equivalents in virtual text
vim.api.nvim_buf_set_extmark(buf, ns, line, 0, {
  virt_text = {{'[Link]', 'Comment'}},
  virt_text_pos = 'overlay',
})
```

### High Contrast Mode

```lua
-- Option to use simpler ASCII instead of Unicode
confluence.setup({
  render_options = {
    use_unicode = false,  -- Use ASCII box drawing
    high_contrast = true,  -- Stronger color contrast
  }
})
```

---

## Mobile/Remote Editing Considerations

### SSH/Mosh Compatibility

- Test rendering over SSH
- Fallback to ASCII if Unicode doesn't render
- Detect TERM capabilities

### tmux/screen Integration

```lua
-- Detect if running in tmux
if vim.env.TMUX then
  -- Adjust rendering for tmux capabilities
  render_options.use_24bit_color = false
end
```

---

## Performance Optimizations

### Lazy Loading

- Only fetch page tree when space is opened
- Paginate large spaces (show first 100 pages, load more)
- Cache space metadata

### Progressive Rendering

```lua
-- Show skeleton while loading
┌─ Loading... ────────────────┐
│ ▓▓▓▓▓▓▓░░░░░░░░░░░  40%    │
└─────────────────────────────┘

-- Then replace with actual content
```

### Virtual Scrolling (Future)

For very large pages, only render visible portion:
```lua
-- Render lines [viewport_start, viewport_end]
-- Update on scroll events
```

---

## Recommended Implementation Order

### Phase 1: MVP
1. `:ConfluenceOpen <id>` - Open single page
2. Basic rendering (no images, simple formatting)
3. Buffer-based space browser (no telescope yet)

### Phase 2: Enhanced Navigation
1. Telescope integration for spaces and pages
2. Link following with `gx`
3. Navigation history (back/forward)

### Phase 3: Rich Content
1. Image rendering (terminal protocol)
2. Code block syntax highlighting
3. Comment display

### Phase 4: Polish
1. Search with preview
2. Breadcrumbs
3. Status line integration
4. Recent pages tracking

---

## Testing the UI

### Visual Regression Tests

```lua
-- Using mini.test
local child = MiniTest.new_child_neovim()
child.o.lines, child.o.columns = 40, 120

-- Open space browser
child.cmd('Confluence')

-- Verify layout
expect.reference_screenshot(child.get_screenshot(), {
  path = 'tests/screenshots/space_browser.txt'
})
```

### Interaction Tests

```lua
-- Test link following
it('follows internal links with <CR>', function()
  open_test_page()

  -- Move cursor to link
  vim.fn.search('Related Page')
  vim.cmd('normal! gx')

  -- Verify new page opened
  assert.equals('confluence', vim.bo.filetype)
  assert.equals('987654321', vim.b.confluence_page_id)
end)
```

### Accessibility Tests

```lua
-- Ensure all actions have keybindings
it('all actions keyboard accessible', function()
  local mappings = vim.api.nvim_buf_get_keymap(buf, 'n')

  assert.is_not_nil(find_mapping(mappings, 'gx'))
  assert.is_not_nil(find_mapping(mappings, '<C-]>'))
  assert.is_not_nil(find_mapping(mappings, '<leader>c'))
end)
```

---

## Questions for User

Before implementing, please confirm:

1. **Telescope Dependency**: Should we require telescope.nvim, or always provide buffer-based fallback?
2. **Icon Usage**: Use Unicode icons (🏢📄📁) or ASCII alternatives ([S] [P] [F])?
3. **Default Layout**: Prefer floating windows, splits, or full-screen buffers?
4. **Recent Pages**: Track locally (JSON file) or sync with Confluence (if API supports)?
5. **Edit Support**: Include in MVP or defer to Phase 4?

Please review and let me know which design elements to prioritize!
