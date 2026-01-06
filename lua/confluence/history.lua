-- Navigation history for Confluence pages (tag stack style)
local M = {}

-- History stack
-- Format: {page_id, title, buffer}
local history_stack = {}
local history_position = 0

--- Push a page onto the history stack
---@param page_id string Page ID
---@param title string Page title
function M.push(page_id, title)
  -- If we're not at the end of the stack, truncate everything after current position
  if history_position < #history_stack then
    for i = #history_stack, history_position + 1, -1 do
      table.remove(history_stack, i)
    end
  end

  -- Add new entry
  table.insert(history_stack, {
    page_id = page_id,
    title = title,
    timestamp = os.time()
  })

  history_position = #history_stack
end

--- Go back in history
function M.go_back()
  if history_position <= 1 then
    vim.notify('At oldest page in history', vim.log.levels.INFO)
    return false
  end

  history_position = history_position - 1
  local entry = history_stack[history_position]

  vim.notify(string.format('← %s', entry.title), vim.log.levels.INFO)
  require('confluence').open_page(entry.page_id, {skip_history = true})

  return true
end

--- Go forward in history
function M.go_forward()
  if history_position >= #history_stack then
    vim.notify('At newest page in history', vim.log.levels.INFO)
    return false
  end

  history_position = history_position + 1
  local entry = history_stack[history_position]

  vim.notify(string.format('→ %s', entry.title), vim.log.levels.INFO)
  require('confluence').open_page(entry.page_id, {skip_history = true})

  return true
end

--- Show history list
function M.show_history()
  if #history_stack == 0 then
    vim.notify('No history', vim.log.levels.INFO)
    return
  end

  local lines = {'# Navigation History', ''}
  for i, entry in ipairs(history_stack) do
    local marker = (i == history_position) and '► ' or '  '
    table.insert(lines, string.format('%s%d. %s', marker, i, entry.title))
  end

  -- Create a new buffer to show history
  local buf = vim.api.nvim_create_buf(false, true)
  vim.api.nvim_buf_set_lines(buf, 0, -1, false, lines)
  vim.api.nvim_buf_set_option(buf, 'buftype', 'nofile')
  vim.api.nvim_buf_set_option(buf, 'modifiable', false)
  vim.api.nvim_buf_set_name(buf, '[Confluence History]')

  -- Open in a split
  vim.cmd('split')
  vim.api.nvim_set_current_buf(buf)
end

--- Get current history position
function M.get_position()
  return history_position, #history_stack
end

--- Clear history
function M.clear()
  history_stack = {}
  history_position = 0
end

return M
