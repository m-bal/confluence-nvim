# confluence-nvim

A Neovim plugin for reading Confluence documentation with native-like rendering in the terminal.

## Features

- 🎨 **Rich Terminal Rendering**: View Confluence pages with visual fidelity matching the web interface
  - Colored macro panels (info, warning, error, note, success)
  - Syntax-highlighted code blocks with box-drawing frames
  - Tables with proper formatting
  - Status lozenges with colors
  - Links and navigation

- 🖼️ **Image Support**: Display images directly in supported terminals (Kitty, iTerm2, Sixel)
  - Automatic protocol detection
  - Graceful fallback for unsupported terminals

- 💬 **Comments**: View page and inline comments
  - Threaded comment display
  - Inline comment indicators

- ⚡ **Performance**: Smart caching and async operations
  - LRU cache for API responses
  - Non-blocking UI during network requests
  - Configurable cache TTL

- 🔐 **Secure Authentication**: Multiple auth methods
  - API token (Confluence Cloud)
  - Personal Access Token (Data Center)
  - Credentials never logged or stored in plaintext

## Requirements

- Neovim >= 0.10.0
- Rust toolchain (for building)
- Confluence Cloud or Data Center instance
- API token or Personal Access Token

## Installation

### Using [lazy.nvim](https://github.com/folke/lazy.nvim)

```lua
{
  'm-bal/confluence-nvim',
  build = 'cargo build --release',
  config = function()
    require('confluence').setup({
      confluence_url = 'https://your-instance.atlassian.net/wiki',
      auth = {
        email = 'your-email@example.com',  -- Required for Confluence Cloud
        token = vim.env.CONFLUENCE_API_TOKEN,
      },
    })
  end,
}
```

### Using [packer.nvim](https://github.com/wbthomason/packer.nvim)

```lua
use {
  'm-bal/confluence-nvim',
  run = 'cargo build --release',
  config = function()
    require('confluence').setup({
      confluence_url = 'https://your-instance.atlassian.net/wiki',
      auth = {
        email = 'your-email@example.com',
        token = vim.env.CONFLUENCE_API_TOKEN,
      },
    })
  end,
}
```

### Using [vim-plug](https://github.com/junegunn/vim-plug)

```vim
Plug 'm-bal/confluence-nvim', { 'do': 'cargo build --release' }
```

Then in your `init.lua`:

```lua
require('confluence').setup({
  confluence_url = 'https://your-instance.atlassian.net/wiki',
  auth = {
    email = 'your-email@example.com',
    token = vim.env.CONFLUENCE_API_TOKEN,
  },
})
```

## Configuration

### Full Configuration Example

```lua
require('confluence').setup({
  -- Required: Base URL of your Confluence instance
  confluence_url = 'https://your-instance.atlassian.net/wiki',

  -- Required: Authentication
  auth = {
    -- For Confluence Cloud: email + API token
    email = 'your-email@example.com',
    token = vim.env.CONFLUENCE_API_TOKEN,

    -- For Confluence Data Center: PAT only
    -- token = vim.env.CONFLUENCE_PAT,
  },

  -- Optional: Caching configuration
  cache_enabled = true,
  cache_ttl = 900,  -- 15 minutes
  cache_size = 50,  -- Maximum number of pages to cache

  -- Optional: Rendering options
  render_options = {
    enable_syntax_highlighting = true,
    enable_images = true,
    max_image_width = 80,
  },
})
```

### Obtaining API Tokens

#### Confluence Cloud

1. Go to https://id.atlassian.com/manage-profile/security/api-tokens
2. Click "Create API token"
3. Give it a label and copy the token
4. Use with your email address in the configuration

#### Confluence Data Center

1. Go to your Confluence instance settings
2. Navigate to Personal Access Tokens
3. Create a new token
4. Use the token without an email address

**Security Note**: Never commit tokens to version control. Use environment variables:

```bash
export CONFLUENCE_API_TOKEN='your-token-here'
```

## Usage

### Commands

- `:ConfluenceOpen <page-id>` - Open a Confluence page by ID or URL
- `:ConfluenceSearch <query>` - Search for Confluence pages
- `:ConfluenceRefresh` - Refresh the current page
- `:ConfluenceClearCache` - Clear the cache

### Examples

```vim
" Open a page by ID
:ConfluenceOpen 123456789

" Open a page by URL
:ConfluenceOpen https://your-instance.atlassian.net/wiki/spaces/SPACE/pages/123456789

" Search for pages
:ConfluenceSearch project documentation

" Refresh current page
:ConfluenceRefresh
```

## Development

### Building from Source

```bash
git clone https://github.com/m-bal/confluence-nvim.git
cd confluence-nvim
cargo build --release
```

### Running Tests

```bash
# Unit tests
cargo test

# With coverage
cargo tarpaulin --out Html

# Integration tests
cargo test --test '*'

# Clippy lints
cargo clippy -- -D warnings
```

### Project Structure

```
confluence-nvim/
├── src/
│   ├── api/           # Confluence REST API client
│   ├── cache/         # LRU caching layer
│   ├── config/        # Configuration management
│   └── renderer/      # Content rendering engine
├── lua/
│   └── confluence/    # Lua plugin interface
├── tests/             # Integration tests
└── doc/               # Neovim help documentation
```

## Roadmap

- [x] Basic project structure
- [x] Confluence API client
- [x] Content caching
- [ ] Rich terminal rendering
- [ ] Image support
- [ ] Comment system
- [ ] Navigation and link following
- [ ] Search functionality
- [ ] Edit support (future)

## Contributing

Contributions are welcome! This project follows the **Verification-Driven Development (VDD)** methodology.

See [CONTRIBUTING.md](CONTRIBUTING.md) and [VDD_BREAKDOWN.md](VDD_BREAKDOWN.md) for details.

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Acknowledgments

- Built with [nvim-oxi](https://github.com/noib3/nvim-oxi) for Neovim Rust bindings
- Inspired by various Neovim documentation plugins

---

**Status**: 🚧 Under Active Development (VDD Phase 1 - Foundation Complete)
