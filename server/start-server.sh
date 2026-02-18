#!/bin/bash
# Template script for starting the claudiator server.
# Copy this file or edit it directly, filling in your own values before running.
cd "$(dirname "$0")"
export CLAUDIATOR_API_KEY=your-api-key-here
export CLAUDIATOR_PORT=3000
export CLAUDIATOR_BIND=0.0.0.0
export CLAUDIATOR_DB_PATH=./claudiator-dev.db
export CLAUDIATOR_APNS_KEY_PATH="/path/to/your/AuthKey.p8"
export CLAUDIATOR_APNS_KEY_ID=YOUR_KEY_ID
export CLAUDIATOR_APNS_TEAM_ID=YOUR_TEAM_ID
export CLAUDIATOR_APNS_BUNDLE_ID=com.claudiator.app
export CLAUDIATOR_APNS_SANDBOX=true
./target/release/claudiator-server "$@"
