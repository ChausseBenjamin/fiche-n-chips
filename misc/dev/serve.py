import http.server
import os
import sys
from pathlib import Path
from urllib.parse import urlsplit

MIME = {
    ".html": "text/html",
    ".css": "text/css",
    ".svg": "image/svg+xml",
    ".ico": "image/x-icon",
    ".woff2": "font/woff2",
    ".woff": "font/woff",
    ".txt": "text/plain",
    ".xml": "application/xml",
    ".png": "image/png",
    ".jpg": "image/jpeg",
    ".jpeg": "image/jpeg",
    ".webp": "image/webp",
    ".json": "application/json",
    ".map": "application/json",
    ".wasm": "application/wasm",
    ".webmanifest": "application/manifest+json",
}

# Extensions the build script brotli-compresses (see app/build.rs).
BROTLI_EXTENSIONS = {".html", ".css", ".svg"}


def find_out_dir():
    root = Path(__file__).resolve().parent.parent.parent
    candidates = list((root / "target/debug/build").glob("fiche-n-chips-*/out"))
    if not candidates:
        print("no build output found; build the project first", file=sys.stderr)
        sys.exit(1)
    return str(max(candidates, key=os.path.getmtime))


DIR = None


class Handler(http.server.SimpleHTTPRequestHandler):
    def __init__(self, *args, **kwargs):
        super().__init__(*args, directory=DIR, **kwargs)

    def end_headers(self):
        # Only the assets the build script compressed are brotli-encoded.
        # Everything else (png, woff2, txt, xml, webmanifest, error pages) is
        # served as-is, so advertising `br` for it would corrupt it.
        path = urlsplit(self.path).path
        if path.endswith("/"):
            path += "index.html"
        if os.path.splitext(path)[1] in BROTLI_EXTENSIONS:
            self.send_header("Content-Encoding", "br")
        super().end_headers()

    def guess_type(self, path):
        _, ext = os.path.splitext(path)
        return MIME.get(ext) or super().guess_type(path)


def main():
    global DIR
    DIR = sys.argv[1] if len(sys.argv) > 1 else find_out_dir()

    host = os.environ.get("HOST", "localhost")
    port = int(os.environ.get("PORT", 8080))
    httpd = http.server.HTTPServer((host, port), Handler)
    print(f"Serving {os.path.abspath(DIR)} at http://{host}:{port}")
    httpd.serve_forever()


if __name__ == "__main__":
    main()
