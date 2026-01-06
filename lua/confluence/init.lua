-- Confluence Neovim Plugin - Main module
local api = require('confluence.api')
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

    -- Create new buffer
    local buf = vim.api.nvim_create_buf(false, true)

    -- Simple rendering: just show the title and raw content for MVP
    local lines = {
      '# ' .. page.title,
      '',
      'Space: ' .. (page.space and page.space.name or 'Unknown'),
      'ID: ' .. page.id,
      '',
      '---',
      '',
    }

    -- Add content (simplified - just extract text from HTML)
    if page.body and page.body.storage and page.body.storage.value then
      local content = page.body.storage.value
      vim.notify('DEBUG: Got content, length: ' .. #content, vim.log.levels.INFO)

      -- Simple HTML tag stripping for MVP
      -- First, replace common tags with newlines for better formatting
      content = content:gsub('<br[^>]*>', '\n')
      content = content:gsub('</p>', '\n\n')
      content = content:gsub('</div>', '\n')
      content = content:gsub('</h[1-6]>', '\n\n')

      -- Strip remaining HTML tags
      content = content:gsub('<[^>]+>', '')

      -- Decode common HTML entities
      content = content:gsub('&nbsp;', ' ')
      content = content:gsub('&amp;', '&')
      content = content:gsub('&lt;', '<')
      content = content:gsub('&gt;', '>')
      content = content:gsub('&quot;', '"')

      -- Split into lines and add to buffer
      for line in content:gmatch('[^\r\n]+') do
        local trimmed = vim.trim(line)
        if trimmed ~= '' then
          table.insert(lines, trimmed)
        end
      end
    else
      table.insert(lines, '[No content available]')
      vim.notify('WARNING: Page has no body content', vim.log.levels.WARN)
    end

    -- Set buffer content
    vim.api.nvim_buf_set_lines(buf, 0, -1, false, lines)

    -- Set buffer options
    vim.api.nvim_buf_set_option(buf, 'buftype', 'nofile')
    vim.api.nvim_buf_set_option(buf, 'filetype', 'confluence')
    vim.api.nvim_buf_set_option(buf, 'modifiable', false)

    -- Set buffer name
    vim.api.nvim_buf_set_name(buf, '[Confluence] ' .. page.title)

    -- Open in current window
    vim.api.nvim_set_current_buf(buf)

    vim.notify('Page loaded: ' .. page.title, vim.log.levels.INFO)
  end)
end

return M
