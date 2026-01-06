-- VDD Test: Inline code and code tick handling
local renderer = require('confluence.renderer')

describe('confluence inline code (VDD)', function()
  it('should render inline code tags with backticks', function()
    local confluence_html = '<p>Use the <code>npm install</code> command to install dependencies.</p>'

    local output = renderer.html_to_text(confluence_html)

    print("\n=== Inline Code Output ===")
    print(output)
    print("=== End Output ===\n")

    -- Should show inline code with backticks
    assert.has_match('`npm install`', output)
    assert.has_match('Use the', output)
    assert.has_match('command to install', output)
  end)

  it('should handle multiple inline code snippets in one line', function()
    local confluence_html = '<p>Run <code>git status</code> then <code>git commit</code> to save changes.</p>'

    local output = renderer.html_to_text(confluence_html)

    print("\n=== Multiple Inline Code ===")
    print(output)
    print("=== End Output ===\n")

    assert.has_match('`git status`', output)
    assert.has_match('`git commit`', output)
  end)

  it('should handle tt tags (teletype) as inline code', function()
    local confluence_html = '<p>The <tt>config.json</tt> file contains settings.</p>'

    local output = renderer.html_to_text(confluence_html)

    print("\n=== TT Tag Output ===")
    print(output)
    print("=== End Output ===\n")

    assert.has_match('`config.json`', output)
  end)

  it('should handle kbd tags (keyboard) as inline code', function()
    local confluence_html = '<p>Press <kbd>Ctrl+C</kbd> to copy.</p>'

    local output = renderer.html_to_text(confluence_html)

    print("\n=== KBD Tag Output ===")
    print(output)
    print("=== End Output ===\n")

    assert.has_match('`Ctrl%+C`', output)
  end)

  it('should differentiate between code blocks and inline code', function()
    local confluence_html = '<p>Example using <code>console.log()</code>:</p>' ..
      '<ac:structured-macro ac:name="code">' ..
      '<ac:parameter ac:name="language">javascript</ac:parameter>' ..
      '<ac:plain-text-body>console.log("Hello");</ac:plain-text-body>' ..
      '</ac:structured-macro>'

    local output = renderer.html_to_text(confluence_html)

    print("\n=== Mixed Code Output ===")
    print(output)
    print("=== End Output ===\n")

    -- Inline code should have backticks
    assert.has_match('`console.log%(%)`', output)

    -- Code block should have borders
    assert.has_match('┌─ JAVASCRIPT', output)
    assert.has_match('└', output)
  end)
end)
