import http.server
import os
import sys
from pathlib import Path

MIME = {
    ".html": "text/html",
    ".css": "text/css",
    ".svg": "image/svg+xml",
    ".ico": "image/x-icon",
    ".woff2": "font/woff2",
    ".txt": "text/plain",
    ".xml": "application/xml",
}

def find_out_dir():
    root = Path(__file__).resolve().parent.parent
    candidates = list((root / "target/debug/build").glob("fiche-n-chips-*/out"))
    if not candidates:
        print("no build output found; build the project first", file=sys.stderr)
        sys.exit(1)
    return str(max(candidates, key=os.path.getmtime))

DIR = sys.argv[1] if len(sys.argv) > 1 else find_out_dir()

class Handler(http.server.SimpleHTTPRequestHandler):
    def __init__(self, *args, **kwargs):
        super().__init__(*args, directory=DIR, **kwargs)

    def end_headers(self):
        self.send_header("Content-Encoding", "br")
        super().end_headers()

    def guess_type(self, path):
        _, ext = os.path.splitext(path)
        return MIME.get(ext) or super().guess_type(path)

host = os.environ.get("HOST", "localhost")
port = int(os.environ.get("PORT", 8080))
httpd = http.server.HTTPServer((host, port), Handler)
print(f"Serving {os.path.abspath(DIR)} at http://{host}:{port}")
httpd.serve_forever()
