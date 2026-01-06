-- VDD Test: Code block rendering
local renderer = require('confluence.renderer')

describe('confluence code blocks (VDD)', function()
  it('should render code blocks with borders and language label', function()
    local confluence_html = '<ac:structured-macro ac:name="code" ac:schema-version="1">' ..
      '<ac:parameter ac:name="language">javascript</ac:parameter>' ..
      '<ac:plain-text-body>function hello() {\n  console.log("Hello World");\n}</ac:plain-text-body>' ..
      '</ac:structured-macro>'

    local output = renderer.html_to_text(confluence_html)

    print("\n=== Code Block Output ===")
    print(output)
    print("=== End Output ===\n")

    -- Should have a bordered box
    assert.has_match('┌', output)
    assert.has_match('└', output)
    assert.has_match('│', output)

    -- Should show language
    assert.has_match('JAVASCRIPT', output)

    -- Should contain code content
    assert.has_match('function hello', output)
    assert.has_match('console.log', output)
  end)

  it('should handle code blocks without CDATA', function()
    local confluence_html = '<ac:structured-macro ac:name="code">' ..
      '<ac:parameter ac:name="language">python</ac:parameter>' ..
      '<ac:plain-text-body>def greet():\n    print("Hello")</ac:plain-text-body>' ..
      '</ac:structured-macro>'

    local output = renderer.html_to_text(confluence_html)

    print("\n=== Python Code Output ===")
    print(output)
    print("=== End Output ===\n")

    assert.has_match('PYTHON', output)
    assert.has_match('def greet', output)
    assert.has_match('print', output)
  end)

  it('should handle code blocks with no language specified', function()
    local confluence_html = '<ac:structured-macro ac:name="code">' ..
      '<ac:plain-text-body>some code here</ac:plain-text-body>' ..
      '</ac:structured-macro>'

    local output = renderer.html_to_text(confluence_html)

    print("\n=== No Language Code Output ===")
    print(output)
    print("=== End Output ===\n")

    -- Should default to TEXT
    assert.has_match('TEXT', output)
    assert.has_match('some code here', output)
  end)

  it('should handle multiple code blocks in one page', function()
    local confluence_html = '<p>Here is some JavaScript:</p>' ..
      '<ac:structured-macro ac:name="code">' ..
      '<ac:parameter ac:name="language">javascript</ac:parameter>' ..
      '<ac:plain-text-body>const x = 1;</ac:plain-text-body>' ..
      '</ac:structured-macro>' ..
      '<p>And some Python:</p>' ..
      '<ac:structured-macro ac:name="code">' ..
      '<ac:parameter ac:name="language">python</ac:parameter>' ..
      '<ac:plain-text-body>x = 1</ac:plain-text-body>' ..
      '</ac:structured-macro>'

    local output = renderer.html_to_text(confluence_html)

    print("\n=== Multiple Code Blocks ===")
    print(output)
    print("=== End Output ===\n")

    assert.has_match('JAVASCRIPT', output)
    assert.has_match('PYTHON', output)
    assert.has_match('const x = 1', output)
    assert.has_match('x = 1', output)
  end)

  it('should preserve indentation and line breaks in code', function()
    local confluence_html = '<ac:structured-macro ac:name="code">' ..
      '<ac:parameter ac:name="language">javascript</ac:parameter>' ..
      '<ac:plain-text-body>function test() {\n  if (true) {\n    return 42;\n  }\n}</ac:plain-text-body>' ..
      '</ac:structured-macro>'

    local output = renderer.html_to_text(confluence_html)

    print("\n=== Indented Code Output ===")
    print(output)
    print("=== End Output ===\n")

    -- Should preserve the structure
    assert.has_match('function test', output)
    assert.has_match('if %(true%)', output)
    assert.has_match('return 42', output)

    -- CRITICAL: Should preserve indentation
    -- Lines inside the box should have their indentation preserved
    -- (│ + space + 2-space indent for 'if')
    assert.has_match('│   if %(true%)', output)  -- 3 total: 1 padding + 2 indent
    assert.has_match('│     return 42', output)  -- 5 total: 1 padding + 4 indent
  end)
end)
