-- Realistic Confluence Content Tests
-- These tests use actual XHTML patterns from real Confluence pages

local renderer = require('confluence.renderer')

describe('Realistic Confluence Content', function()

  describe('API Documentation Pattern', function()
    it('should render API endpoint documentation with parameters and examples', function()
      -- This is a typical pattern for REST API docs in Confluence
      local confluence_xhtml = '<h2>POST /api/users</h2>' ..
        '<p>Creates a new user account.</p>' ..
        '<h3>Parameters</h3>' ..
        '<table>' ..
        '<tr><th>Name</th><th>Type</th><th>Required</th><th>Description</th></tr>' ..
        '<tr><td><code>email</code></td><td>string</td><td>Yes</td><td>User email address</td></tr>' ..
        '<tr><td><code>name</code></td><td>string</td><td>Yes</td><td>Full name</td></tr>' ..
        '<tr><td><code>role</code></td><td>string</td><td>No</td><td>User role (default: viewer)</td></tr>' ..
        '</table>' ..
        '<h3>Example Request</h3>' ..
        '<ac:structured-macro ac:name="code">' ..
        '<ac:parameter ac:name="language">json</ac:parameter>' ..
        '<ac:plain-text-body>{\n  "email": "user@example.com",\n  "name": "John Doe",\n  "role": "admin"\n}</ac:plain-text-body>' ..
        '</ac:structured-macro>' ..
        '<h3>Example Response</h3>' ..
        '<ac:structured-macro ac:name="code">' ..
        '<ac:parameter ac:name="language">json</ac:parameter>' ..
        '<ac:plain-text-body>{\n  "id": "12345",\n  "email": "user@example.com",\n  "created_at": "2025-01-06T10:30:00Z"\n}</ac:plain-text-body>' ..
        '</ac:structured-macro>'

      local output = renderer.html_to_text(confluence_xhtml)

      -- Should have clear section headers
      assert.has_match('POST /api/users', output)
      assert.has_match('Parameters', output)
      assert.has_match('Example Request', output)

      -- Should render table structure
      assert.has_match('Name.*Type.*Required', output)
      assert.has_match('`email`', output)

      -- Should preserve JSON formatting in code blocks
      assert.has_match('"email": "user@example.com"', output)
      assert.has_match('"name": "John Doe"', output)

      -- Should have bordered code blocks
      assert.has_match('┌─ JSON', output)
      assert.has_match('└─', output)
    end)
  end)

  describe('Meeting Notes Pattern', function()
    it('should render meeting notes with attendees, agenda, and action items', function()
      -- Common pattern for team meeting notes
      local confluence_xhtml = '<h1>Engineering Sync - Jan 6, 2025</h1>' ..
        '<h2>Attendees</h2>' ..
        '<ul>' ..
        '<li>Alice (Engineering Lead)</li>' ..
        '<li>Bob (Backend Dev)</li>' ..
        '<li>Carol (Frontend Dev)</li>' ..
        '</ul>' ..
        '<h2>Agenda</h2>' ..
        '<ol>' ..
        '<li>Sprint review</li>' ..
        '<li>Technical debt discussion</li>' ..
        '<li>Q1 planning</li>' ..
        '</ol>' ..
        '<h2>Discussion</h2>' ..
        '<p>Team reviewed the completed sprint. Key highlights:</p>' ..
        '<ul>' ..
        '<li>Authentication system is <strong>complete</strong> and deployed</li>' ..
        '<li>Performance improvements show <em>40% reduction</em> in API latency</li>' ..
        '<li>Need to address the <code>user_cache</code> memory leak</li>' ..
        '</ul>' ..
        '<ac:structured-macro ac:name="info">' ..
        '<ac:rich-text-body><p>Next sprint starts January 13th</p></ac:rich-text-body>' ..
        '</ac:structured-macro>' ..
        '<h2>Action Items</h2>' ..
        '<table>' ..
        '<tr><th>Task</th><th>Owner</th><th>Due Date</th></tr>' ..
        '<tr><td>Fix user_cache memory leak</td><td>Bob</td><td>Jan 10</td></tr>' ..
        '<tr><td>Write Q1 planning doc</td><td>Alice</td><td>Jan 8</td></tr>' ..
        '</table>'

      local output = renderer.html_to_text(confluence_xhtml)

      -- Should have clear structure
      assert.has_match('Engineering Sync', output)
      assert.has_match('Attendees', output)
      assert.has_match('Alice %(Engineering Lead%)', output)

      -- Should preserve formatting
      assert.has_match('complete', output)  -- bold text preserved as word
      assert.has_match('40%% reduction', output)  -- emphasis preserved
      assert.has_match('`user_cache`', output)  -- inline code

      -- Should render info macro
      assert.has_match('INFO', output)
      assert.has_match('Next sprint starts', output)

      -- Should render action items table
      assert.has_match('Task.*Owner.*Due Date', output)
      assert.has_match('Fix user_cache memory leak', output)
    end)
  end)

  describe('Troubleshooting Guide Pattern', function()
    it('should render troubleshooting steps with warnings and code snippets', function()
      local confluence_xhtml = '<h1>Troubleshooting: Database Connection Issues</h1>' ..
        '<ac:structured-macro ac:name="warning">' ..
        '<ac:rich-text-body><p>Always backup your database before making configuration changes!</p></ac:rich-text-body>' ..
        '</ac:structured-macro>' ..
        '<h2>Symptoms</h2>' ..
        '<ul>' ..
        '<li>Application logs show <code>ECONNREFUSED</code> errors</li>' ..
        '<li>Health check endpoint returns 503</li>' ..
        '<li>Users see "Database unavailable" message</li>' ..
        '</ul>' ..
        '<h2>Diagnosis Steps</h2>' ..
        '<ol>' ..
        '<li>Check if database service is running:' ..
        '<ac:structured-macro ac:name="code">' ..
        '<ac:parameter ac:name="language">bash</ac:parameter>' ..
        '<ac:plain-text-body>systemctl status postgresql</ac:plain-text-body>' ..
        '</ac:structured-macro></li>' ..
        '<li>Verify connection string in config file:' ..
        '<ac:structured-macro ac:name="code">' ..
        '<ac:parameter ac:name="language">bash</ac:parameter>' ..
        '<ac:plain-text-body>cat /etc/app/database.conf | grep CONNECTION_STRING</ac:plain-text-body>' ..
        '</ac:structured-macro></li>' ..
        '<li>Test connection manually:' ..
        '<ac:structured-macro ac:name="code">' ..
        '<ac:parameter ac:name="language">bash</ac:parameter>' ..
        '<ac:plain-text-body>psql -h localhost -U appuser -d production</ac:plain-text-body>' ..
        '</ac:structured-macro></li>' ..
        '</ol>' ..
        '<h2>Common Solutions</h2>' ..
        '<h3>Solution 1: Restart Database Service</h3>' ..
        '<ac:structured-macro ac:name="code">' ..
        '<ac:parameter ac:name="language">bash</ac:parameter>' ..
        '<ac:plain-text-body>sudo systemctl restart postgresql\nsudo systemctl status postgresql</ac:plain-text-body>' ..
        '</ac:structured-macro>' ..
        '<h3>Solution 2: Check Firewall Rules</h3>' ..
        '<p>Ensure port 5432 is open:</p>' ..
        '<ac:structured-macro ac:name="code">' ..
        '<ac:parameter ac:name="language">bash</ac:parameter>' ..
        '<ac:plain-text-body>sudo firewall-cmd --list-all | grep 5432</ac:plain-text-body>' ..
        '</ac:structured-macro>' ..
        '<ac:structured-macro ac:name="note">' ..
        '<ac:rich-text-body><p>If issues persist, check the <a href="/wiki/spaces/ENG/pages/999">Database Maintenance Guide</a> →</p></ac:rich-text-body>' ..
        '</ac:structured-macro>'

      local output = renderer.html_to_text(confluence_xhtml)

      -- Should have warning
      assert.has_match('WARNING', output)
      assert.has_match('Always backup your database', output)

      -- Should have inline code
      assert.has_match('`ECONNREFUSED`', output)

      -- Should have code blocks with proper language
      assert.has_match('┌─ BASH', output)
      assert.has_match('systemctl status postgresql', output)

      -- Should preserve numbered steps structure
      assert.has_match('Diagnosis Steps', output)

      -- Should have note macro
      assert.has_match('NOTE', output)
      assert.has_match('Database Maintenance Guide', output)

      -- Should indicate internal link
      assert.has_match('→', output)
    end)
  end)

  describe('Design Document Pattern', function()
    it('should render design doc with requirements, architecture, and decisions', function()
      local confluence_xhtml = '<h1>Design: User Authentication Service</h1>' ..
        '<p><strong>Status:</strong> Approved | <strong>Author:</strong> Alice | <strong>Date:</strong> 2025-01-06</p>' ..
        '<h2>Overview</h2>' ..
        '<p>This document describes the design for implementing JWT-based authentication to replace the current session-based system.</p>' ..
        '<h2>Requirements</h2>' ..
        '<table>' ..
        '<tr><th>ID</th><th>Requirement</th><th>Priority</th></tr>' ..
        '<tr><td>R1</td><td>Support token refresh without re-login</td><td>High</td></tr>' ..
        '<tr><td>R2</td><td>Tokens must expire after 1 hour</td><td>High</td></tr>' ..
        '<tr><td>R3</td><td>Support multiple devices per user</td><td>Medium</td></tr>' ..
        '</table>' ..
        '<h2>Architecture</h2>' ..
        '<p>The system will consist of three main components:</p>' ..
        '<ul>' ..
        '<li><strong>Auth Service</strong> - Issues and validates JWT tokens</li>' ..
        '<li><strong>Token Store</strong> - Redis cache for refresh tokens</li>' ..
        '<li><strong>API Gateway</strong> - Validates tokens on each request</li>' ..
        '</ul>' ..
        '<ac:structured-macro ac:name="info">' ..
        '<ac:rich-text-body><p>We chose JWT over OAuth2 for simplicity since we don\'t need third-party auth.</p></ac:rich-text-body>' ..
        '</ac:structured-macro>' ..
        '<h2>Security Considerations</h2>' ..
        '<ac:structured-macro ac:name="warning">' ..
        '<ac:rich-text-body>' ..
        '<ul>' ..
        '<li>Tokens must be signed with RS256 (asymmetric)</li>' ..
        '<li>Never log token values</li>' ..
        '<li>Implement rate limiting on auth endpoints</li>' ..
        '</ul>' ..
        '</ac:rich-text-body>' ..
        '</ac:structured-macro>' ..
        '<h2>Implementation Plan</h2>' ..
        '<ol>' ..
        '<li>Set up JWT library and key management</li>' ..
        '<li>Implement token generation endpoint</li>' ..
        '<li>Add middleware for token validation</li>' ..
        '<li>Migrate existing sessions to tokens</li>' ..
        '<li>Deploy and monitor</li>' ..
        '</ol>'

      local output = renderer.html_to_text(confluence_xhtml)

      -- Should have metadata
      assert.has_match('Status:.*Approved', output)
      assert.has_match('Author:.*Alice', output)

      -- Should render requirements table
      assert.has_match('ID.*Requirement.*Priority', output)
      assert.has_match('R1.*Support token refresh', output)

      -- Should have component list
      assert.has_match('Auth Service', output)
      assert.has_match('Token Store', output)

      -- Should have info and warning boxes
      assert.has_match('INFO', output)
      assert.has_match('WARNING', output)
      assert.has_match('We chose JWT over OAuth2', output)

      -- Should preserve security items
      assert.has_match('Never log token values', output)
    end)
  end)

  describe('Release Notes Pattern', function()
    it('should render release notes with features, fixes, and breaking changes', function()
      local confluence_xhtml = '<h1>Release Notes - v2.5.0</h1>' ..
        '<p><strong>Release Date:</strong> January 6, 2025</p>' ..
        '<h2>🎉 New Features</h2>' ..
        '<ul>' ..
        '<li><strong>Dark Mode Support</strong> - Users can now toggle between light and dark themes in settings</li>' ..
        '<li><strong>Export to PDF</strong> - Added ability to export reports as PDF files</li>' ..
        '<li><strong>Real-time Notifications</strong> - WebSocket-based push notifications for important events</li>' ..
        '</ul>' ..
        '<h2>🐛 Bug Fixes</h2>' ..
        '<ul>' ..
        '<li>Fixed memory leak in <code>EventEmitter</code> class</li>' ..
        '<li>Resolved issue where dates showed incorrect timezone</li>' ..
        '<li>Corrected alignment issues in mobile view</li>' ..
        '</ul>' ..
        '<h2>⚡ Performance Improvements</h2>' ..
        '<ul>' ..
        '<li>Reduced initial page load time by 35%</li>' ..
        '<li>Optimized database queries for dashboard</li>' ..
        '<li>Implemented lazy loading for images</li>' ..
        '</ul>' ..
        '<ac:structured-macro ac:name="warning">' ..
        '<ac:rich-text-body>' ..
        '<p><strong>Breaking Changes</strong></p>' ..
        '<ul>' ..
        '<li>API endpoint <code>/api/v1/users</code> is deprecated, use <code>/api/v2/users</code> instead</li>' ..
        '<li>Configuration file format changed - see migration guide</li>' ..
        '</ul>' ..
        '</ac:rich-text-body>' ..
        '</ac:structured-macro>' ..
        '<h2>Migration Guide</h2>' ..
        '<p>To upgrade from v2.4.x to v2.5.0:</p>' ..
        '<ol>' ..
        '<li>Backup your database</li>' ..
        '<li>Update configuration file:' ..
        '<ac:structured-macro ac:name="code">' ..
        '<ac:parameter ac:name="language">yaml</ac:parameter>' ..
        '<ac:plain-text-body># Old format\napi:\n  version: v1\n\n# New format\napi:\n  version: v2\n  enable_deprecation_warnings: true</ac:plain-text-body>' ..
        '</ac:structured-macro></li>' ..
        '<li>Run migration script: <code>npm run migrate</code></li>' ..
        '</ol>'

      local output = renderer.html_to_text(confluence_xhtml)

      -- Should have release header
      assert.has_match('Release Notes %- v2%.5%.0', output)
      assert.has_match('Release Date:', output)

      -- Should have emoji section headers (or at least the text)
      assert.has_match('New Features', output)
      assert.has_match('Bug Fixes', output)
      assert.has_match('Performance Improvements', output)

      -- Should have inline code
      assert.has_match('`EventEmitter`', output)
      assert.has_match('`/api/v1/users`', output)

      -- Should have warning for breaking changes
      assert.has_match('WARNING', output)
      assert.has_match('Breaking Changes', output)

      -- Should have code block for config
      assert.has_match('┌─ YAML', output)
      assert.has_match('api:', output)
    end)
  end)

  describe('Nested and Complex Content', function()
    it('should handle deeply nested lists with mixed content types', function()
      local confluence_xhtml = '<h2>Project Structure</h2>' ..
        '<ul>' ..
        '<li>Frontend' ..
        '<ul>' ..
        '<li>Components' ..
        '<ul>' ..
        '<li><code>Header.tsx</code> - Main navigation component</li>' ..
        '<li><code>Footer.tsx</code> - Site footer with links</li>' ..
        '</ul></li>' ..
        '<li>Pages' ..
        '<ul>' ..
        '<li><code>Home.tsx</code> - Landing page</li>' ..
        '<li><code>Dashboard.tsx</code> - User dashboard with <strong>charts</strong> and <em>widgets</em></li>' ..
        '</ul></li>' ..
        '</ul></li>' ..
        '<li>Backend' ..
        '<ul>' ..
        '<li>Controllers' ..
        '<ul>' ..
        '<li><code>UserController.js</code></li>' ..
        '<li><code>AuthController.js</code></li>' ..
        '</ul></li>' ..
        '<li>Services' ..
        '<ul>' ..
        '<li><code>EmailService.js</code> - Handles email notifications</li>' ..
        '<li><code>PaymentService.js</code> - Integration with Stripe API</li>' ..
        '</ul></li>' ..
        '</ul></li>' ..
        '</ul>'

      local output = renderer.html_to_text(confluence_xhtml)

      -- Should preserve nested structure (checking for indentation or bullets)
      assert.has_match('Frontend', output)
      assert.has_match('Components', output)
      assert.has_match('`Header%.tsx`', output)
      assert.has_match('Backend', output)
      assert.has_match('Services', output)
      assert.has_match('`EmailService%.js`', output)

      -- Should preserve inline formatting
      assert.has_match('charts', output)
      assert.has_match('widgets', output)
    end)

    it('should handle tables inside macros and mixed content', function()
      local confluence_xhtml = '<h2>Environment Configuration</h2>' ..
        '<ac:structured-macro ac:name="info">' ..
        '<ac:rich-text-body>' ..
        '<p>Configuration values for each environment:</p>' ..
        '<table>' ..
        '<tr><th>Variable</th><th>Development</th><th>Production</th></tr>' ..
        '<tr><td><code>API_URL</code></td><td>http://localhost:3000</td><td>https://api.example.com</td></tr>' ..
        '<tr><td><code>DB_HOST</code></td><td>localhost</td><td>db.prod.internal</td></tr>' ..
        '<tr><td><code>LOG_LEVEL</code></td><td>debug</td><td>error</td></tr>' ..
        '</table>' ..
        '<p>Use <code>.env.development</code> or <code>.env.production</code> files.</p>' ..
        '</ac:rich-text-body>' ..
        '</ac:structured-macro>'

      local output = renderer.html_to_text(confluence_xhtml)

      -- Should have info box
      assert.has_match('INFO', output)
      assert.has_match('Configuration values', output)

      -- Should render table inside macro
      assert.has_match('Variable.*Development.*Production', output)
      assert.has_match('`API_URL`', output)
      assert.has_match('localhost:3000', output)
      assert.has_match('api%.example%.com', output)

      -- Should have inline code
      assert.has_match('`%.env%.development`', output)
    end)

    it('should handle multiple macros in sequence with different types', function()
      local confluence_xhtml = '<h2>Deployment Checklist</h2>' ..
        '<ac:structured-macro ac:name="info">' ..
        '<ac:rich-text-body><p>Pre-deployment requirements</p></ac:rich-text-body>' ..
        '</ac:structured-macro>' ..
        '<ul>' ..
        '<li>All tests passing</li>' ..
        '<li>Code review approved</li>' ..
        '<li>Security scan completed</li>' ..
        '</ul>' ..
        '<ac:structured-macro ac:name="warning">' ..
        '<ac:rich-text-body><p>Database migrations will cause 2-minute downtime</p></ac:rich-text-body>' ..
        '</ac:structured-macro>' ..
        '<ac:structured-macro ac:name="note">' ..
        '<ac:rich-text-body><p>Remember to notify #engineering channel before deploying</p></ac:rich-text-body>' ..
        '</ac:structured-macro>' ..
        '<ac:structured-macro ac:name="code">' ..
        '<ac:parameter ac:name="language">bash</ac:parameter>' ..
        '<ac:plain-text-body>./deploy.sh production --with-migrations</ac:plain-text-body>' ..
        '</ac:structured-macro>'

      local output = renderer.html_to_text(confluence_xhtml)

      -- Should have all macro types
      assert.has_match('INFO', output)
      assert.has_match('WARNING', output)
      assert.has_match('NOTE', output)

      -- Should preserve list between macros
      assert.has_match('All tests passing', output)
      assert.has_match('Security scan completed', output)

      -- Should have code block
      assert.has_match('┌─ BASH', output)
      assert.has_match('%./deploy%.sh production', output)
    end)
  end)

  describe('Real-world Edge Cases', function()
    it('should handle empty paragraphs and excessive whitespace', function()
      local confluence_xhtml = '<h2>Introduction</h2>' ..
        '<p></p>' ..
        '<p>   </p>' ..
        '<p>This is the actual content.</p>' ..
        '<p></p>' ..
        '<p>Another paragraph after empty ones.</p>'

      local output = renderer.html_to_text(confluence_xhtml)

      -- Should have content without excessive blank lines
      assert.has_match('Introduction', output)
      assert.has_match('This is the actual content', output)
      assert.has_match('Another paragraph', output)

      -- Should not have excessive whitespace (hard to test precisely, but output shouldn't be mostly empty)
      local non_whitespace = output:gsub('%s+', '')
      assert.is_true(#non_whitespace > 20, "Should have substantial content")
    end)

    it('should handle special characters and HTML entities', function()
      local confluence_xhtml = '<p>Use the &lt;div&gt; tag for layout.</p>' ..
        '<p>The price is $100 &amp; tax is 10%.</p>' ..
        '<p>Quote: &quot;Hello World&quot;</p>' ..
        '<p>Math: 5 &lt; 10 &amp;&amp; 10 &gt; 5</p>'

      local output = renderer.html_to_text(confluence_xhtml)

      -- Should decode HTML entities properly
      assert.has_match('<div>', output)
      assert.has_match('$100 & tax', output)
      assert.has_match('"Hello World"', output)
      assert.has_match('5 < 10', output)
      assert.has_match('10 > 5', output)
    end)

    it('should handle links with various href patterns', function()
      local confluence_xhtml = '<p>See the <a href="/wiki/spaces/ENG/pages/12345/API+Documentation">API Docs</a> →</p>' ..
        '<p>External link: <a href="https://example.com">Example Site</a></p>' ..
        '<p>Relative: <a href="/pages/67890">Related Page</a> →</p>' ..
        '<p>Email: <a href="mailto:support@example.com">Contact Support</a></p>'

      local output = renderer.html_to_text(confluence_xhtml)

      -- Should indicate internal Confluence links with arrow
      assert.has_match('API Docs →', output)
      assert.has_match('Related Page →', output)

      -- External links should not have the arrow (or should have it removed in the pattern)
      assert.has_match('Example Site', output)
      assert.has_match('Contact Support', output)
    end)
  end)
end)
