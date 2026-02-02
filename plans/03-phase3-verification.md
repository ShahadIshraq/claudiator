# Phase 3 — End-to-End Verification

After Phase 1 and Phase 2 are complete. Manual testing.

---

## Step 1: Build

```bash
cd ~/workspace/claudiator
cargo build --workspace
```

Both `claudiator-hook` and `test-server` should compile without errors.

## Step 2: Smoke Tests

```bash
# Version check
cargo run -- version
# Expected: "claudiator-hook v0.1.0"

# Send with no config (should exit 0, log error)
echo '{"session_id":"x","hook_event_name":"Stop"}' | cargo run -- send
echo $?
# Expected: 0
cat ~/.claude/claudiator/error.log
# Expected: config read error logged

# Test with no config (should exit 1)
cargo run -- test
echo $?
# Expected: 1, error printed to stderr
```

## Step 3: Start Test Server

```bash
# Terminal 1
cargo run -p test-server -- --port 3000 --api-key "test-key"
# Expected output:
#   Claudiator test server running on http://0.0.0.0:3000
#   API key: test-key
#   Waiting for events...
```

## Step 4: Configure and Test Connection

```bash
# Terminal 2
mkdir -p ~/.claude/claudiator
cat > ~/.claude/claudiator/config.toml << 'EOF'
server_url = "http://localhost:3000"
api_key = "test-key"
device_name = "dev-machine"
device_id = "00000000-0000-0000-0000-000000000001"
platform = "mac"
EOF

cargo run -- test
# Expected: "Connection successful!"
# Test server terminal should show the ping request
```

## Step 5: Send Events

```bash
# Single event
echo '{"session_id":"sess-001","hook_event_name":"SessionStart","cwd":"/tmp/project"}' | cargo run -- send
# Expected: exit 0, test server logs:
#   [timestamp] EVENT received
#     Device:  dev-machine (mac) [00000000-...]
#     Session: sess-001
#     Event:   SessionStart
#     CWD:     /tmp/project
#   ---

# All event types
for event in SessionStart SessionEnd Stop Notification UserPromptSubmit; do
  echo "{\"session_id\":\"sess-001\",\"hook_event_name\":\"$event\",\"cwd\":\"/tmp\"}" | cargo run -- send
done
# Expected: 5 events logged on test server

# Notification with subtype
echo '{"session_id":"sess-001","hook_event_name":"Notification","notification_type":"permission_prompt","message":"Claude wants to run bash"}' | cargo run -- send
# Expected: test server shows notification_type in log
```

## Step 6: Error Cases

```bash
# Bad API key
cat > ~/.claude/claudiator/config.toml << 'EOF'
server_url = "http://localhost:3000"
api_key = "wrong-key"
device_name = "dev-machine"
device_id = "00000000-0000-0000-0000-000000000001"
platform = "mac"
EOF

cargo run -- test
# Expected: exit 1, "Connection failed: server returned 401: ..."

echo '{"session_id":"x","hook_event_name":"Stop"}' | cargo run -- send
echo $?
# Expected: exit 0 (always), error logged to error.log

# Server unreachable (stop test server first)
echo '{"session_id":"x","hook_event_name":"Stop"}' | cargo run -- send
echo $?
# Expected: exit 0, network error logged to error.log

# Malformed stdin
echo 'not json' | cargo run -- send
echo $?
# Expected: exit 0, parse error logged to error.log

# Empty stdin
echo '' | cargo run -- send
echo $?
# Expected: exit 0, parse error logged
```

## Step 7: Verify Error Log

```bash
cat ~/.claude/claudiator/error.log
# Should contain timestamped entries for each error above
# Format: [2026-02-02T15:30:00+00:00] send: server returned 401: ...
```

## Step 8: Install Script Dry Run (Optional)

```bash
# Test install script logic without actually downloading
# (useful after binary is built locally)
bash scripts/install.sh
# Walk through the prompts, verify config.toml is written correctly
# Verify hooks are merged into settings.json if chosen
```
