-- Link handling and navigation for Confluence pages
local M = {}

-- Store links found in the current page
-- Format: { line_num = { {text = "Link Text", page_id = "123456", col_start = 10, col_end = 20}, ... } }
local page_links = {}

--- Extract page ID from Confluence URL
---@param url string URL to parse
---@return string|nil Page ID or nil
local function extract_page_id(url)
  -- Match /pages/123456 or /pages/123456/Title
  local id = url:match('/pages/(%d+)')
  if id then return id end

  -- Match other Confluence URL patterns
  id = url:match('pageId=(%d+)')
  return id
end

--- Parse links from HTML and store their positions
---@param html string HTML content
---@param base_url string Base Confluence URL
---@return table Processed HTML with link markers
function M.extract_links(html, base_url)
  local links = {}
  local link_count = 0

  -- Process and extract all links
  local processed = html:gsub('<a%s+href="([^"]+)"[^>]*>(.-)</a>', function(href, text)
    -- Check if it's a Confluence page link
    local page_id = extract_page_id(href)

    if page_id then
      link_count = link_count + 1
      table.insert(links, {
        text = text,
        page_id = page_id,
        href = href,
        index = link_count
      })
      -- Return text with indicator and hidden marker
      return text .. ' →[' .. link_count .. ']'
    else
      -- External link
      return text
    end
  end)

  return processed, links
end

--- Set up links for a buffer
---@param bufnr number Buffer number
---@param links table Array of link metadata
function M.setup_buffer_links(bufnr, links)
  -- Store links globally indexed by buffer
  if not M.buffer_links then
    M.buffer_links = {}
  end

  M.buffer_links[bufnr] = links

  -- Set up keymap for link following
  vim.api.nvim_buf_set_keymap(bufnr, 'n', 'gx', '', {
    callback = function()
      M.follow_link_under_cursor(bufnr)
    end,
    noremap = true,
    silent = true,
    desc = 'Follow Confluence link under cursor'
  })

  vim.api.nvim_buf_set_keymap(bufnr, 'n', '<C-]>', '', {
    callback = function()
      M.follow_link_under_cursor(bufnr)
    end,
    noremap = true,
    silent = true,
    desc = 'Follow Confluence link (tag-style)'
  })
end

--- Find and follow link under cursor
---@param bufnr number Buffer number
function M.follow_link_under_cursor(bufnr)
  local links = M.buffer_links and M.buffer_links[bufnr]
  if not links or #links == 0 then
    vim.notify('No links found in this page', vim.log.levels.INFO)
    return
  end

  -- Get current line
  local cursor = vim.api.nvim_win_get_cursor(0)
  local line_num = cursor[1]
  local line = vim.api.nvim_buf_get_lines(bufnr, line_num - 1, line_num, false)[1]

  if not line then
    vim.notify('No line under cursor', vim.log.levels.WARN)
    return
  end

  -- Find link markers in the line: →[N]
  local link_index = line:match('→%[(%d+)%]')

  if link_index then
    link_index = tonumber(link_index)
    local link = links[link_index]

    if link and link.page_id then
      vim.notify('Opening page: ' .. link.text, vim.log.levels.INFO)
      -- Open the linked page
      require('confluence').open_page(link.page_id)
    else
      vim.notify('Link not found', vim.log.levels.WARN)
    end
  else
    -- Show available links
    vim.notify(string.format('No link under cursor. Page has %d links. Use → to find them.', #links), vim.log.levels.INFO)
  end
end

return M
