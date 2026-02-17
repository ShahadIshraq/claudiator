#!/usr/bin/env bash
set -euo pipefail

# Banner
echo "================================"
echo "  Claudiator Hook Installer"
echo "================================"
echo ""

# Detect OS
OS=$(uname -s)
case "$OS" in
    Darwin)
        OS_TARGET="apple-darwin"
        PLATFORM="mac"
        ;;
    Linux)
        OS_TARGET="unknown-linux-gnu"
        PLATFORM="linux"
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

# Set variables
INSTALL_DIR="$HOME/.claude/claudiator"
BINARY_NAME="claudiator-hook"
REPO="shahadishraq/claudiator"
DOWNLOAD_URL="https://github.com/${REPO}/releases/latest/download/${BINARY_NAME}-${TARGET}.tar.gz"

# Create install directory
mkdir -p "$INSTALL_DIR"

# Download and extract
echo "Downloading ${BINARY_NAME} for ${TARGET}..."
if ! curl -fsSL "$DOWNLOAD_URL" | tar xz -C "$INSTALL_DIR/"; then
    echo "Error: Failed to download or extract binary"
    exit 1
fi

# Make executable
chmod +x "$INSTALL_DIR/$BINARY_NAME"

# Prompt for configuration
echo ""
read -r -p "Server URL: " SERVER_URL
read -r -s -p "API Key: " API_KEY
echo ""

# Set device info
DEVICE_NAME=$(hostname)
if command -v uuidgen &> /dev/null; then
    DEVICE_ID=$(uuidgen)
elif [ -f /proc/sys/kernel/random/uuid ]; then
    DEVICE_ID=$(cat /proc/sys/kernel/random/uuid)
else
    DEVICE_ID=$(python3 -c 'import uuid; print(uuid.uuid4())')
fi

# Write config.toml
cat > "$INSTALL_DIR/config.toml" << EOF
server_url = "$SERVER_URL"
api_key = "$API_KEY"
device_name = "$DEVICE_NAME"
device_id = "$DEVICE_ID"
platform = "$PLATFORM"
EOF

# Test connection
echo ""
echo "Testing connection..."
if ! "$INSTALL_DIR/$BINARY_NAME" test; then
    echo "Warning: Connection test failed. You can re-run: $INSTALL_DIR/$BINARY_NAME test"
else
    echo "Connection test successful!"
fi

# Ask about hooks configuration
echo ""
read -r -p "Auto-configure Claude Code hooks in ~/.claude/settings.json? [Y/n]: " CONFIGURE_HOOKS
CONFIGURE_HOOKS=${CONFIGURE_HOOKS:-Y}

HOOKS_CONFIGURED=false

if [[ "$CONFIGURE_HOOKS" =~ ^[Yy]$ ]] || [[ -z "$CONFIGURE_HOOKS" ]]; then
    SETTINGS_FILE="$HOME/.claude/settings.json"
    HOOK_COMMAND="~/.claude/claudiator/claudiator-hook send"

    # Create settings directory if it doesn't exist
    mkdir -p "$HOME/.claude"

    # Define the hooks to add
    HOOKS_TO_ADD='[
      "SessionStart",
      "SessionEnd",
      "Stop",
      "Notification",
      "UserPromptSubmit",
      "SubagentStart",
      "SubagentStop",
      "PermissionRequest",
      "TeammateIdle",
      "TaskCompleted"
    ]'

    if command -v jq &> /dev/null; then
        # Use jq for JSON manipulation
        # Create file if it doesn't exist
        if [ ! -f "$SETTINGS_FILE" ]; then
            echo '{}' > "$SETTINGS_FILE"
        fi

        # For each hook event, add if not already present
        for EVENT in SessionStart SessionEnd Stop Notification UserPromptSubmit SubagentStart SubagentStop PermissionRequest TeammateIdle TaskCompleted; do
            # Check if hook already exists
            EXISTING=$(jq -r --arg event "$EVENT" --arg cmd "$HOOK_COMMAND" \
                '.hooks[$event] // [] | map(select(.hooks[]? | select(.command == $cmd))) | length' \
                "$SETTINGS_FILE")

            if [ "$EXISTING" -eq 0 ]; then
                # Add the hook
                jq --arg event "$EVENT" --arg cmd "$HOOK_COMMAND" \
                    '.hooks[$event] = (.hooks[$event] // []) + [{"matcher": "", "hooks": [{"type": "command", "command": $cmd}]}]' \
                    "$SETTINGS_FILE" > "${SETTINGS_FILE}.tmp" && mv "${SETTINGS_FILE}.tmp" "$SETTINGS_FILE"
            fi
        done
        HOOKS_CONFIGURED=true

    elif command -v python3 &> /dev/null; then
        # Use Python for JSON manipulation
        python3 << 'PYEOF'
