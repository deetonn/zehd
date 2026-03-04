#!/usr/bin/env bash
#
# Creates a fresh zehd test project for development.
# Cleans up any previous test project, builds the CLI, scaffolds a new one.
#
# Usage: ./scripts/dev-project.sh [project-name]
#

set -euo pipefail

PROJECT_NAME="${1:-test-app}"
REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
PROJECT_DIR="$REPO_ROOT/$PROJECT_NAME"

# Cleanup previous project
if [ -d "$PROJECT_DIR" ]; then
  rm -rf "$PROJECT_DIR"
fi

# Build the CLI
cargo build -p zehd-cli --quiet 2>&1

# Scaffold project (run from repo root so it creates the dir here)
cd "$REPO_ROOT"
cargo run -p zehd-cli --quiet -- new "$PROJECT_NAME" <<< ""

echo ""
echo "Project ready: $PROJECT_DIR"
echo "  cd $PROJECT_DIR && cargo run -p zehd-cli --quiet -- dev"
