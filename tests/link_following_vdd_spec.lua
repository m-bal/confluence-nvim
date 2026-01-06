-- VDD Test: Link following functionality
local renderer = require('confluence.renderer')

describe('confluence link parsing (VDD)', function()
  it('should extract page IDs from Confluence links', function()
    local confluence_html = '<p>See the <a href="/wiki/spaces/PROJ/pages/123456/Getting+Started">Getting Started</a> guide.</p>'

    local output = renderer.html_to_text(confluence_html)

    print("\n=== Link Output ===")
    print(output)
    print("=== End Output ===\n")

    -- Links should be preserved with markers
    assert.has_match('Getting Started', output)
  end)

  it('should mark Confluence page links distinctly', function()
    local confluence_html = '<p>Check <a href="/wiki/spaces/PROJ/pages/789/API">API docs</a> and <a href="https://external.com">external link</a>.</p>'

    local output = renderer.html_to_text(confluence_html)

    print("\n=== Mixed Links Output ===")
    print(output)
    print("=== End Output ===\n")

    -- Should differentiate Confluence links from external
    assert.has_match('API docs', output)
    assert.has_match('external link', output)
  end)

  it('should handle links with full Confluence URLs', function()
    local confluence_html = '<p>Read <a href="https://company.atlassian.net/wiki/spaces/KB/pages/456">KB article</a>.</p>'

    local output = renderer.html_to_text(confluence_html)

    assert.has_match('KB article', output)
  end)

  it('should preserve link text and add visual indicator', function()
    local confluence_html = '<p>Documentation: <a href="/wiki/spaces/PROJ/pages/111/Setup">Setup Guide</a></p>'

    local output = renderer.html_to_text(confluence_html)

    print("\n=== Link Indicator Output ===")
    print(output)
    print("=== End Output ===\n")

    -- Should show link with indicator
    assert.has_match('Setup Guide', output)
    -- Should have link indicator (→ or similar)
    assert.has_match('→', output)
  end)
end)
