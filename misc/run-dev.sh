#!/bin/sh

# meant to be run from the git root

export PUBLIC_ADDR="https://1yr.ca"
export LISTEN_ADDR="127.0.0.1"
export HTTP_PORT="9001"
export PASTE_PORT="9002"
# export HTML_DIR=""
export DB_PATH="app/.internal/"
export DB_CACHE="32"
export RETENTION="365days"
export SLUG_LEN="4"
export PASTE_MAX_KB="256"
export REDIRECT_MAX_KB="2"
export REDIRECT_SLUG_LIMIT="144"

target/debug/fiche-n-chips
