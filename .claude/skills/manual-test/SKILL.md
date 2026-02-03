---
name: manual-test
description: Manually run and test the Claudiator server, hook test server, and iOS build
disable-model-invocation: true
argument-hint: "[component: server | hook | all]"
allowed-tools: Bash, Read
---

Run manual tests for the Claudiator project. The component to test is: $ARGUMENTS (default: all).

Follow these steps depending on the component requested.

## Ports

- Claudiator server: **3001** (use this for testing to avoid conflicts)
- Hook test server: **3002**

## 1. Server (`server` or `all`)

### Build and start

```bash
cd server
CLAUDIATOR_API_KEY=test-key CLAUDIATOR_PORT=3001 cargo run
```

Run the server in the background so you can send requests to it.

### Test sequence

Run these curl commands sequentially, checking each response:

**a) Ping**

```bash
curl -s -H "Authorization: Bearer test-key" http://localhost:3001/api/v1/ping
```

Expect: `{"status":"ok","server_version":"..."}`.

**b) SessionStart (no title)**

```bash
curl -s -X POST http://localhost:3001/api/v1/events \
  -H "Authorization: Bearer test-key" \
  -H "Content-Type: application/json" \
  -d '{"device":{"device_id":"test-device-001","device_name":"Test MacBook","platform":"mac"},"event":{"session_id":"sess-test-123","hook_event_name":"SessionStart","cwd":"/Users/dev/my-project"},"timestamp":"2026-01-01T10:00:00.000Z"}'
```

Expect: `{"status":"ok"}`.

**c) List sessions — no title yet**

```bash
curl -s -H "Authorization: Bearer test-key" http://localhost:3001/api/v1/devices/test-device-001/sessions
```

Expect: session with no `title` field (omitted when null).

**d) UserPromptSubmit — sets title**

```bash
curl -s -X POST http://localhost:3001/api/v1/events \
  -H "Authorization: Bearer test-key" \
  -H "Content-Type: application/json" \
  -d '{"device":{"device_id":"test-device-001","device_name":"Test MacBook","platform":"mac"},"event":{"session_id":"sess-test-123","hook_event_name":"UserPromptSubmit","cwd":"/Users/dev/my-project","prompt":"Fix the auth bug in login"},"timestamp":"2026-01-01T10:01:00.000Z"}'
```

Expect: `{"status":"ok"}`.

**e) List sessions — title should be set**

```bash
curl -s -H "Authorization: Bearer test-key" http://localhost:3001/api/v1/devices/test-device-001/sessions
```

Expect: `"title": "Fix the auth bug in login"`.

**f) Second UserPromptSubmit — title must NOT change**

```bash
curl -s -X POST http://localhost:3001/api/v1/events \
  -H "Authorization: Bearer test-key" \
  -H "Content-Type: application/json" \
  -d '{"device":{"device_id":"test-device-001","device_name":"Test MacBook","platform":"mac"},"event":{"session_id":"sess-test-123","hook_event_name":"UserPromptSubmit","cwd":"/Users/dev/my-project","prompt":"This should NOT replace the title"},"timestamp":"2026-01-01T10:02:00.000Z"}'
```

Then list sessions again and verify `title` is still `"Fix the auth bug in login"`.

**g) Truncation test — prompt over 200 chars**

Send a new session with a prompt that is 250+ characters long. Verify the returned title is truncated to 200 characters plus an ellipsis character.

**h) List devices**

```bash
curl -s -H "Authorization: Bearer test-key" http://localhost:3001/api/v1/devices
```

Expect: device list with `test-device-001`.

**i) List events**

```bash
curl -s -H "Authorization: Bearer test-key" http://localhost:3001/api/v1/sessions/sess-test-123/events
```

Expect: events array with the events sent above.

### Cleanup

Stop the background server process when done. Delete the test database:

```bash
rm -f server/claudiator.db server/claudiator.db-shm server/claudiator.db-wal
```

## 2. Hook test server (`hook` or `all`)

### Build and start

```bash
cd hook/test-server
cargo run -- --port 3002 --api-key test-key
```

Run in the background.

### Test

Send a UserPromptSubmit event with a prompt field:

```bash
curl -s -X POST http://localhost:3002/api/v1/events \
  -H "Authorization: Bearer test-key" \
  -H "Content-Type: application/json" \
  -d '{"device":{"device_id":"test-device-001","device_name":"Test MacBook","platform":"mac"},"event":{"session_id":"sess-test-456","hook_event_name":"UserPromptSubmit","prompt":"Hello from test"},"timestamp":"2026-01-01T10:00:00.000Z"}'
```

Check the test server stdout output. Expect to see `Prompt: Hello from test` in the log.

### Cleanup

Stop the background test server process.

## 3. iOS app

The iOS app cannot be tested from the CLI. Report that to the user and suggest:

- Open `ios/Claudiator.xcodeproj` in Xcode
- Build and run on simulator
- Verify session rows show the prompt title when available
- Verify the detail view shows a "Title" row
- Verify fallback: sessions without a title show the CWD or session ID

## Reporting

After running all tests, present a summary table:

| Test | Result |
|------|--------|
| Server ping | ... |
| SessionStart (no title) | ... |
| UserPromptSubmit sets title | ... |
| Second prompt does not overwrite title | ... |
| Truncation (>200 chars) | ... |
| List devices | ... |
| List events | ... |
| Hook test server logs prompt | ... |
| iOS (manual) | Requires Xcode |

Mark each as PASS or FAIL with a brief note if it failed.
