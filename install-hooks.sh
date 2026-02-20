#!/bin/bash
# Installs the versioned pre-commit hook into .git/hooks/ so git picks it up
# automatically on every commit without any git config required.
set -e

REPO_ROOT=$(git rev-parse --show-toplevel)
TARGET="$REPO_ROOT/.git/hooks/pre-commit"
SOURCE="$REPO_ROOT/.githooks/pre-commit"

ln -sf "$SOURCE" "$TARGET"
chmod +x "$SOURCE"
echo "Hook installed: $TARGET -> $SOURCE"
