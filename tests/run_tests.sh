#!/bin/bash
# Test runner for confluence-nvim

set -e

# Check if plenary.nvim is available
if ! [ -d "$HOME/.local/share/nvim/site/pack/vendor/start/plenary.nvim" ]; then
  echo "Installing plenary.nvim for testing..."
  mkdir -p "$HOME/.local/share/nvim/site/pack/vendor/start"
  git clone --depth=1 https://github.com/nvim-lua/plenary.nvim \
    "$HOME/.local/share/nvim/site/pack/vendor/start/plenary.nvim"
fi

# Get the directory of this script
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

# Run tests
echo "Running tests..."
nvim --headless --noplugin -u "$SCRIPT_DIR/minimal_init.lua" \
  -c "PlenaryBustedDirectory $SCRIPT_DIR { minimal_init = '$SCRIPT_DIR/minimal_init.lua' }"

echo ""
echo "Tests completed!"