import json
import os

settings_file = os.path.expanduser("~/.claude/settings.json")
hook_command = "~/.claude/claudiator/claudiator-hook send"
events = ["SessionStart", "SessionEnd", "Stop", "Notification", "UserPromptSubmit", "SubagentStart", "SubagentStop", "PermissionRequest", "TeammateIdle", "TaskCompleted"]

# Load or create settings
if os.path.exists(settings_file):
    with open(settings_file, 'r') as f:
        settings = json.load(f)
else:
    settings = {}

# Ensure hooks key exists
if 'hooks' not in settings:
    settings['hooks'] = {}

# Add hooks for each event
for event in events:
    if event not in settings['hooks']:
        settings['hooks'][event] = []

    # Check if hook already exists
    existing = any(
        any(h.get('command') == hook_command for h in hook.get('hooks', []))
        for hook in settings['hooks'][event]
    )

    if not existing:
        settings['hooks'][event].append({
            "matcher": "",
            "hooks": [{"type": "command", "command": hook_command}]
        })

# Write back
with open(settings_file, 'w') as f:
    json.dump(settings, f, indent=2)
PYEOF
        HOOKS_CONFIGURED=true

    else
        # No jq or python3 available, print manual instructions
        echo ""
        echo "Neither jq nor python3 found. Please manually add the following to ~/.claude/settings.json:"
        echo ""
        cat << 'JSONEOF'
{
  "hooks": {
    "SessionStart": [
      {
        "matcher": "",
        "hooks": [{"type": "command", "command": "~/.claude/claudiator/claudiator-hook send"}]
      }
    ],
    "SessionEnd": [
      {
        "matcher": "",
        "hooks": [{"type": "command", "command": "~/.claude/claudiator/claudiator-hook send"}]
      }
    ],
    "Stop": [
      {
        "matcher": "",
        "hooks": [{"type": "command", "command": "~/.claude/claudiator/claudiator-hook send"}]
      }
    ],
    "Notification": [
      {
        "matcher": "",
        "hooks": [{"type": "command", "command": "~/.claude/claudiator/claudiator-hook send"}]
      }
    ],
    "UserPromptSubmit": [
      {
        "matcher": "",
        "hooks": [{"type": "command", "command": "~/.claude/claudiator/claudiator-hook send"}]
      }
    ],
    "SubagentStart": [
      {
        "matcher": "",
        "hooks": [{"type": "command", "command": "~/.claude/claudiator/claudiator-hook send"}]
      }
    ],
    "SubagentStop": [
      {
        "matcher": "",
        "hooks": [{"type": "command", "command": "~/.claude/claudiator/claudiator-hook send"}]
      }
    ],
    "PermissionRequest": [
      {
        "matcher": "",
        "hooks": [{"type": "command", "command": "~/.claude/claudiator/claudiator-hook send"}]
      }
    ],
    "TeammateIdle": [
      {
        "matcher": "",
        "hooks": [{"type": "command", "command": "~/.claude/claudiator/claudiator-hook send"}]
      }
    ],
    "TaskCompleted": [
      {
        "matcher": "",
        "hooks": [{"type": "command", "command": "~/.claude/claudiator/claudiator-hook send"}]
      }
    ]
  }
}
JSONEOF
        echo ""
    fi
fi

# Print summary
echo ""
echo "================================"
echo "  Installation Complete!"
echo "================================"
echo "  ✓ Binary installed to: $INSTALL_DIR/$BINARY_NAME"
echo "  ✓ Config written to: $INSTALL_DIR/config.toml"
if [ "$HOOKS_CONFIGURED" = true ]; then
    echo "  ✓ Claude Code hooks configured in ~/.claude/settings.json"
fi
echo ""
echo "  To test: $INSTALL_DIR/$BINARY_NAME test"
echo "  To uninstall: rm -rf $INSTALL_DIR"
echo "================================"
echo ""
