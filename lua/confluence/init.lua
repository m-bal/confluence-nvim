-- Confluence Neovim Plugin - Main module
local api = require('confluence.api')
local renderer = require('confluence.renderer')
local M = {}

--- Setup the plugin
---@param opts table Configuration options
--- Required: confluence_url (string), auth (table with type='token'|'pat', token=string, email=string?)
--- Optional: cache_enabled (boolean), cache_ttl (number), cache_size (number)
function M.setup(opts)
  api.setup(opts)

  -- Auto-load Telescope extension if Telescope is available
  local has_telescope, telescope = pcall(require, 'telescope')
  if has_telescope then
    telescope.load_extension('confluence')
  end

  vim.notify('Confluence plugin initialized', vim.log.levels.INFO)
end

--- Open a Confluence page in a new buffer
---@param page_id string Page ID to open
function M.open_page(page_id)
  api.fetch_page(page_id, function(err, page)
    if err then
      vim.notify('Failed to fetch page: ' .. err, vim.log.levels.ERROR)
      return
    end

    -- Debug: check what we received
    if not page then
      vim.notify('ERROR: Page data is nil', vim.log.levels.ERROR)
      return
    end

    if not page.title then
      vim.notify('ERROR: Page has no title', vim.log.levels.ERROR)
      return
    end

    -- Debug: log content size if available
    if page.body and page.body.storage and page.body.storage.value then
      vim.notify('DEBUG: Got content, length: ' .. #page.body.storage.value, vim.log.levels.INFO)
    else
      vim.notify('WARNING: Page has no body content', vim.log.levels.WARN)
    end

    -- Create new buffer
    local buf = vim.api.nvim_create_buf(false, true)

    -- Render page content and extract links
    local lines, links = renderer.render_page(page)

    -- Set buffer content
    vim.api.nvim_buf_set_lines(buf, 0, -1, false, lines)

    -- Set buffer options
    vim.api.nvim_buf_set_option(buf, 'buftype', 'nofile')
    vim.api.nvim_buf_set_option(buf, 'filetype', 'confluence')
    vim.api.nvim_buf_set_option(buf, 'modifiable', false)

    -- Set buffer name
    vim.api.nvim_buf_set_name(buf, '[Confluence] ' .. page.title)

    -- Store page metadata in buffer variables
    vim.api.nvim_buf_set_var(buf, 'confluence_page_id', page.id)
    vim.api.nvim_buf_set_var(buf, 'confluence_page_title', page.title)
    if page.space then
      vim.api.nvim_buf_set_var(buf, 'confluence_space_key', page.space.key or 'Unknown')
    end

    -- Set up link following if there are links
    if links and #links > 0 then
      local link_handler = require('confluence.links')
      link_handler.setup_buffer_links(buf, links)
      vim.notify(string.format('Page loaded: %s (%d links)', page.title, #links), vim.log.levels.INFO)
    else
      vim.notify('Page loaded: ' .. page.title, vim.log.levels.INFO)
    end

    -- Open in current window
    vim.api.nvim_set_current_buf(buf)
  end)
end

return M
