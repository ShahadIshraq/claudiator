#!/usr/bin/env bash
set -euo pipefail

REPO="shahadishraq/claudiator"
INSTALL_DIR="$HOME/.claude/claudiator"
BINARY="$INSTALL_DIR/claudiator-hook"

CHECK_ONLY=false
if [[ "${1:-}" = "--check" ]]; then
    CHECK_ONLY=true
fi

# Cleanup temp directory on exit
TEMP_DIR=""
cleanup() {
    if [[ -n "$TEMP_DIR" && -d "$TEMP_DIR" ]]; then
        rm -rf "$TEMP_DIR"
    fi
}
trap cleanup EXIT

# Detect OS
OS=$(uname -s)
case "$OS" in
    Darwin)
        OS_TARGET="apple-darwin"
        ;;
    Linux)
        OS_TARGET="unknown-linux-gnu"
        ;;
    *)
        echo "Error: Unsupported operating system: $OS"
        exit 1
        ;;
esac

# Detect architecture
ARCH=$(uname -m)
case "$ARCH" in
    arm64|aarch64)
        ARCH_TARGET="aarch64"
        ;;
    x86_64)
        ARCH_TARGET="x86_64"
        ;;
    *)
        echo "Error: Unsupported architecture: $ARCH"
        exit 1
        ;;
esac

# Build target string
TARGET="${ARCH_TARGET}-${OS_TARGET}"

# Verify installation exists
if [[ ! -f "$BINARY" ]]; then
    echo "Error: claudiator-hook not found at $BINARY"
    echo "Run install.sh first."
    exit 1
fi

# Get current version
CURRENT_VERSION=$("$BINARY" version | grep -oE '[0-9]+\.[0-9]+\.[0-9]+')
if [[ -z "$CURRENT_VERSION" ]]; then
    echo "Error: Could not determine current version"
    exit 1
fi
echo "Current version: $CURRENT_VERSION"

# Query GitHub API for the latest hook-v* release
echo "Checking for updates..."
RELEASES_JSON=$(curl -sfL "https://api.github.com/repos/$REPO/releases" 2>/dev/null) || {
    echo "Error: Failed to query GitHub releases API"
    exit 1
}

# Find the first release with a tag starting with "hook-v"
LATEST_TAG=$(echo "$RELEASES_JSON" | grep -o '"tag_name":\s*"hook-v[^"]*"' | head -1 | sed 's/.*"hook-v\([^"]*\)".*/\1/')
if [[ -z "$LATEST_TAG" ]]; then
    echo "Error: No hook-v* release found"
    exit 1
fi

LATEST_VERSION="$LATEST_TAG"
echo "Latest version:  $LATEST_VERSION"

# Compare versions
NEWER=$(printf '%s\n%s\n' "$CURRENT_VERSION" "$LATEST_VERSION" | sort -V | tail -1)
if [[ "$NEWER" = "$CURRENT_VERSION" ]]; then
    echo "Already up to date."
    exit 0
fi

echo "Update available: $CURRENT_VERSION -> $LATEST_VERSION"

if [[ "$CHECK_ONLY" = true ]]; then
    exit 0
fi

# Download new version
TEMP_DIR=$(mktemp -d)
ASSET_NAME="claudiator-hook-${TARGET}.tar.gz"
DOWNLOAD_URL="https://github.com/$REPO/releases/download/hook-v${LATEST_VERSION}/${ASSET_NAME}"

echo "Downloading $ASSET_NAME..."
if ! curl -fSL "$DOWNLOAD_URL" | tar -xz -C "$TEMP_DIR"; then
    echo "Error: Failed to download or extract binary"
    exit 1
fi

if [[ ! -f "$TEMP_DIR/claudiator-hook" ]]; then
    echo "Error: Binary not found in downloaded archive"
    exit 1
fi

# Backup current binary
echo "Backing up current binary..."
cp "$BINARY" "${BINARY}.bak"

# Install new binary
echo "Installing new binary..."
cp "$TEMP_DIR/claudiator-hook" "$BINARY"
chmod +x "$BINARY"

# Verify new binary works; rollback on failure
echo "Verifying new binary..."
if "$BINARY" version > /dev/null 2>&1; then
    rm -f "${BINARY}.bak"
    echo ""
    echo "Updated successfully: $CURRENT_VERSION -> $LATEST_VERSION"
else
    echo "Error: New binary failed to run. Rolling back..."
    cp "${BINARY}.bak" "$BINARY"
    rm -f "${BINARY}.bak"
    echo "Rollback successful. Previous version restored."
    exit 1
fi
