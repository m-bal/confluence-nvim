# confluence-nvim

A Neovim plugin for reading Confluence documentation with native rendering, built using **Verification-Driven Development (VDD)** methodology.

## Features

- 🔍 **Browse Confluence spaces and pages** via Telescope integration
- 🔐 **Secure by design** - HTTPS-only, input validation, injection prevention
- ⚡ **Async API calls** - Non-blocking with proper error handling
- 📝 **Rich content rendering** - Beautiful bordered boxes, tables, code blocks with syntax
- 🔗 **Link following** - Navigate between pages with `gx` or `<C-]>` keymaps
- ⏮️ **Navigation history** - Go back/forward through visited pages (Vim tag-stack style)
- 🗺️ **Breadcrumbs** - View page hierarchy and space information
- 📊 **Statusline integration** - Show current page in lualine/statusline
- 🎯 **Search functionality** - Full-text search across your Confluence instance
- 🔒 **Security-hardened** - 10 critical vulnerabilities fixed via adversarial review

## Requirements

- Neovim >= 0.9.0
- [telescope.nvim](https://github.com/nvim-telescope/telescope.nvim) (required)
- `curl` command-line tool (for API requests)
- `base64` command (usually pre-installed on Unix systems)

## Installation

### Using [lazy.nvim](https://github.com/folke/lazy.nvim)

```lua
{
  'm-bal/confluence-nvim',
  dependencies = {
    'nvim-telescope/telescope.nvim',
  },
  config = function()
    require('confluence').setup({
      confluence_url = 'https://your-company.atlassian.net/wiki',
      auth = {
        type = 'token',  -- 'token' for Confluence Cloud, 'pat' for Data Center
        email = 'your.email@example.com',  -- Required for 'token' type
        token = 'your-api-token',  -- Get from https://id.atlassian.com/manage-profile/security/api-tokens
      },
    })
  end,
}
```

## Usage

### Telescope Commands

```vim
" Browse all Confluence spaces
:Telescope confluence spaces

" Search Confluence
:Telescope confluence search
```

### Keybindings

In Telescope pickers:

- `<CR>` - Open selected page/space
- `<C-x>` - Open page in horizontal split
- `<C-v>` - Open page in vertical split

In Confluence page buffers:

- `gx` or `<C-]>` - Follow link under cursor to another Confluence page
- `<C-t>` - Go back in navigation history
- `<C-i>` - Go forward in navigation history
- `<leader>cb` - Show breadcrumbs (space > page hierarchy)

### Statusline Integration

To show the current Confluence page in your statusline, add this to your lualine config:

```lua
require('lualine').setup({
  sections = {
    lualine_c = {
      'filename',
      function()
        return require('confluence').statusline()
      end,
    },
  },
})
```

This will display: `🏢 [SPACE] Page Title` when viewing Confluence pages.

## Security

- **HTTPS Only**: HTTP URLs rejected
- **Input Validation**: All inputs sanitized
- **10MB Response Limit**: Prevents DoS
- **100-Level Depth Limit**: Prevents stack overflow

See [ADVERSARIAL_FINDINGS.md](./ADVERSARIAL_FINDINGS.md) for full security audit.

## License

MIT OR Apache-2.0
