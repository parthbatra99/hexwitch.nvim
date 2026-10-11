#!/bin/sh
# No personal Neovim configuration, plugin manager, downloads, or credential files.
set -eu
root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
export NVIM_APPNAME=hexwitch-standalone
export HEXWITCH_STANDALONE_INIT="$root/adapters/nvim/examples/standalone.lua"
exec nvim --clean -c 'lua dofile(vim.env.HEXWITCH_STANDALONE_INIT)' "$@"
