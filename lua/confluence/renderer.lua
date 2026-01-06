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

--- Create a bordered box around content
---@param content string Content to box
---@param title string Box title (e.g., "INFO", "WARNING", "CODE")
---@param width number Box width
---@param preserve_indent boolean Whether to preserve leading whitespace (for code)
---@return string Boxed content
local function create_box(content, title, width, preserve_indent)
  width = width or 70
  preserve_indent = preserve_indent or false
  local lines = vim.split(content, '\n')
  local result = {}

  -- Top border
  table.insert(result, '┌─ ' .. title .. ' ' .. string.rep('─', width - #title - 4) .. '┐')

  -- Content lines
  for _, line in ipairs(lines) do
    -- For code blocks, preserve indentation; for others, trim
    local display_line = preserve_indent and line or vim.trim(line)

    -- Skip empty lines only if not preserving indentation
    if preserve_indent or display_line ~= '' then
      local padding = width - vim.fn.strdisplaywidth(display_line) - 2
      if padding < 0 then padding = 0 end
      table.insert(result, '│ ' .. display_line .. string.rep(' ', padding) .. '│')
    end
  end

  -- Bottom border
  table.insert(result, '└' .. string.rep('─', width) .. '┘')

  return table.concat(result, '\n')
end

--- Extract and process Confluence macros
---@param html string HTML content
---@return string Processed HTML with macro replacements
local function process_macros(html)
  local content = html

  -- Info macro
  content = content:gsub('<ac:structured%-macro%s+ac:name="info"[^>]*>(.-)</ac:structured%-macro>', function(macro_content)
    local text = macro_content:gsub('<[^>]+>', ''):gsub('%s+', ' ')
    text = vim.trim(text)
    return '\n' .. create_box(text, 'ℹ INFO', 70) .. '\n'
  end)

  -- Warning macro
  content = content:gsub('<ac:structured%-macro%s+ac:name="warning"[^>]*>(.-)</ac:structured%-macro>', function(macro_content)
    local text = macro_content:gsub('<[^>]+>', ''):gsub('%s+', ' ')
    text = vim.trim(text)
    return '\n' .. create_box(text, '⚠ WARNING', 70) .. '\n'
  end)

  -- Note macro
  content = content:gsub('<ac:structured%-macro%s+ac:name="note"[^>]*>(.-)</ac:structured%-macro>', function(macro_content)
    local text = macro_content:gsub('<[^>]+>', ''):gsub('%s+', ' ')
    text = vim.trim(text)
    return '\n' .. create_box(text, '📝 NOTE', 70) .. '\n'
  end)

  -- Code macro - preserve indentation
  content = content:gsub('<ac:structured%-macro%s+ac:name="code"[^>]*>(.-)</ac:structured%-macro>', function(macro_content)
    -- Extract language parameter
    local lang = macro_content:match('<ac:parameter%s+ac:name="language">([^<]+)</ac:parameter>') or 'text'
    -- Extract code content
    local code = macro_content:match('<ac:plain%-text%-body>(.-)</ac:plain%-text%-body>')
    if not code then
      code = macro_content:gsub('<[^>]+>', '')
    end
    -- Trim only trailing/leading newlines, not indentation
    code = code:gsub('^%s*\n', ''):gsub('\n%s*$', '')
    return '\n' .. create_box(code, lang:upper(), 70, true) .. '\n'  -- preserve_indent=true
  end)

  return content
end

--- Render table with box-drawing characters
---@param table_html string HTML table content
---@return string Formatted table
local function render_table(table_html)
  local rows = {}
  local is_header = false

  -- Extract rows
  for row in table_html:gmatch('<tr[^>]*>(.-)</tr>') do
    local cells = {}
    local row_is_header = row:match('<th')

    -- Extract cells (th or td)
    for cell in row:gmatch('<t[hd][^>]*>(.-)</t[hd]>') do
      local text = cell:gsub('<[^>]+>', ''):gsub('%s+', ' ')
      text = vim.trim(text)
      table.insert(cells, text)
    end

    if #cells > 0 then
      table.insert(rows, {cells = cells, is_header = row_is_header})
    end
  end

  if #rows == 0 then
    return table_html
  end

  -- Calculate column widths
  local col_count = #rows[1].cells
  local col_widths = {}
  for i = 1, col_count do
    col_widths[i] = 0
  end

  for _, row in ipairs(rows) do
    for i, cell in ipairs(row.cells) do
      col_widths[i] = math.max(col_widths[i], vim.fn.strdisplaywidth(cell))
    end
  end

  -- Render table
  local result = {}
  local separator = '├' .. table.concat(vim.tbl_map(function(w) return string.rep('─', w + 2) end, col_widths), '┼') .. '┤'
  local top = '┌' .. table.concat(vim.tbl_map(function(w) return string.rep('─', w + 2) end, col_widths), '┬') .. '┐'
  local bottom = '└' .. table.concat(vim.tbl_map(function(w) return string.rep('─', w + 2) end, col_widths), '┴') .. '┘'

  table.insert(result, top)

  for idx, row in ipairs(rows) do
    local line = '│'
    for i, cell in ipairs(row.cells) do
      local padding = col_widths[i] - vim.fn.strdisplaywidth(cell)
      line = line .. ' ' .. cell .. string.rep(' ', padding) .. ' │'
    end
    table.insert(result, line)

    -- Add separator after header
    if row.is_header then
      table.insert(result, separator)
    end
  end

  table.insert(result, bottom)

  return '\n' .. table.concat(result, '\n') .. '\n'
end

--- Process HTML content into plain text with formatting
---@param html string HTML content from Confluence
---@return string Processed plain text
function M.html_to_text(html)
  if not html or html == '' then
    return ''
  end

  local content = html

  -- Process Confluence macros FIRST (before stripping HTML)
  content = process_macros(content)

  -- Process tables with box-drawing characters
  content = content:gsub('<table[^>]*>(.-)</table>', render_table)

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

  -- Paragraphs and divs
  content = content:gsub('</p>', '\n')
  content = content:gsub('</div>', '\n')

  -- Line breaks
  content = content:gsub('<br[^>]*/>', '\n')
  content = content:gsub('<br[^>]*>', '\n')

  -- Convert inline code tags to backticks BEFORE stripping HTML
  content = content:gsub('<code[^>]*>(.-)</code>', '`%1`')
  content = content:gsub('<tt[^>]*>(.-)</tt>', '`%1`')
  content = content:gsub('<kbd[^>]*>(.-)</kbd>', '`%1`')

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
