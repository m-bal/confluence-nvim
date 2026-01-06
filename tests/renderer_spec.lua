-- Unit tests for confluence.renderer
local renderer = require('confluence.renderer')

describe('confluence.renderer', function()
  describe('html_to_text', function()
    it('should handle empty or nil input', function()
      assert.equals('', renderer.html_to_text(nil))
      assert.equals('', renderer.html_to_text(''))
    end)

    it('should decode HTML entities', function()
      local input = 'Test &nbsp; &amp; &lt; &gt; &quot; &#39; text'
      local output = renderer.html_to_text(input)

      assert.has_match(' ', output)  -- &nbsp;
      assert.has_match('&', output)  -- &amp;
      assert.has_match('<', output)  -- &lt;
      assert.has_match('>', output)  -- &gt;
      assert.has_match('"', output)  -- &quot;
      assert.has_match("'", output)  -- &#39;
    end)

    it('should convert br tags to newlines', function()
      local input = 'Line 1<br>Line 2<br/>Line 3<br />Line 4'
      local output = renderer.html_to_text(input)

      -- Should contain newlines
      assert.has_match('\n', output)
    end)

    it('should convert p tags to double newlines', function()
      local input = '<p>Paragraph 1</p><p>Paragraph 2</p>'
      local output = renderer.html_to_text(input)

      -- Should contain double newlines for paragraph separation
      assert.has_match('\n\n', output)
      assert.has_match('Paragraph 1', output)
      assert.has_match('Paragraph 2', output)
    end)

    it('should convert heading tags to newlines', function()
      local input = '<h1>Heading 1</h1><h2>Heading 2</h2><h3>Heading 3</h3>'
      local output = renderer.html_to_text(input)

      assert.has_match('Heading 1', output)
      assert.has_match('Heading 2', output)
      assert.has_match('Heading 3', output)
      assert.has_match('\n\n', output)
    end)

    it('should convert li tags to newlines', function()
      local input = '<ul><li>Item 1</li><li>Item 2</li></ul>'
      local output = renderer.html_to_text(input)

      assert.has_match('Item 1', output)
      assert.has_match('Item 2', output)
      assert.has_match('\n', output)
    end)

    it('should strip all HTML tags', function()
      local input = '<strong>Bold</strong> <em>Italic</em> <a href="url">Link</a>'
      local output = renderer.html_to_text(input)

      -- Should not contain any tags
      assert.is_false(output:match('<[^>]+>'))
      -- Should contain the text content
      assert.has_match('Bold', output)
      assert.has_match('Italic', output)
      assert.has_match('Link', output)
    end)

    it('should handle complex nested HTML', function()
      local input = [[
        <div>
          <h1>Title</h1>
          <p>First paragraph with <strong>bold</strong> and <em>italic</em> text.</p>
          <p>Second paragraph with a <a href="http://example.com">link</a>.</p>
          <ul>
            <li>Item 1</li>
            <li>Item 2</li>
          </ul>
        </div>
      ]]
      local output = renderer.html_to_text(input)

      -- Should contain all text content
      assert.has_match('Title', output)
      assert.has_match('First paragraph', output)
      assert.has_match('bold', output)
      assert.has_match('italic', output)
      assert.has_match('Second paragraph', output)
      assert.has_match('link', output)
      assert.has_match('Item 1', output)
      assert.has_match('Item 2', output)

      -- Should not contain HTML tags
      assert.is_false(output:match('<[^>]+>'))
    end)

    it('should handle Confluence-specific HTML', function()
      local input = [[
        <ac:structured-macro ac:name="info">
          <ac:rich-text-body>
            <p>This is an info box</p>
          </ac:rich-text-body>
        </ac:structured-macro>
      ]]
      local output = renderer.html_to_text(input)

      assert.has_match('This is an info box', output)
      assert.is_false(output:match('<[^>]+>'))
    end)
  end)

  describe('render_page', function()
    it('should render page with all metadata', function()
      local page = {
        id = '123456',
        title = 'Test Page',
        space = {
          name = 'Test Space'
        },
        body = {
          storage = {
            value = '<p>Test content</p>'
          }
        }
      }

      local lines = renderer.render_page(page)

      -- Check header
      assert.equals('# Test Page', lines[1])

      -- Find metadata lines
      local has_space = false
      local has_id = false
      for _, line in ipairs(lines) do
        if line:match('Space: Test Space') then
          has_space = true
        end
        if line:match('ID: 123456') then
          has_id = true
        end
      end

      assert.is_true(has_space)
      assert.is_true(has_id)

      -- Check content
      local has_content = false
      for _, line in ipairs(lines) do
        if line:match('Test content') then
          has_content = true
        end
      end
      assert.is_true(has_content)
    end)

    it('should handle page with no space info', function()
      local page = {
        id = '123456',
        title = 'Test Page',
        body = {
          storage = {
            value = '<p>Content</p>'
          }
        }
      }

      local lines = renderer.render_page(page)

      local has_unknown_space = false
      for _, line in ipairs(lines) do
        if line:match('Space: Unknown') then
          has_unknown_space = true
        end
      end

      assert.is_true(has_unknown_space)
    end)

    it('should handle page with no content', function()
      local page = {
        id = '123456',
        title = 'Empty Page',
        space = {
          name = 'Test Space'
        }
      }

      local lines = renderer.render_page(page)

      -- Should have the no content message
      local has_no_content = false
      for _, line in ipairs(lines) do
        if line:match('%[No content available%]') then
          has_no_content = true
        end
      end

      assert.is_true(has_no_content)
    end)

    it('should handle page with empty content', function()
      local page = {
        id = '123456',
        title = 'Empty Content Page',
        space = {
          name = 'Test Space'
        },
        body = {
          storage = {
            value = ''
          }
        }
      }

      local lines = renderer.render_page(page)

      -- Should have basic structure but minimal content
      assert.is_true(#lines > 0)
      assert.equals('# Empty Content Page', lines[1])
    end)

    it('should filter out empty lines', function()
      local page = {
        id = '123456',
        title = 'Test Page',
        space = {
          name = 'Test Space'
        },
        body = {
          storage = {
            value = '<p>Line 1</p><p></p><p>   </p><p>Line 2</p>'
          }
        }
      }

      local lines = renderer.render_page(page)

      -- Count content lines (excluding header/metadata)
      local content_lines = {}
      local in_content = false
      for _, line in ipairs(lines) do
        if line == '---' then
          in_content = true
        elseif in_content and line ~= '' then
          table.insert(content_lines, line)
        end
      end

      -- Should have filtered out empty paragraphs
      -- We should have Line 1 and Line 2, but not the empty ones
      local has_line1 = false
      local has_line2 = false
      for _, line in ipairs(content_lines) do
        if line:match('Line 1') then has_line1 = true end
        if line:match('Line 2') then has_line2 = true end
      end

      assert.is_true(has_line1)
      assert.is_true(has_line2)
    end)
  end)
end)
