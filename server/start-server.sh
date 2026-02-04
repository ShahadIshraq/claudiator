#!/bin/bash
cd "$(dirname "$0")"
export CLAUDIATOR_API_KEY=test-key-12345
export CLAUDIATOR_PORT=3000
export CLAUDIATOR_BIND=0.0.0.0
export CLAUDIATOR_DB_PATH=./claudiator-dev.db
export CLAUDIATOR_APNS_KEY_PATH="/Users/shahadishraq/Documents/apple developer/claudiatorMe_AuthKey_QARUFK3TXT.p8"
export CLAUDIATOR_APNS_KEY_ID=QARUFK3TXT
export CLAUDIATOR_APNS_TEAM_ID=Y4X5LMM4FD
export CLAUDIATOR_APNS_BUNDLE_ID=com.claudiator.app
export CLAUDIATOR_APNS_SANDBOX=true
./target/release/claudiator-server "$@"
