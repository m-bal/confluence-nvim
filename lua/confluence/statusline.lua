-- Status line integration for Confluence buffers
local M = {}

--- Get status line component for current buffer
---@return string Status line text
function M.get_statusline()
  local buf = vim.api.nvim_get_current_buf()

  -- Check if this is a Confluence buffer
  local ok_id, page_id = pcall(vim.api.nvim_buf_get_var, buf, 'confluence_page_id')
  if not ok_id then
    return ''
  end

  local ok_title, title = pcall(vim.api.nvim_buf_get_var, buf, 'confluence_page_title')
  local ok_space, space = pcall(vim.api.nvim_buf_get_var, buf, 'confluence_space_key')

  if ok_title and ok_space then
    return string.format('🏢 [%s] %s', space, title)
  elseif ok_title then
    return string.format('📄 %s', title)
  else
    return '🏢 Confluence'
  end
end

--- Show breadcrumbs for current page
function M.show_breadcrumbs()
  local buf = vim.api.nvim_get_current_buf()

  local ok_title, title = pcall(vim.api.nvim_buf_get_var, buf, 'confluence_page_title')
  local ok_space, space = pcall(vim.api.nvim_buf_get_var, buf, 'confluence_space_key')

  if not ok_title then
    vim.notify('Not a Confluence buffer', vim.log.levels.WARN)
    return
  end

  -- Simple breadcrumb for now (space > page)
  local breadcrumb = string.format('🏠 %s > 📄 %s', space or 'Unknown', title)

  vim.notify(breadcrumb, vim.log.levels.INFO)
end

return M
