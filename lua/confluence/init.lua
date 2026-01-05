-- Confluence Neovim Plugin
-- Main Lua entry point

local M = {}

-- Default configuration
local default_config = {
  confluence_url = '',
  auth = {
    email = nil,
    token = '',
  },
  cache_enabled = true,
  cache_ttl = 900, -- 15 minutes
  cache_size = 50,
  render_options = {
    enable_syntax_highlighting = true,
    enable_images = true,
    max_image_width = 80,
  },
}

-- Plugin state
local state = {
  initialized = false,
  config = vim.deepcopy(default_config),
  rust_loaded = false,
}

--- Setup the Confluence plugin
---@param opts table User configuration
function M.setup(opts)
  -- Merge user config with defaults
  state.config = vim.tbl_deep_extend('force', default_config, opts or {})

  -- Validate required fields
  if state.config.confluence_url == '' then
    vim.notify('Confluence: confluence_url is required', vim.log.levels.ERROR)
    return
  end

  if state.config.auth.token == '' then
    vim.notify('Confluence: auth.token is required', vim.log.levels.ERROR)
    return
  end

  -- Load Rust library
  local ok, rust_lib = pcall(require, 'confluence_nvim')
  if not ok then
    vim.notify('Confluence: Failed to load Rust library: ' .. tostring(rust_lib), vim.log.levels.ERROR)
    return
  end

  -- Initialize Rust plugin
  local setup_ok, err = pcall(rust_lib.setup, state.config)
  if not setup_ok then
    vim.notify('Confluence: Setup failed: ' .. tostring(err), vim.log.levels.ERROR)
    return
  end

  state.rust_loaded = true
  state.initialized = true

  -- Register commands
  M.register_commands()

  -- Set up highlight groups
  M.setup_highlights()

  vim.notify('Confluence plugin initialized', vim.log.levels.INFO)
end

--- Register Neovim commands
function M.register_commands()
  -- Main entry point - opens space browser
  vim.api.nvim_create_user_command('Confluence', function()
    M.browse_spaces()
  end, {
    desc = 'Open Confluence space browser',
  })

  vim.api.nvim_create_user_command('ConfluenceOpen', function(args)
    M.open_page(args.args)
  end, {
    nargs = 1,
    desc = 'Open a Confluence page by ID or URL',
  })

  vim.api.nvim_create_user_command('ConfluenceSpace', function(args)
    M.browse_pages(args.args)
  end, {
    nargs = '?',
    desc = 'Browse pages in a space',
  })

  vim.api.nvim_create_user_command('ConfluenceSearch', function(args)
    M.search(args.args)
  end, {
    nargs = '*',
    desc = 'Search Confluence pages',
  })

  vim.api.nvim_create_user_command('ConfluenceRecent', function()
    M.recent_pages()
  end, {
    desc = 'Show recent Confluence pages',
  })

  vim.api.nvim_create_user_command('ConfluenceRefresh', function()
    M.refresh_current_page()
  end, {
    desc = 'Refresh the current Confluence page',
  })

  vim.api.nvim_create_user_command('ConfluenceClearCache', function()
    M.clear_cache()
  end, {
    desc = 'Clear the Confluence cache',
  })
end

--- Set up Confluence-specific highlight groups
function M.setup_highlights()
  local highlights = {
    -- Macro panels
    ConfluenceInfoPanel = { fg = '#0052CC', bg = '#DEEBFF', bold = true },
    ConfluenceWarningPanel = { fg = '#FF8B00', bg = '#FFFAE6', bold = true },
    ConfluenceErrorPanel = { fg = '#DE350B', bg = '#FFEBE6', bold = true },
    ConfluenceSuccessPanel = { fg = '#006644', bg = '#E3FCEF', bold = true },
    ConfluenceNotePanel = { fg = '#6554C0', bg = '#EAE6FF', bold = true },

    -- Code blocks
    ConfluenceCodeBlock = { fg = '#172B4D', bg = '#F4F5F7' },

    -- Tables
    ConfluenceTableHeader = { fg = '#253858', bg = '#DFE1E6', bold = true },

    -- Links
    ConfluenceLink = { fg = '#0052CC', underline = true },

    -- Comments
    ConfluenceComment = { fg = '#6B778C', italic = true },

    -- Status indicators
    ConfluenceStatusGreen = { fg = '#00875A' },
    ConfluenceStatusYellow = { fg = '#FF8B00' },
    ConfluenceStatusRed = { fg = '#DE350B' },
    ConfluenceStatusGray = { fg = '#6B778C' },
    ConfluenceStatusBlue = { fg = '#0052CC' },
  }

  for group, opts in pairs(highlights) do
    vim.api.nvim_set_hl(0, group, opts)
  end
