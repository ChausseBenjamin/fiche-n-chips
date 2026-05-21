import http.server
import os
import sys

DIR = sys.argv[1] if len(sys.argv) > 1 else "."

class Handler(http.server.SimpleHTTPRequestHandler):
    def __init__(self, *args, **kwargs):
        super().__init__(*args, directory=DIR, **kwargs)

    def end_headers(self):
        self.send_header("Content-Encoding", "br")
        super().end_headers()

host = os.environ.get("HOST", "localhost")
port = int(os.environ.get("PORT", 8080))
httpd = http.server.HTTPServer((host, port), Handler)
print(f"Serving {os.path.abspath(DIR)} at http://{host}:{port}")
httpd.serve_forever()
