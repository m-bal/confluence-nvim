-- Confluence content renderer
local M = {}

--- Decode HTML entities
---@param text string Text containing HTML entities
---@return string Decoded text
local function decode_entities(text)
  local result = text
  result = result:gsub('&nbsp;', ' ')
  result = result:gsub('&amp;', '&')
  result = result:gsub('&lt;', '<')
  result = result:gsub('&gt;', '>')
  result = result:gsub('&quot;', '"')
  result = result:gsub('&#39;', "'")
  return result
end

--- Process HTML content into plain text with formatting
---@param html string HTML content from Confluence
---@return string Processed plain text
function M.html_to_text(html)
  if not html or html == '' then
    return ''
  end

  local content = html

  -- Replace structural tags with newlines for better formatting
  content = content:gsub('<br[^>]*>', '\n')
  content = content:gsub('<br[^>]*/>', '\n')
  content = content:gsub('</p>', '\n\n')
  content = content:gsub('</div>', '\n')
  content = content:gsub('</h[1-6]>', '\n\n')
  content = content:gsub('</li>', '\n')

  -- Strip remaining HTML tags
  content = content:gsub('<[^>]+>', '')

  -- Decode HTML entities
  content = decode_entities(content)

  return content
end

--- Render a Confluence page to buffer lines
---@param page table Page data from Confluence API
---@return table Array of lines for buffer
function M.render_page(page)
  local lines = {
    '# ' .. page.title,
    '',
    'Space: ' .. (page.space and page.space.name or 'Unknown'),
    'ID: ' .. page.id,
    '',
    '---',
    '',
  }

  -- Add content if available
  if page.body and page.body.storage and page.body.storage.value then
    local content = M.html_to_text(page.body.storage.value)

    -- Split into lines and filter empty ones
    for line in content:gmatch('[^\r\n]+') do
      local trimmed = vim.trim(line)
      if trimmed ~= '' then
        table.insert(lines, trimmed)
      end
    end
  else
    table.insert(lines, '[No content available]')
  end

  return lines
end

return M
