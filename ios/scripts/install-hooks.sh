#!/bin/bash

# Install git hooks from .githooks directory

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
HOOKS_DIR="$REPO_ROOT/.githooks"
GIT_HOOKS_DIR="$REPO_ROOT/.git/hooks"

echo "Installing git hooks from .githooks..."

if [ ! -d "$GIT_HOOKS_DIR" ]; then
  echo "Error: .git/hooks directory not found"
  exit 1
fi

if [ ! -d "$HOOKS_DIR" ]; then
  echo "Error: .githooks directory not found"
  exit 1
fi

# Install pre-commit hook
if [ -f "$HOOKS_DIR/pre-commit" ]; then
  cp "$HOOKS_DIR/pre-commit" "$GIT_HOOKS_DIR/pre-commit"
  chmod +x "$GIT_HOOKS_DIR/pre-commit"
  echo "✓ Installed pre-commit hook"
else
  echo "Warning: pre-commit hook not found in .githooks"
fi

echo "✓ Git hooks installed successfully"
echo ""
echo "To uninstall hooks, run: rm .git/hooks/pre-commit"
