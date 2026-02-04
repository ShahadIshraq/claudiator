#!/usr/bin/env bash

# Claudiator Test Data Seeder
# Seeds dummy devices, sessions, and events for testing
#
# Usage:
#   ./scripts/seed.sh                           # defaults: localhost:3000, test-key
#   SERVER_URL=http://host:8080 ./scripts/seed.sh
#   API_KEY=my-key ./scripts/seed.sh

set -euo pipefail

SERVER_URL="${SERVER_URL:-http://localhost:3000}"
API_KEY="${API_KEY:-test-key}"

GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m'

EVENT_COUNT=0

# Generate ISO8601 timestamp N minutes ago
ts() {
    local minutes_ago=$1
    if [[ "$OSTYPE" == "darwin"* ]]; then
        date -u -v-"${minutes_ago}"M +"%Y-%m-%dT%H:%M:%S.000Z"
    else
        date -u -d "$minutes_ago minutes ago" +"%Y-%m-%dT%H:%M:%S.000Z"
    fi
}

# Post an event to the server
# Args: device_id device_name platform session_id hook_event_name timestamp [json_extra_event_fields]
post_event() {
    local device_id=$1
    local device_name=$2
    local platform=$3
    local session_id=$4
    local hook_event_name=$5
    local timestamp=$6
    local extra=${7:-}

    local event_fields="\"session_id\": \"$session_id\", \"hook_event_name\": \"$hook_event_name\""
    if [[ -n "$extra" ]]; then
        event_fields="$event_fields, $extra"
    fi

    local payload
    payload=$(cat <<EOF
{
    "device": {
        "device_id": "$device_id",
        "device_name": "$device_name",
        "platform": "$platform"
    },
    "event": {
        $event_fields
    },
    "timestamp": "$timestamp"
}
EOF
)

    local http_code
    http_code=$(curl -s -o /dev/null -w "%{http_code}" -X POST "$SERVER_URL/api/v1/events" \
        -H "Authorization: Bearer $API_KEY" \
        -H "Content-Type: application/json" \
        -d "$payload")

    if [[ "$http_code" == "200" ]]; then
        echo -e "  ${GREEN}✓${NC} $session_id: $hook_event_name"
        EVENT_COUNT=$((EVENT_COUNT + 1))
    else
        echo -e "  ${RED}✗${NC} $session_id: $hook_event_name (HTTP $http_code)"
        exit 1
    fi
}

echo -e "${BLUE}Seeding Claudiator at $SERVER_URL${NC}"
echo ""

# ── Device 1: Shahad's MacBook Pro ──────────────────────────────
echo -e "${BLUE}Device 1: Shahad's MacBook Pro (mac)${NC}"

# Session 1 — active, waiting for input
post_event "dev-macbook-001" "Shahad's MacBook Pro" "mac" \
    "sess-mac-001" "SessionStart" "$(ts 30)" \
    '"cwd": "/Users/shahad/workspace/web-app"'

post_event "dev-macbook-001" "Shahad's MacBook Pro" "mac" \
    "sess-mac-001" "UserPromptSubmit" "$(ts 28)" \
    '"cwd": "/Users/shahad/workspace/web-app", "prompt": "Help me refactor the authentication module to use OAuth2"'

post_event "dev-macbook-001" "Shahad's MacBook Pro" "mac" \
    "sess-mac-001" "Stop" "$(ts 27)" \
    '"cwd": "/Users/shahad/workspace/web-app"'

# Session 2 — ended
post_event "dev-macbook-001" "Shahad's MacBook Pro" "mac" \
    "sess-mac-002" "SessionStart" "$(ts 60)" \
    '"cwd": "/Users/shahad/workspace/api-server"'

post_event "dev-macbook-001" "Shahad's MacBook Pro" "mac" \
    "sess-mac-002" "UserPromptSubmit" "$(ts 58)" \
    '"cwd": "/Users/shahad/workspace/api-server", "prompt": "Fix the database connection pool timeout issue"'

post_event "dev-macbook-001" "Shahad's MacBook Pro" "mac" \
    "sess-mac-002" "SessionEnd" "$(ts 50)" \
    '"cwd": "/Users/shahad/workspace/api-server"'

echo ""

# ── Device 2: Linux Server ──────────────────────────────────────
echo -e "${BLUE}Device 2: prod-server-01 (linux)${NC}"

# Session 1 — waiting for permission
post_event "dev-linux-001" "prod-server-01" "linux" \
    "sess-linux-001" "SessionStart" "$(ts 45)" \
    '"cwd": "/home/deploy/services/payment-api"'

post_event "dev-linux-001" "prod-server-01" "linux" \
    "sess-linux-001" "UserPromptSubmit" "$(ts 43)" \
    '"cwd": "/home/deploy/services/payment-api", "prompt": "Add rate limiting to the payment endpoints"'

post_event "dev-linux-001" "prod-server-01" "linux" \
    "sess-linux-001" "Notification" "$(ts 42)" \
    '"cwd": "/home/deploy/services/payment-api", "notification_type": "permission_prompt"'

# Session 2 — idle
post_event "dev-linux-001" "prod-server-01" "linux" \
    "sess-linux-002" "SessionStart" "$(ts 90)" \
    '"cwd": "/home/deploy/services/notification-svc"'

post_event "dev-linux-001" "prod-server-01" "linux" \
    "sess-linux-002" "Notification" "$(ts 85)" \
    '"cwd": "/home/deploy/services/notification-svc", "notification_type": "idle_prompt"'

echo ""

# ── Device 3: Office iMac ───────────────────────────────────────
echo -e "${BLUE}Device 3: Office iMac (mac)${NC}"

# Session 1 — active with multiple prompts
post_event "dev-desktop-001" "Office iMac" "mac" \
    "sess-desk-001" "SessionStart" "$(ts 40)" \
    '"cwd": "/Users/shahad/projects/mobile-app"'

post_event "dev-desktop-001" "Office iMac" "mac" \
    "sess-desk-001" "UserPromptSubmit" "$(ts 38)" \
    '"cwd": "/Users/shahad/projects/mobile-app", "prompt": "Implement push notification handling for iOS"'

post_event "dev-desktop-001" "Office iMac" "mac" \
    "sess-desk-001" "Stop" "$(ts 35)" \
    '"cwd": "/Users/shahad/projects/mobile-app"'

post_event "dev-desktop-001" "Office iMac" "mac" \
    "sess-desk-001" "UserPromptSubmit" "$(ts 20)" \
    '"cwd": "/Users/shahad/projects/mobile-app", "prompt": "Now add the server-side push endpoint"'

echo ""
echo -e "${GREEN}Done!${NC} Seeded 3 devices, 5 sessions, $EVENT_COUNT events"
