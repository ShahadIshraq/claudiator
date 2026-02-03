#!/usr/bin/env bash
set -euo pipefail

# Cleanup temp directory on exit
TEMP_DIR=""
cleanup() {
    if [[ -n "$TEMP_DIR" && -d "$TEMP_DIR" ]]; then
        rm -rf "$TEMP_DIR"
    fi
}
trap cleanup EXIT

# Banner
echo "================================"
echo "  Claudiator Server Installer"
echo "================================"
echo ""

# Check prerequisites
if [[ $EUID -ne 0 ]]; then
    echo "Error: This script must be run as root or with sudo"
    exit 1
fi

if ! command -v systemctl &> /dev/null; then
    echo "Error: systemd is required but not found"
    exit 1
fi

if ! command -v curl &> /dev/null; then
    echo "Error: curl is required but not found"
    exit 1
fi

# Detect existing installation
IS_UPGRADE=false
if [[ -f /opt/claudiator/claudiator-server ]]; then
    IS_UPGRADE=true
    echo "Existing installation detected."
    read -p "Continue with upgrade? [Y/n] " -r
    echo
    if [[ $REPLY =~ ^[Nn]$ ]]; then
        echo "Upgrade cancelled."
        exit 0
    fi
    echo "Stopping service..."
    systemctl stop claudiator-server || true
fi

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
        echo "Supported architectures: x86_64, aarch64"
        exit 1
        ;;
esac

echo "Detected architecture: $ARCH"
echo "Target: $TARGET"
echo ""

# Download binary
echo "Downloading claudiator-server..."
TEMP_DIR=$(mktemp -d)
DOWNLOAD_URL="https://github.com/shahadishraq/claudiator/releases/latest/download/claudiator-server-${TARGET}.tar.gz"

if ! curl -fSL "$DOWNLOAD_URL" | tar -xz -C "$TEMP_DIR"; then
    echo "Error: Failed to download or extract binary"
    exit 1
fi

# Fresh install setup
if [[ "$IS_UPGRADE" = false ]]; then
    echo "Creating installation directory..."
    mkdir -p /opt/claudiator/data

    echo "Creating system user..."
    if ! id claudiator &> /dev/null; then
        useradd --system --no-create-home --shell /usr/sbin/nologin claudiator
    fi
fi

# Install binary
echo "Installing binary..."
cp "$TEMP_DIR/claudiator-server" /opt/claudiator/claudiator-server
chmod 755 /opt/claudiator/claudiator-server

# Set ownership
chown -R claudiator:claudiator /opt/claudiator

# Fresh install configuration
if [[ "$IS_UPGRADE" = false ]]; then
    echo ""
    echo "Configuration"
    echo "─────────────"

    # API key prompt
    GENERATED_KEY=$(openssl rand -hex 32)
    echo "Generated API key: $GENERATED_KEY"
    read -p "Use this key? [Y/n] " -r
    echo

    if [[ $REPLY =~ ^[Nn]$ ]]; then
        read -p "Enter API key: " -r API_KEY
        echo
    else
        API_KEY="$GENERATED_KEY"
    fi

    echo "Save this key — you'll need it to configure hook clients."
    echo ""

    # Port prompt
    read -p "Port [3000]: " -r PORT
    PORT=${PORT:-3000}

    # Validate port
    if ! [[ "$PORT" =~ ^[0-9]+$ ]] || [[ "$PORT" -lt 1 ]] || [[ "$PORT" -gt 65535 ]]; then
        echo "Error: Invalid port number. Must be between 1 and 65535."
        exit 1
    fi

    # Bind address prompt
    read -p "Bind address [0.0.0.0]: " -r BIND
    BIND=${BIND:-0.0.0.0}

    # Database path prompt
    read -p "Database path [/opt/claudiator/data/claudiator.db]: " -r DB_PATH
    DB_PATH=${DB_PATH:-/opt/claudiator/data/claudiator.db}

    # Write .env file
    echo "Writing configuration..."
    cat > /opt/claudiator/.env <<EOF
CLAUDIATOR_API_KEY=$API_KEY
CLAUDIATOR_PORT=$PORT
CLAUDIATOR_BIND=$BIND
CLAUDIATOR_DB_PATH=$DB_PATH
EOF

    chmod 600 /opt/claudiator/.env
    chown claudiator:claudiator /opt/claudiator/.env

    # Write systemd unit file
    echo "Creating systemd service..."
    cat > /etc/systemd/system/claudiator-server.service <<'EOF'
[Unit]
Description=Claudiator Event Ingestion Server
After=network.target

[Service]
Type=simple
User=claudiator
Group=claudiator
WorkingDirectory=/opt/claudiator
EnvironmentFile=/opt/claudiator/.env
ExecStart=/opt/claudiator/claudiator-server
Restart=on-failure
RestartSec=5
StandardOutput=journal
StandardError=journal

[Install]
WantedBy=multi-user.target
EOF

    systemctl daemon-reload
    systemctl enable claudiator-server
fi

# For upgrades, load existing config for health check
if [[ "$IS_UPGRADE" = true ]]; then
    # Source the .env file to get API_KEY and PORT
    set -a
    source /opt/claudiator/.env
    set +a
    API_KEY="$CLAUDIATOR_API_KEY"
    PORT="$CLAUDIATOR_PORT"
fi

# Start/restart service
echo ""
if [[ "$IS_UPGRADE" = true ]]; then
    echo "Restarting service..."
    systemctl restart claudiator-server
else
    echo "Starting service..."
    systemctl start claudiator-server
fi

sleep 2

# Health check
echo "Performing health check..."
if curl -sf -H "Authorization: Bearer $API_KEY" "http://localhost:${PORT}/api/v1/ping" > /dev/null 2>&1; then
    echo "Health check: OK"
else
    echo "Warning: Health check failed. Check logs with: journalctl -u claudiator-server -f"
fi

# Print summary
echo ""
if [[ "$IS_UPGRADE" = true ]]; then
    echo "  Claudiator Server Upgraded"
    echo "  ─────────────────────────────"
    echo "  Binary:   /opt/claudiator/claudiator-server"
    echo "  Config:   /opt/claudiator/.env (preserved)"
    echo "  Service:  claudiator-server (restarted)"
else
    echo "  Claudiator Server Installed"
    echo "  ─────────────────────────────"
    echo "  Binary:   /opt/claudiator/claudiator-server"
    echo "  Config:   /opt/claudiator/.env"
    echo "  Database: $DB_PATH"
    echo "  Service:  claudiator-server"
    echo ""
    echo "  Commands:"
    echo "    systemctl status claudiator-server"
    echo "    systemctl restart claudiator-server"
    echo "    journalctl -u claudiator-server -f"
    echo ""
    echo "  To uninstall:"
    echo "    systemctl stop claudiator-server"
    echo "    systemctl disable claudiator-server"
    echo "    rm /etc/systemd/system/claudiator-server.service"
    echo "    systemctl daemon-reload"
    echo "    userdel claudiator"
    echo "    rm -rf /opt/claudiator"
fi
echo ""
