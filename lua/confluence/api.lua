-- Confluence API client (Lua implementation for MVP)
local M = {}

local config = nil

--- Setup API client with configuration
---@param opts table Configuration options
function M.setup(opts)
  -- Validate required fields
  if not opts.confluence_url or opts.confluence_url == '' then
    error('confluence_url is required')
  end

  if not opts.auth or not opts.auth.token or opts.auth.token == '' then
    error('auth.token is required')
  end

  -- Validate HTTPS (security requirement from adversarial review)
  if not vim.startswith(opts.confluence_url, 'https://') then
    error('Only HTTPS URLs are allowed for security')
  end

  config = opts
end

--- Get configuration
function M.get_config()
  return config
end

--- Make authenticated API request
---@param endpoint string API endpoint path
---@param callback function Callback function (error, result)
local function api_request(endpoint, callback)
  if not config then
    callback('Plugin not configured. Call require("confluence").setup() first', nil)
    return
  end

  local url = config.confluence_url .. endpoint
  local auth_header

  if config.auth.type == 'token' and config.auth.email then
    -- Basic auth with email and API token
    local credentials = config.auth.email .. ':' .. config.auth.token
    local encoded = vim.fn.system('echo -n "' .. credentials .. '" | base64'):gsub('\n', '')
    auth_header = 'Authorization: Basic ' .. encoded
  else
    -- Bearer token (PAT)
    auth_header = 'Authorization: Bearer ' .. config.auth.token
  end

  -- Use curl to make request
  local cmd = {
    'curl',
    '-s',
    '-H',
    auth_header,
    '-H',
    'Accept: application/json',
    url,
  }

  vim.fn.jobstart(cmd, {
    stdout_buffered = true,
    on_stdout = function(_, data)
      if data and #data > 0 then
        local json_str = table.concat(data, '\n')
        local ok, result = pcall(vim.fn.json_decode, json_str)
        if ok then
          callback(nil, result)
        else
          callback('Failed to parse JSON response: ' .. tostring(result), nil)
        end
      end
    end,
    on_stderr = function(_, data)
      if data and #data > 0 and data[1] ~= '' then
        callback('API error: ' .. table.concat(data, '\n'), nil)
      end
    end,
    on_exit = function(_, exit_code)
      if exit_code ~= 0 then
        callback('Request failed with exit code ' .. exit_code, nil)
      end
    end,
  })
end

--- List all Confluence spaces
---@param callback function Callback (error, spaces)
function M.list_spaces(callback)
  api_request('/rest/api/space', function(err, result)
    if err then
      callback(err, nil)
    else
      callback(nil, result.results or {})
    end
  end)
end

--- List pages in a space
---@param space_key string Space key
---@param callback function Callback (error, pages)
function M.list_pages(space_key, callback)
  api_request('/rest/api/space/' .. space_key .. '/content/page?expand=space', function(err, result)
    if err then
      callback(err, nil)
    else
      callback(nil, result.results or {})
    end
  end)
end

--- Search Confluence content
---@param query string Search query
---@param callback function Callback (error, results)
function M.search(query, callback)
  -- Sanitize query (per adversarial review C-03)
  local sanitized = query:gsub('\\', '\\\\'):gsub('"', '\\"'):gsub('%(', '\\('):gsub('%)', '\\)')
  local cql = 'text ~ "' .. sanitized .. '"'
  local encoded_cql = vim.fn.shellescape(cql)

  api_request('/rest/api/content/search?cql=' .. encoded_cql, function(err, result)
    if err then
      callback(err, nil)
    else
      callback(nil, result.results or {})
    end
  end)
end

--- Fetch a page by ID
---@param page_id string Page ID
---@param callback function Callback (error, page)
function M.fetch_page(page_id, callback)
  -- Validate page ID (per adversarial review C-02)
  if not page_id:match('^[%w%-_]+$') then
    callback('Invalid page ID: contains unsafe characters', nil)
    return
  end

  api_request('/rest/api/content/' .. page_id .. '?expand=body.storage,space', callback)
end

return M
