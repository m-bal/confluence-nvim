-- Tests with real Confluence XHTML storage format examples
local renderer = require('confluence.renderer')

describe('confluence real-world examples', function()
  describe('typical Confluence page content', function()
    it('should render a page with headings, paragraphs, and lists', function()
      -- Real example from Confluence storage format
      local confluence_html = [[
<h1>Getting Started</h1>
<p>Welcome to our documentation. This page will help you get started.</p>
<h2>Prerequisites</h2>
<ul>
  <li>Node.js 16 or higher</li>
  <li>npm or yarn</li>
  <li>Git</li>
</ul>
<h2>Installation</h2>
<p>Follow these steps:</p>
<ol>
  <li>Clone the repository</li>
  <li>Install dependencies with <code>npm install</code></li>
  <li>Run <code>npm start</code></li>
</ol>
      ]]

      local output = renderer.html_to_text(confluence_html)

      print("\n=== Rendered Output ===")
      print(output)
      print("=== End Output ===\n")

      -- Verify key content is present
      assert.has_match('Getting Started', output)
      assert.has_match('Prerequisites', output)
      assert.has_match('Node.js 16 or higher', output)
      assert.has_match('npm install', output)
    end)

    it('should render Confluence macros', function()
      -- Real Confluence macro example
      local confluence_html = '<ac:structured-macro ac:name="info" ac:schema-version="1">' ..
        '<ac:rich-text-body>' ..
        '<p>This is important information!</p>' ..
        '</ac:rich-text-body>' ..
        '</ac:structured-macro>' ..
        '<p>Regular content follows.</p>' ..
        '<ac:structured-macro ac:name="code" ac:schema-version="1">' ..
        '<ac:parameter ac:name="language">javascript</ac:parameter>' ..
        '<ac:plain-text-body>function hello() { console.log("Hello World"); }</ac:plain-text-body>' ..
        '</ac:structured-macro>'

      local output = renderer.html_to_text(confluence_html)

      print("\n=== Macro Output ===")
      print(output)
      print("=== End Output ===\n")

      assert.has_match('important information', output)
      assert.has_match('Regular content', output)
      assert.has_match('function hello', output)
    end)

    it('should render tables', function()
      local confluence_html = [[
<table>
  <thead>
    <tr>
      <th>Name</th>
      <th>Type</th>
      <th>Description</th>
    </tr>
  </thead>
  <tbody>
    <tr>
      <td>id</td>
      <td>string</td>
      <td>Unique identifier</td>
    </tr>
    <tr>
      <td>title</td>
      <td>string</td>
      <td>Page title</td>
    </tr>
  </tbody>
</table>
      ]]

      local output = renderer.html_to_text(confluence_html)

      print("\n=== Table Output ===")
      print(output)
      print("=== End Output ===\n")

      -- Tables are just stripped to text - we should see all content
      assert.has_match('Name', output)
      assert.has_match('Type', output)
      assert.has_match('Description', output)
      assert.has_match('Unique identifier', output)
    end)

    it('should handle complex nested structures', function()
      local confluence_html = [[
<ac:layout>
  <ac:layout-section ac:type="two_equal">
    <ac:layout-cell>
      <h3>Left Column</h3>
      <p>Content in left column with <strong>bold</strong> and <em>italic</em> text.</p>
      <ul>
        <li>Item 1</li>
        <li>Item 2</li>
      </ul>
    </ac:layout-cell>
    <ac:layout-cell>
      <h3>Right Column</h3>
      <p>Content in right column.</p>
    </ac:layout-cell>
  </ac:layout-section>
</ac:layout>
      ]]

      local output = renderer.html_to_text(confluence_html)

      print("\n=== Layout Output ===")
      print(output)
      print("=== End Output ===\n")

      assert.has_match('Left Column', output)
      assert.has_match('Right Column', output)
      assert.has_match('bold', output)
      assert.has_match('italic', output)
    end)

    it('should handle inline formatting and links', function()
      local confluence_html = [[
<p>This is a <strong>bold statement</strong> with <em>emphasis</em> and a
<a href="https://example.com">link to example</a>. We also support
<code>inline code</code> and <del>strikethrough</del> text.</p>
      ]]

      local output = renderer.html_to_text(confluence_html)

      print("\n=== Inline Formatting Output ===")
      print(output)
      print("=== End Output ===\n")

      -- All text should be present, formatting stripped
      assert.has_match('bold statement', output)
      assert.has_match('emphasis', output)
      assert.has_match('link to example', output)
      assert.has_match('inline code', output)
      assert.has_match('strikethrough', output)
    end)
  end)

  describe('rendering quality', function()
    it('should have reasonable line breaks between sections', function()
      local confluence_html = [[
<h1>Title</h1>
<p>First paragraph.</p>
<p>Second paragraph.</p>
<h2>Subsection</h2>
<p>Content here.</p>
      ]]

      local output = renderer.html_to_text(confluence_html)

      -- Check for double newlines after headings and paragraphs
      -- This ensures readable separation
      local lines = vim.split(output, '\n')

      print("\n=== Line Count: " .. #lines .. " ===")
      for i, line in ipairs(lines) do
        print(string.format("%2d: %s", i, line))
      end
      print("=== End Lines ===\n")

      -- Should have multiple lines (good separation)
      assert.is_true(#lines > 5)
    end)
  end)
end)
