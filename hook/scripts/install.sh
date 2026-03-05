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
# Query GitHub API for the latest hook-v* release
echo "Querying latest hook release..."
RELEASES_JSON=$(curl -sfL "https://api.github.com/repos/${REPO}/releases" 2>/dev/null) || {
    echo "Error: Failed to query GitHub releases API"
    exit 1
}

LATEST_TAG=$(echo "$RELEASES_JSON" | grep -o '"tag_name":\s*"hook-v[^"]*"' | head -1 | sed 's/.*"hook-v\([^"]*\)".*/\1/')
if [[ -z "$LATEST_TAG" ]]; then
    echo "Error: No hook release found on GitHub"
    exit 1
fi

echo "Latest hook release: v${LATEST_TAG}"
DOWNLOAD_URL="https://github.com/${REPO}/releases/download/hook-v${LATEST_TAG}/${BINARY_NAME}-${TARGET}.tar.gz"

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
    HOOK_HTTP_URL="${SERVER_URL%/}/api/v1/hooks/http"

    echo ""
    read -r -p "Hook transport (command/http/both) [command]: " HOOK_TRANSPORT
    HOOK_TRANSPORT=${HOOK_TRANSPORT:-command}
    HOOK_TRANSPORT=$(echo "$HOOK_TRANSPORT" | tr '[:upper:]' '[:lower:]')

    USE_COMMAND=false
    USE_HTTP=false
    case "$HOOK_TRANSPORT" in
        command)
            USE_COMMAND=true
            ;;
        http)
            USE_HTTP=true
            ;;
        both)
            USE_COMMAND=true
            USE_HTTP=true
            ;;
        *)
            echo "Unknown option '$HOOK_TRANSPORT' - defaulting to 'command'."
            USE_COMMAND=true
            ;;
    esac

    # Create settings directory if it doesn't exist
    mkdir -p "$HOME/.claude"

    # Define the hooks to add
    HOOKS_TO_ADD='[
      "SessionStart",
      "SessionEnd",
      "Stop",
      "Notification",
      "UserPromptSubmit",
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
        for EVENT in SessionStart SessionEnd Stop Notification UserPromptSubmit PermissionRequest TeammateIdle TaskCompleted; do
            if [ "$USE_COMMAND" = true ]; then
                # Check if command hook already exists
                EXISTING_CMD=$(jq -r --arg event "$EVENT" --arg cmd "$HOOK_COMMAND" \
                    '.hooks[$event] // [] | map(select(.hooks[]? | select(.command == $cmd))) | length' \
                    "$SETTINGS_FILE")

                if [ "$EXISTING_CMD" -eq 0 ]; then
                    # Add the command hook
                    jq --arg event "$EVENT" --arg cmd "$HOOK_COMMAND" \
                        '.hooks[$event] = (.hooks[$event] // []) + [{"matcher": "", "hooks": [{"type": "command", "command": $cmd}]}]' \
                        "$SETTINGS_FILE" > "${SETTINGS_FILE}.tmp" && mv "${SETTINGS_FILE}.tmp" "$SETTINGS_FILE"
                fi
            fi

            if [ "$USE_HTTP" = true ]; then
                # Update headers for matching HTTP hooks (if present) so reruns refresh credentials.
                jq --arg event "$EVENT" \
                    --arg url "$HOOK_HTTP_URL" \
                    --arg auth "Bearer $API_KEY" \
                    --arg device_id "$DEVICE_ID" \
                    --arg device_name "$DEVICE_NAME" \
                    --arg platform "$PLATFORM" \
                    '.hooks[$event] = ((.hooks[$event] // []) | map(
                        if (.hooks | type) == "array" then
                            .hooks = (.hooks | map(
                                if (.type == "http" and .url == $url) then
                                    . + {"headers": {"Authorization": $auth, "X-Claudiator-Device-Id": $device_id, "X-Claudiator-Device-Name": $device_name, "X-Claudiator-Platform": $platform}}
                                else
                                    .
                                end
                            ))
                        else
                            .
                        end
                    ))' \
                    "$SETTINGS_FILE" > "${SETTINGS_FILE}.tmp" && mv "${SETTINGS_FILE}.tmp" "$SETTINGS_FILE"

                # Check if HTTP hook already exists
                EXISTING_HTTP=$(jq -r --arg event "$EVENT" --arg url "$HOOK_HTTP_URL" \
                    '.hooks[$event] // [] | map(select(.hooks[]? | select(.type == "http" and .url == $url))) | length' \
                    "$SETTINGS_FILE")

                if [ "$EXISTING_HTTP" -eq 0 ]; then
                    # Add the HTTP hook
                    jq --arg event "$EVENT" \
                        --arg url "$HOOK_HTTP_URL" \
                        --arg auth "Bearer $API_KEY" \
                        --arg device_id "$DEVICE_ID" \
                        --arg device_name "$DEVICE_NAME" \
                        --arg platform "$PLATFORM" \
                        '.hooks[$event] = (.hooks[$event] // []) + [{"matcher": "", "hooks": [{"type": "http", "url": $url, "headers": {"Authorization": $auth, "X-Claudiator-Device-Id": $device_id, "X-Claudiator-Device-Name": $device_name, "X-Claudiator-Platform": $platform}}]}]' \
                        "$SETTINGS_FILE" > "${SETTINGS_FILE}.tmp" && mv "${SETTINGS_FILE}.tmp" "$SETTINGS_FILE"
                fi
            fi
        done
        HOOKS_CONFIGURED=true

    elif command -v python3 &> /dev/null; then
        # Use Python for JSON manipulation
        CLAUDIATOR_HTTP_URL="$HOOK_HTTP_URL" \
        CLAUDIATOR_USE_COMMAND="$USE_COMMAND" \
        CLAUDIATOR_USE_HTTP="$USE_HTTP" \
        CLAUDIATOR_API_KEY="$API_KEY" \
        CLAUDIATOR_DEVICE_ID="$DEVICE_ID" \
        CLAUDIATOR_DEVICE_NAME="$DEVICE_NAME" \
        CLAUDIATOR_PLATFORM="$PLATFORM" \
        python3 << 'PYEOF'
