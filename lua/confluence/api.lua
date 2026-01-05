-- Confluence API client (Lua implementation for MVP)
local M = {}

local config = nil
local debug_mode = false

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

  -- Enable debug mode if requested
  if opts.debug then
    debug_mode = true
  end

  config = opts
end

--- Get configuration
function M.get_config()
  return config
end

--- Debug log helper
local function debug_log(message)
  if debug_mode then
    print('[confluence.nvim] ' .. message)
  end
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

    -- Base64 encode using command (works on all systems with base64 command)
    local b64 = vim.fn.system('printf "%s" "' .. credentials .. '" | base64 | tr -d "\n"')
    auth_header = 'Authorization: Basic ' .. b64
    debug_log('Using Basic auth with email: ' .. config.auth.email)
  else
    -- Bearer token (PAT)
    auth_header = 'Authorization: Bearer ' .. config.auth.token
    debug_log('Using Bearer token (PAT)')
  end

  debug_log('Request URL: ' .. url)

  -- Use curl to make request
  local cmd = {
    'curl',
    '-s',  -- Silent mode
    '-w', '\n%{http_code}',  -- Write HTTP status code at end
    '-H', auth_header,
    '-H', 'Accept: application/json',
    '-H', 'Content-Type: application/json',
    url,
  }

  local stdout_data = {}
  local stderr_data = {}

  vim.fn.jobstart(cmd, {
    stdout_buffered = true,
    stderr_buffered = true,
    on_stdout = function(_, data)
      if data then
        for _, line in ipairs(data) do
          if line ~= '' then
            table.insert(stdout_data, line)
          end
        end
      end
    end,
    on_stderr = function(_, data)
      if data then
        for _, line in ipairs(data) do
          if line ~= '' then
            table.insert(stderr_data, line)
          end
        end
      end
    end,
    on_exit = function(_, exit_code)
      -- Check for curl errors
      if exit_code ~= 0 then
        local err_msg = 'curl failed with exit code ' .. exit_code
        if #stderr_data > 0 then
          err_msg = err_msg .. ': ' .. table.concat(stderr_data, ' ')
        end
        debug_log('ERROR: ' .. err_msg)
        callback(err_msg, nil)
        return
      end

      -- Check for stderr output (warnings/errors)
      if #stderr_data > 0 then
        debug_log('STDERR: ' .. table.concat(stderr_data, ' '))
      end

      if #stdout_data == 0 then
        callback('Empty response from server', nil)
        return
      end

      -- Extract HTTP status code (last line)
      local http_code = table.remove(stdout_data)
      debug_log('HTTP Status: ' .. http_code)

      -- Check HTTP status
      if http_code ~= '200' then
        local response_body = table.concat(stdout_data, '\n')
        local err_msg = 'HTTP ' .. http_code
        if http_code == '401' then
          err_msg = 'Authentication failed (401). Check your API token and email.'
        elseif http_code == '403' then
          err_msg = 'Access forbidden (403). Check your permissions.'
        elseif http_code == '404' then
          err_msg = 'Resource not found (404). Check your Confluence URL.'
        end
        debug_log('ERROR: ' .. err_msg)
        debug_log('Response: ' .. response_body:sub(1, 500))
        callback(err_msg, nil)
        return
      end

      -- Parse JSON response
      local json_str = table.concat(stdout_data, '\n')
      debug_log('Response length: ' .. #json_str .. ' bytes')

      local ok, result = pcall(vim.fn.json_decode, json_str)
      if ok then
        debug_log('JSON parsed successfully')
        callback(nil, result)
      else
        debug_log('ERROR: Failed to parse JSON: ' .. tostring(result))
        debug_log('Raw response: ' .. json_str:sub(1, 500))
        callback('Failed to parse JSON response: ' .. tostring(result), nil)
      end
    end,
  })
end

--- List all Confluence spaces
---@param callback function Callback (error, spaces)
function M.list_spaces(callback)
  debug_log('Fetching spaces...')
  api_request('/rest/api/space', function(err, result)
    if err then
      callback(err, nil)
    else
      local spaces = result.results or {}
      debug_log('Found ' .. #spaces .. ' spaces')
      callback(nil, spaces)
    end
  end)
end

--- List pages in a space
---@param space_key string Space key
---@param callback function Callback (error, pages)
function M.list_pages(space_key, callback)
  debug_log('Fetching pages for space: ' .. space_key)
  api_request('/rest/api/space/' .. space_key .. '/content/page?expand=space', function(err, result)
    if err then
      callback(err, nil)
    else
      local pages = result.results or {}
      debug_log('Found ' .. #pages .. ' pages')
      callback(nil, pages)
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

  -- URL encode the CQL query
  local encoded_cql = vim.uri_encode(cql)

  debug_log('Searching for: ' .. query)
  api_request('/rest/api/content/search?cql=' .. encoded_cql, function(err, result)
    if err then
      callback(err, nil)
    else
      local results = result.results or {}
      debug_log('Found ' .. #results .. ' search results')
      callback(nil, results)
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

  debug_log('Fetching page: ' .. page_id)
  api_request('/rest/api/content/' .. page_id .. '?expand=body.storage,space', callback)
end

return M
