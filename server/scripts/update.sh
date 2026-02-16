#!/usr/bin/env bash
set -euo pipefail

REPO="shahadishraq/claudiator"
INSTALL_DIR="/opt/claudiator"
BINARY="$INSTALL_DIR/claudiator-server"
ENV_FILE="$INSTALL_DIR/.env"
SERVICE="claudiator-server"

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

# Must run as root
if [[ $EUID -ne 0 ]]; then
    echo "Error: This script must be run as root or with sudo"
    exit 1
fi

# Verify installation exists
if [[ ! -f "$BINARY" ]]; then
    echo "Error: claudiator-server not found at $BINARY"
    echo "Run install.sh first."
    exit 1
fi

# Get current version
CURRENT_VERSION=$("$BINARY" --version | grep -oE '[0-9]+\.[0-9]+\.[0-9]+')
if [[ -z "$CURRENT_VERSION" ]]; then
    echo "Error: Could not determine current version"
    exit 1
fi
echo "Current version: $CURRENT_VERSION"

# Detect architecture
ARCH=$(uname -m)
case "$ARCH" in
    x86_64)
        TARGET="x86_64-unknown-linux-gnu"
        ;;
    aarch64)
        TARGET="aarch64-unknown-linux-gnu"
        ;;
    *)
        echo "Error: Unsupported architecture: $ARCH"
        exit 1
        ;;
esac

# Query GitHub API for the latest server-v* release
echo "Checking for updates..."
RELEASES_JSON=$(curl -sfL "https://api.github.com/repos/$REPO/releases" 2>/dev/null) || {
    echo "Error: Failed to query GitHub releases API"
    exit 1
}

# Find the first release with a tag starting with "server-v"
# Extract tag_name for server releases
LATEST_TAG=$(echo "$RELEASES_JSON" | grep -o '"tag_name":\s*"server-v[^"]*"' | head -1 | sed 's/.*"server-v\([^"]*\)".*/\1/')
if [[ -z "$LATEST_TAG" ]]; then
    echo "Error: No server-v* release found"
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
ASSET_NAME="claudiator-server-${TARGET}.tar.gz"
DOWNLOAD_URL="https://github.com/$REPO/releases/download/server-v${LATEST_VERSION}/${ASSET_NAME}"

echo "Downloading $ASSET_NAME..."
if ! curl -fSL "$DOWNLOAD_URL" | tar -xz -C "$TEMP_DIR"; then
    echo "Error: Failed to download or extract binary"
    exit 1
fi

if [[ ! -f "$TEMP_DIR/claudiator-server" ]]; then
    echo "Error: Binary not found in downloaded archive"
    exit 1
fi

# Stop service
echo "Stopping $SERVICE..."
systemctl stop "$SERVICE" || true

# Backup current binary
echo "Backing up current binary..."
cp "$BINARY" "${BINARY}.bak"

# Install new binary
echo "Installing new binary..."
cp "$TEMP_DIR/claudiator-server" "$BINARY"
chmod 755 "$BINARY"
chown claudiator:claudiator "$BINARY"

# Start service
echo "Starting $SERVICE..."
systemctl start "$SERVICE"
sleep 2

# Health check
echo "Performing health check..."
set -a
source "$ENV_FILE"
set +a

if curl -sf -H "Authorization: Bearer $CLAUDIATOR_API_KEY" "http://localhost:${CLAUDIATOR_PORT}/api/v1/ping" > /dev/null 2>&1; then
    echo "Health check: OK"
    echo ""
    echo "Updated successfully: $CURRENT_VERSION -> $LATEST_VERSION"
    # Remove backup on success
    rm -f "${BINARY}.bak"
else
    echo "Health check: FAILED"
    echo "Rolling back to previous version..."
    systemctl stop "$SERVICE" || true
    mv "${BINARY}.bak" "$BINARY"
    chown claudiator:claudiator "$BINARY"
    systemctl start "$SERVICE"
    sleep 2

    if curl -sf -H "Authorization: Bearer $CLAUDIATOR_API_KEY" "http://localhost:${CLAUDIATOR_PORT}/api/v1/ping" > /dev/null 2>&1; then
        echo "Rollback successful. Previous version restored."
    else
        echo "Warning: Rollback health check also failed. Check logs:"
        echo "  journalctl -u $SERVICE -f"
    fi
    exit 1
fi
