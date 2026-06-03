# 🍟 fiche-n-chips

A modern rewrite of the classic [fiche][1] terminal pastebin — with a side of URL
shortening. Single binary, zero runtime dependencies.

To see a live instance, check out [1yr.ca][4] where I host mine.

## Introduction

- **Single binary** — everything needed to run is compiled into one executable.
  Static assets (homepage, 404 page, CSS, favicon, robots.txt, sitemap) are
  embedded at build time.
- **Deduplication** — pastes are content-hashed with xxh3. Identical content
  returns the same existing slug. Saves space, no surprises.
- **Compressed storage** — everything is stored brotli-compressed.
- **Browser-aware responses** — browsers get HTML, terminals get ANSI-coloured
  text. For color-less outputs in the terminal, set the `X-Disable-Colors: true` header.
- **Configurable retention** — entries live for a set duration (default: 365
  days), then auto-expire.
- **Graceful shutdown** — SIGTERM and Ctrl+C drain active connections before
  exiting.
- **Structured logging** — JSON or plain text output, configurable level and
  destination.

### Pastebin

```
# Send some text:
echo "Hello, world" | nc localhost 9002
# => localhost:9001/aB3x

# Send a text file:
cat ~/some_file.txt | nc localhost 9002
# => localhost:9001/nOvk

# Or send the output of any command:
ls -la | nc localhost 9002
# => localhost:9001/iOXa
```

Just pipe text over a raw TCP connection — works with netcat, socat, or any tool
that can open a TCP socket. Each paste gets a unique alphanumeric slug.

Paste reads have a per-chunk timeout (default: 1s) and a maximum body size
(default: 256 KB). Duplicate content is detected automatically and reuses the
same slug, so identical pastes always map to the same URL.

### Redirects (URL shortener)

```
# Get an auto-generated short URL:
curl -X POST http://localhost:9001/api/shorten \
  -H "Content-Type: application/json" \
  -d '{"url": "https://example.com/very/long/url"}'
# => Successfully created a short url for the requested target, cheers!
# => You may use the url below:
# => http://localhost:9001/xY9z

# Shorten to something specific:
curl -X POST http://localhost:9001/api/shorten \
  -H "Content-Type: application/json" \
  -d '{"url": "https://example.com/very/long/url", "slug": "customized" }'
# => Successfully created a short url for the requested target, cheers!
# => You may use the url below:
# => http://localhost:9001/customized
```

Accepts JSON with a required `url` field and an optional `slug` field for custom
short paths. Custom slugs are validated (no `.`, `?`, `&`, `/` or path
normalization tricks). If no slug is provided, a random one is generated
(configurable length, default: 4 characters).

Both pastes and redirects can be deleted within a configurable time window
(default: 10 minutes) via a `DELETE` request to the slug's URL.

### Why "fiche-n-chips"?

[Fiche][1] is the original C pastebin this is built on. The "-n-chips" is the
extra stuff bolted on: URL shortening, a web UI, an API, dedup, compression.
Also "fish and chips" is pretty based :P

## Self-Hosting

### Building

```sh
cargo build --release
```

Requires a nightly Rust toolchain — the exact version is pinned in
`rust-toolchain.toml`. On Linux, [sccache][2] and [mold][3] are used
automatically if installed (configured in `.cargo/config.toml`).

### Running

```sh
PUBLIC_ADDR="localhost" \
HTTP_PORT="9001" \
PASTE_PORT="9002" \
DB_PATH="app/.internal/" \
cargo run --release --package fiche-n-chips
```

### Configuration

Every setting can be set via CLI flag **or** environment variable — no config
file needed.

| Variable               | Default              | What it does                           |
|------------------------|----------------------|----------------------------------------|
| `PUBLIC_ADDR`          | `localhost`          | Public-facing domain                   |
| `LISTEN_ADDR`          | `127.0.0.1`          | Address to bind the server to          |
| `HTTP_PORT`            | `9001`               | HTTP server (browser & API)            |
| `PASTE_PORT`           | `9002`               | TCP pastebin (netcat)                  |
| `MAINTAINER_EMAIL`     | `john@example.com`   | Contact email in error messages        |
| `HTML_DIR`             | —                    | Directory for static web files[^1]     |
| `DB_PATH`              | `.internal/store.db` | Database file or directory             |
| `DB_CACHE`             | `32`                 | SQLite memory cache in MB              |
| `RETENTION`            | `365days`            | How long before entries get deleted    |
| `SLUG_LEN`             | `4`                  | Length of auto-generated slugs         |
| `PASTE_MAX_SIZE`       | `256KB`              | Maximum paste body size                |
| `PASTE_READ_TIMEOUT`   | `1s`                 | Per-chunk timeout for paste uploads    |
| `REDIRECT_MAX_SIZE`    | `2KB`                | Maximum URL length to shorten          |
| `REDIRECT_SLUG_LIMIT`  | `144`                | Maximum custom slug length             |
| `USER_DELETION_WINDOW` | `10m`                | Time window to delete your own entry   |
| `CLEANUP_INTERVAL`     | `1h`                 | How often expired entries are swept    |
| `SHUTDOWN_TIMEOUT`     | `10s`                | Graceful shutdown timeout              |
| `LOG_LEVEL`            | `info`               | One of: debug, info, warn, error       |
| `LOG_OUTPUT`           | `stdout`             | One of: stdout, stderr, or a file path |
| `LOG_FORMAT`           | `json`               | One of: plain, json, none              |
| `LOG_TRACE`            | `false`              | Include source location in log entries |

<!-- | `NTFY_SERVER` | —                | ntfy.sh URL for notifications | --->
<!-- | `NTFY_PREFIX` | `fiche-n-chips-` | ntfy topic prefix             | --->

See `cargo run -- --help` for the full list.

## Development

```sh
# Run tests
cargo test --workspace

# Lint
cargo clippy --workspace --all-targets --all-features

# Dev mode (short retention, frequent cleanup)
misc/run-dev.sh
```

## Roadmap

- [x] dual purpose
  - [x] terminal pastebin (netcat)
  - [x] url-shortening (API)
- [x] data stored in a simple sqlite database
- [x] data compresed with brotli in the server
- [x] web hosting (for homepage, 404, style.css robots.txt, favicon, etc...)
  - [x] Supports either
    - [x] files embedded in the binary
    - [x] specified directory (file list only detected at startup)
  - [x] Accepts multiple equivalent paths:
    - [x] with or without trailing slashes
    - [x] with or without the file extension
- [x] automatic cleanup after a set time-period (1yr by default)
- [x] user-based deletion within 10min. of creating a paste/redirect
- [x] graceful shutdown
- [x] user-privacy conscious (IPs can be salted+hashed to help user anonimity)
    - [x] rate-limiting based on hashed/anonimized IPs
    - [x] logs only track hashed/anonimized IPs
- [ ] Manage users with special permissions:
    - [ ] anonimized whitelist
    - [ ] anonimized blacklist
- [ ] [ntfy.sh][4] support for notifications:
  - [ ] prematurely deleted pastebin (server storage shortage)
  - [ ] short-url about to expire (ex: 1 week prior)

## License

MPL-2.0

[^1]: By default, the binary serves the html files for [1yr.ca][4] which are
      embedded inside of it. I **highly** recommend you change this by providing
      a custom `HTML_DIR`...

[0]: https://example.com
[1]: https://github.com/solusipse/fiche
[2]: https://github.com/mozilla/sccache
[3]: https://github.com/rui314/mold
[4]: https://1yr.ca
