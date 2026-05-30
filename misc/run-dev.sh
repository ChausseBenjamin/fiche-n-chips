#!/bin/sh

# meant to be run from the git root

export RUST_BACKTRACE=1
export PUBLIC_ADDR="localhost"
export LISTEN_ADDR="127.0.0.1"
export HTTP_PORT="9001"
export PASTE_PORT="9002"
# export HTML_DIR=""
export DB_PATH="app/.internal/"
export DB_CACHE="32"
export RETENTION="365days"
export SLUG_LEN="4"
export PASTE_MAX_SIZE="256KB"
export PASTE_READ_TIMEOUT="1s"
export REDIRECT_MAX_SIZE="2KB"
export REDIRECT_SLUG_LIMIT="144"
export LOG_LEVEL="debug"
export LOG_OUTPUT="stdout"
export LOG_FORMAT="json"
export LOG_TRACE="true"

target/debug/fiche-n-chips
