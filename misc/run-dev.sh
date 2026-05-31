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
export MAINTAINER_EMAIL="dev@example.com"
export USER_DELETION_WINDOW="10m"
# export RETENTION="365days"
export RETENTION="20s"
export SLUG_LEN="4"
# export CLEANUP_INTERVAL="1h"
export CLEANUP_INTERVAL="10s"
# export SHUTDOWN_TIMEOUT="10s"
export SHUTDOWN_TIMEOUT="100ms"
export NTFY_SERVER=""
export NTFY_PREFIX="fiche-n-chips-"
export PASTE_MAX_SIZE="256KB"
export PASTE_READ_TIMEOUT="1s"
export REDIRECT_MAX_SIZE="2KB"
export REDIRECT_SLUG_LIMIT="144"
export LOG_LEVEL="debug"
export LOG_OUTPUT="stdout"
export LOG_FORMAT="json"
export LOG_TRACE="true"

target/debug/fiche-n-chips
