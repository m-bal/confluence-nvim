-- Test helpers for Confluence plugin
local M = {}

-- Mock Confluence API responses
M.mock_api = {
  page_response = function(page_id, title, content)
    return vim.fn.json_encode({
      id = page_id,
      title = title,
      type = 'page',
      body = {
        storage = {
          value = content,
          representation = 'storage',
        },
      },
      space = {
        key = 'TEST',
        name = 'Test Space',
      },
      version = {
        number = 1,
        when = '2024-01-05T12:00:00.000Z',
      },
      _links = {
        webui = '/spaces/TEST/pages/' .. page_id,
      },
    })
  end,

  space_response = function(spaces)
    return vim.fn.json_encode({
      results = spaces,
      start = 0,
      limit = 25,
      size = #spaces,
    })
  end,

  search_response = function(results)
    return vim.fn.json_encode({
      results = results,
      start = 0,
      limit = 25,
      size = #results,
    })
  end,
}

-- Setup test environment
M.setup_test_env = function()
  -- Clear any existing configuration
  package.loaded['confluence'] = nil

  -- Initialize plugin with test config
  require('confluence').setup({
    confluence_url = 'http://localhost:8080',
    auth = {
      token = 'test-token-12345',
    },
    cache_enabled = false, -- Disable cache for tests
  })
end

-- Cleanup after tests
M.cleanup = function()
  -- Close all Confluence buffers
  for _, buf in ipairs(vim.api.nvim_list_bufs()) do
    if vim.api.nvim_buf_is_valid(buf) then
      local ok, ft = pcall(vim.api.nvim_buf_get_option, buf, 'filetype')
      if ok and (ft == 'confluence' or ft == 'confluence-browser') then
        vim.api.nvim_buf_delete(buf, { force = true })
      end
    end
  end

  -- Clear namespace
  local ns = vim.api.nvim_create_namespace('confluence')
  for _, buf in ipairs(vim.api.nvim_list_bufs()) do
    if vim.api.nvim_buf_is_valid(buf) then
      vim.api.nvim_buf_clear_namespace(buf, ns, 0, -1)
    end
  end
end

-- Read fixture file
M.read_fixture = function(name)
  local path = vim.fn.getcwd() .. '/tests/fixtures/' .. name .. '.html'
  local file = io.open(path, 'r')
  if not file then
    error('Fixture not found: ' .. path)
  end
  local content = file:read('*all')
  file:close()
  return content
end

-- Wait for condition with timeout
M.wait_for = function(condition, timeout)
  timeout = timeout or 5000
  local result = vim.wait(timeout, condition, 10)
  if not result then
    error('Timeout waiting for condition')
  end
end

-- Find extmark by line number
M.find_mark_by_line = function(marks, line)
  for _, mark in ipairs(marks) do
    if mark[2] == line then
      return mark[4] -- Return extmark details
    end
  end
  return nil
end

-- Find line matching pattern
M.find_line_matching = function(lines, pattern)
  for i, line in ipairs(lines) do
    if line:match(pattern) then
      return i - 1 -- Return 0-indexed line number
    end
  end
  return nil
end

-- Assert buffer has expected options
M.assert_buffer_options = function(buf, expected)
  for opt, value in pairs(expected) do
    local actual = vim.api.nvim_buf_get_option(buf, opt)
    assert.are.equal(value, actual, 'Buffer option ' .. opt .. ' mismatch')
  end
end

-- Assert buffer has expected content
M.assert_buffer_content = function(buf, expected_lines)
  local actual = vim.api.nvim_buf_get_lines(buf, 0, -1, false)
  assert.are.same(expected_lines, actual)
end

-- Create a test page with content
M.create_test_page = function(page_id, title, content)
  -- This would normally call the API, but for tests we mock it
  return {
    id = page_id,
    title = title,
    content = content,
  }
end

return M