end

--- Browse Confluence spaces (telescope picker)
function M.browse_spaces()
  if not state.initialized then
    vim.notify('Confluence: Plugin not initialized. Call setup() first.', vim.log.levels.ERROR)
    return
  end

  local ok, telescope = pcall(require, 'confluence.telescope')
  if not ok then
    vim.notify('Confluence: Telescope integration not available', vim.log.levels.ERROR)
    return
  end

  telescope.spaces()
end

--- Browse pages in a space
---@param space_key string|nil Space key (optional)
function M.browse_pages(space_key)
  if not state.initialized then
    vim.notify('Confluence: Plugin not initialized. Call setup() first.', vim.log.levels.ERROR)
    return
  end

  local ok, telescope = pcall(require, 'confluence.telescope')
  if not ok then
    vim.notify('Confluence: Telescope integration not available', vim.log.levels.ERROR)
    return
  end

  telescope.pages({ space_key = space_key })
end

--- Open a Confluence page
---@param page_id_or_url string Page ID or URL
function M.open_page(page_id_or_url)
  if not state.initialized then
    vim.notify('Confluence: Plugin not initialized. Call setup() first.', vim.log.levels.ERROR)
    return
  end

  -- Extract page ID from URL if needed
  local page_id = page_id_or_url:match('%d+$') or page_id_or_url

  -- TODO: Call Rust API to fetch page
  -- TODO: Render page in buffer

  vim.notify('Opening Confluence page: ' .. page_id, vim.log.levels.INFO)
end

--- Search Confluence pages
---@param query string|nil Search query
function M.search(query)
  if not state.initialized then
    vim.notify('Confluence: Plugin not initialized. Call setup() first.', vim.log.levels.ERROR)
    return
  end

  local ok, telescope = pcall(require, 'confluence.telescope')
  if not ok then
    vim.notify('Confluence: Telescope integration not available', vim.log.levels.ERROR)
    return
  end

  telescope.search({ query = query })
end

--- Show recent pages
function M.recent_pages()
  if not state.initialized then
    vim.notify('Confluence: Plugin not initialized. Call setup() first.', vim.log.levels.ERROR)
    return
  end

  local ok, telescope = pcall(require, 'confluence.telescope')
  if not ok then
    vim.notify('Confluence: Telescope integration not available', vim.log.levels.ERROR)
    return
  end

  telescope.recent()
end

--- Refresh the current Confluence page
function M.refresh_current_page()
  if not state.initialized then
    vim.notify('Confluence: Plugin not initialized. Call setup() first.', vim.log.levels.ERROR)
    return
  end

  local bufnr = vim.api.nvim_get_current_buf()
  local page_id = vim.b[bufnr].confluence_page_id

  if not page_id then
    vim.notify('Confluence: Current buffer is not a Confluence page', vim.log.levels.WARN)
    return
  end

  -- TODO: Implement refresh logic
  vim.notify('Refreshing Confluence page: ' .. page_id, vim.log.levels.INFO)
end

--- Clear the Confluence cache
function M.clear_cache()
  if not state.initialized then
    vim.notify('Confluence: Plugin not initialized. Call setup() first.', vim.log.levels.ERROR)
    return
  end

  -- TODO: Call Rust cache clear function
  vim.notify('Confluence cache cleared', vim.log.levels.INFO)
end

--- Get current configuration
---@return table
function M.get_config()
  return vim.deepcopy(state.config)
end

--- Check if plugin is initialized
---@return boolean
function M.is_initialized()
  return state.initialized
end

return M
