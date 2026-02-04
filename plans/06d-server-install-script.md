# Phase 4d — Server: Install Script APNs Configuration

## Overview

Update the server install script to optionally prompt for APNs configuration during fresh installs. Write APNs env vars to `.env` conditionally.

## Install Script Changes

`server/scripts/install.sh` — add after the DB_PATH prompt (fresh install only):

```bash
# APNs push notification configuration (optional)
echo ""
echo "Push Notifications (optional)"
echo "─────────────────────────────"
echo "Configure APNs for real-time push notifications to iOS."
echo "Requires an Apple Developer account with a .p8 APNs key."
echo "Skip this to use polling-only mode (notifications when app is open)."
echo ""
read -p "Configure APNs push? [y/N] " -r
echo

if [[ $REPLY =~ ^[Yy]$ ]]; then
    read -p "Path to .p8 key file: " -r APNS_KEY_PATH
    if [[ ! -f "$APNS_KEY_PATH" ]]; then
        echo "Warning: File not found at $APNS_KEY_PATH"
        echo "You can copy the key later and update .env manually."
    fi
    read -p "APNs Key ID (10-char): " -r APNS_KEY_ID
    read -p "Team ID: " -r APNS_TEAM_ID
    read -p "Bundle ID [com.claudiator.app]: " -r APNS_BUNDLE_ID
    APNS_BUNDLE_ID=${APNS_BUNDLE_ID:-com.claudiator.app}
    read -p "Use sandbox (for dev/TestFlight)? [Y/n] " -r
    if [[ $REPLY =~ ^[Nn]$ ]]; then
        APNS_SANDBOX=false
    else
        APNS_SANDBOX=true
    fi
fi
```

## .env File Changes

Update the `.env` write block to conditionally include APNs vars:

```bash
cat > /opt/claudiator/.env <<EOF
CLAUDIATOR_API_KEY=$API_KEY
CLAUDIATOR_PORT=$PORT
CLAUDIATOR_BIND=$BIND
CLAUDIATOR_DB_PATH=$DB_PATH
EOF

if [[ -n "${APNS_KEY_PATH:-}" ]]; then
    cat >> /opt/claudiator/.env <<EOF
CLAUDIATOR_APNS_KEY_PATH=$APNS_KEY_PATH
CLAUDIATOR_APNS_KEY_ID=$APNS_KEY_ID
CLAUDIATOR_APNS_TEAM_ID=$APNS_TEAM_ID
CLAUDIATOR_APNS_BUNDLE_ID=$APNS_BUNDLE_ID
CLAUDIATOR_APNS_SANDBOX=$APNS_SANDBOX
EOF
fi
```

## Install Summary Update

Add APNs status to the post-install summary:

```bash
if [[ -n "${APNS_KEY_PATH:-}" ]]; then
    echo "  APNs:     Enabled (key: $APNS_KEY_PATH)"
else
    echo "  APNs:     Not configured (polling-only mode)"
fi
```

## Files Modified

| File | Action |
|---|---|
| `server/scripts/install.sh` | modify — optional APNs prompts + conditional .env vars + summary |
