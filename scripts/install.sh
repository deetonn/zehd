#!/usr/bin/env bash
set -euo pipefail

INSTALL_DIR="${HOME}/.local/bin"

echo "Building zehd-cli (release)..."
cargo build -p zehd-cli --release

echo "Installing to ${INSTALL_DIR}/zehd..."
mkdir -p "$INSTALL_DIR"
cp ./target/release/zehd "$INSTALL_DIR/zehd"

echo "Done."

# Check PATH
if ! echo "$PATH" | tr ':' '\n' | grep -qF ".local/bin"; then
  echo ""
  echo "NOTE: ${INSTALL_DIR} is not in your PATH."
  echo "Add this to your ~/.zshrc:"
  echo "  export PATH=\"\$HOME/.local/bin:\$PATH\""
fi
