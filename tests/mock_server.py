#!/usr/bin/env python3
"""
Mock Confluence API server for integration tests.
Responds to basic API endpoints with test data.
"""

from http.server import BaseHTTPRequestHandler, HTTPServer
import json
import sys
import re
from urllib.parse import urlparse, parse_qs

class MockConfluenceHandler(BaseHTTPRequestHandler):
    def log_message(self, format, *args):
        """Suppress default logging"""
        pass

    def do_GET(self):
        parsed = urlparse(self.path)
        path = parsed.path
        query = parse_qs(parsed.query)

        # Page endpoint: /rest/api/content/{id}
        if re.match(r'/rest/api/content/\d+', path):
            page_id = path.split('/')[-1].split('?')[0]
            self.send_page_response(page_id)

        # Spaces endpoint: /rest/api/space
        elif path == '/rest/api/space':
            self.send_spaces_response()

        # Search endpoint: /rest/api/content/search
        elif path.startswith('/rest/api/content/search'):
            cql = query.get('cql', [''])[0]
            self.send_search_response(cql)

        else:
            self.send_error(404, 'Endpoint not found')

    def send_page_response(self, page_id):
        """Send a mock page response"""
        self.send_response(200)
        self.send_header('Content-type', 'application/json')
        self.end_headers()

        # Different responses for different page IDs
        pages = {
            '123456': {
                'title': 'Getting Started',
                'content': '<h1>Getting Started</h1><p>Welcome!</p>'
            },
            '123457': {
                'title': 'Advanced Topics',
                'content': '<h1>Advanced</h1><p>More details...</p>'
            },
            '123458': {
                'title': 'Code Examples',
                'content': '<h1>Examples</h1><ac:structured-macro ac:name="code"><ac:parameter ac:name="language">rust</ac:parameter><ac:plain-text-body><![CDATA[fn main() {}]]></ac:plain-text-body></ac:structured-macro>'
            },
        }

        page_data = pages.get(page_id, {
            'title': f'Test Page {page_id}',
            'content': f'<p>Test content for page {page_id}</p>'
        })

        response = {
            'id': page_id,
            'type': 'page',
            'title': page_data['title'],
            'body': {
                'storage': {
                    'value': page_data['content'],
                    'representation': 'storage'
                }
            },
            'space': {
                'key': 'TEST',
                'name': 'Test Space'
            },
            'version': {
                'number': 1,
                'when': '2024-01-05T12:00:00.000Z'
            },
            '_links': {
                'webui': f'/spaces/TEST/pages/{page_id}'
            }
        }

        self.wfile.write(json.dumps(response).encode())

    def send_spaces_response(self):
        """Send mock spaces list"""
        self.send_response(200)
        self.send_header('Content-type', 'application/json')
        self.end_headers()

        response = {
            'results': [
                {
                    'id': 1,
                    'key': 'PROJ',
                    'name': 'Project Documentation',
                    'type': 'global',
                    'metadata': {
                        'labels': {'results': [],'start': 0, 'limit': 200, 'size': 0}
                    }
                },
                {
                    'id': 2,
                    'key': 'ENG',
                    'name': 'Engineering Docs',
                    'type': 'global',
                    'metadata': {
                        'labels': {'results': [], 'start': 0, 'limit': 200, 'size': 0}
                    }
                },
                {
                    'id': 3,
                    'key': 'TEST',
                    'name': 'Test Space',
                    'type': 'global',
                    'metadata': {
                        'labels': {'results': [], 'start': 0, 'limit': 200, 'size': 0}
                    }
                },
            ],
            'start': 0,
            'limit': 25,
            'size': 3
        }

        self.wfile.write(json.dumps(response).encode())

    def send_search_response(self, cql):
        """Send mock search results"""
        self.send_response(200)
        self.send_header('Content-type', 'application/json')
        self.end_headers()

        # Simple mock - return matching pages
        results = [
            {
                'id': '123456',
                'type': 'page',
                'title': 'Getting Started',
                'excerpt': 'Welcome to the <strong>project</strong> documentation',
                'space': {'key': 'TEST', 'name': 'Test Space'},
                '_links': {'webui': '/spaces/TEST/pages/123456'}
            }
        ]

        response = {
            'results': results,
            'start': 0,
            'limit': 25,
            'size': len(results)
        }

        self.wfile.write(json.dumps(response).encode())

def run_server(port=8080):
    """Start the mock server"""
    server = HTTPServer(('localhost', port), MockConfluenceHandler)
    print(f'Mock Confluence server running on http://localhost:{port}', file=sys.stderr)
    print('Press Ctrl+C to stop', file=sys.stderr)
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        print('\nShutting down...', file=sys.stderr)
        server.shutdown()

if __name__ == '__main__':
    port = int(sys.argv[1]) if len(sys.argv) > 1 else 8080
    run_server(port)