import json
import os

settings_file = os.path.expanduser("~/.claude/settings.json")
hook_command = "~/.claude/claudiator/claudiator-hook send"
hook_http_url = os.environ.get("CLAUDIATOR_HTTP_URL", "")
use_command = os.environ.get("CLAUDIATOR_USE_COMMAND") == "true"
use_http = os.environ.get("CLAUDIATOR_USE_HTTP") == "true"
api_key = os.environ.get("CLAUDIATOR_API_KEY", "")
device_id = os.environ.get("CLAUDIATOR_DEVICE_ID", "")
device_name = os.environ.get("CLAUDIATOR_DEVICE_NAME", "")
platform = os.environ.get("CLAUDIATOR_PLATFORM", "")
events = ["SessionStart", "SessionEnd", "Stop", "Notification", "UserPromptSubmit", "PermissionRequest", "TeammateIdle", "TaskCompleted"]

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

    if use_command:
        # Check if command hook already exists
        existing_command = any(
            any(h.get('command') == hook_command for h in hook.get('hooks', []))
            for hook in settings['hooks'][event]
        )

        if not existing_command:
            settings['hooks'][event].append({
                "matcher": "",
                "hooks": [{"type": "command", "command": hook_command}]
            })

    if use_http:
        existing_http = False
        for hook_group in settings['hooks'][event]:
            for h in hook_group.get('hooks', []):
                if h.get('type') == 'http' and h.get('url') == hook_http_url:
                    h['headers'] = {
                        "Authorization": f"Bearer {api_key}",
                        "X-Claudiator-Device-Id": device_id,
                        "X-Claudiator-Device-Name": device_name,
                        "X-Claudiator-Platform": platform
                    }
                    existing_http = True

        if not existing_http:
            settings['hooks'][event].append({
                "matcher": "",
                "hooks": [{
                    "type": "http",
                    "url": hook_http_url,
                    "headers": {
                        "Authorization": f"Bearer {api_key}",
                        "X-Claudiator-Device-Id": device_id,
                        "X-Claudiator-Device-Name": device_name,
                        "X-Claudiator-Platform": platform
                    }
                }]
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
        if [ "$USE_COMMAND" = true ] && [ "$USE_HTTP" = true ]; then
            cat << 'JSONEOF'
{
  "hooks": {
    "SessionStart": [
      {
        "matcher": "",
        "hooks": [
          {"type": "command", "command": "~/.claude/claudiator/claudiator-hook send"},
          {"type": "http", "url": "https://your-server.example.com/api/v1/hooks/http", "headers": {"Authorization": "Bearer <YOUR_API_KEY>", "X-Claudiator-Device-Id": "<YOUR_DEVICE_ID>", "X-Claudiator-Device-Name": "<YOUR_DEVICE_NAME>", "X-Claudiator-Platform": "<mac|linux|windows>"}}
        ]
      }
    ]
  }
}
JSONEOF
            echo ""
            echo "Repeat this hook stanza for: SessionEnd, Stop, Notification, UserPromptSubmit, PermissionRequest, TeammateIdle, TaskCompleted."
        elif [ "$USE_HTTP" = true ]; then
            cat << 'JSONEOF'
{
  "hooks": {
    "SessionStart": [
      {
        "matcher": "",
        "hooks": [{"type": "http", "url": "https://your-server.example.com/api/v1/hooks/http", "headers": {"Authorization": "Bearer <YOUR_API_KEY>", "X-Claudiator-Device-Id": "<YOUR_DEVICE_ID>", "X-Claudiator-Device-Name": "<YOUR_DEVICE_NAME>", "X-Claudiator-Platform": "<mac|linux|windows>"}}]
      }
    ]
  }
}
JSONEOF
            echo ""
            echo "Repeat this hook stanza for: SessionEnd, Stop, Notification, UserPromptSubmit, PermissionRequest, TeammateIdle, TaskCompleted."
        else
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
        fi
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
