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

  -- Convert headings with visual markers
  content = content:gsub('<h1[^>]*>([^<]+)</h1>', '\n## %1\n')
  content = content:gsub('<h2[^>]*>([^<]+)</h2>', '\n### %1\n')
  content = content:gsub('<h3[^>]*>([^<]+)</h3>', '\n#### %1\n')
  content = content:gsub('<h4[^>]*>([^<]+)</h4>', '\n##### %1\n')
  content = content:gsub('<h5[^>]*>([^<]+)</h5>', '\n###### %1\n')
  content = content:gsub('<h6[^>]*>([^<]+)</h6>', '\n###### %1\n')

  -- Convert ordered lists with numbers (simple approach - all items get numbers)
  local ol_counter = 0
  content = content:gsub('<ol[^>]*>', function()
    ol_counter = 0
    return '\n'
  end)
  content = content:gsub('</ol>', '\n')

  -- Convert unordered lists with bullet points
  content = content:gsub('<ul[^>]*>', '\n')
  content = content:gsub('</ul>', '\n')

  -- List items - check context for ordered vs unordered
  -- Simple approach: use bullets for all (proper numbering would need state tracking)
  content = content:gsub('<li[^>]*>', '  • ')
  content = content:gsub('</li>', '\n')

  -- Table cells to tab-separated
  content = content:gsub('</td>', '\t')
  content = content:gsub('</th>', '\t')
  content = content:gsub('</tr>', '\n')

  -- Paragraphs and divs
  content = content:gsub('</p>', '\n')
  content = content:gsub('</div>', '\n')

  -- Line breaks
  content = content:gsub('<br[^>]*/>', '\n')
  content = content:gsub('<br[^>]*>', '\n')

  -- Strip remaining HTML tags
  content = content:gsub('<[^>]+>', '')

  -- Decode HTML entities
  content = decode_entities(content)

  -- Clean up excessive whitespace
  -- Reduce multiple blank lines to at most 2
  content = content:gsub('\n\n\n+', '\n\n')

  -- Trim leading/trailing whitespace on each line
  local lines = vim.split(content, '\n')
  for i, line in ipairs(lines) do
    lines[i] = vim.trim(line)
  end
  content = table.concat(lines, '\n')

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
