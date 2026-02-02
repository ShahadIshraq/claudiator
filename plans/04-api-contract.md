# API Contract

Defines the HTTP endpoints that both the real server and test server must implement.

---

## Authentication

All endpoints require:
```
Authorization: Bearer {api_key}
```

Responses on auth failure:
```
HTTP 401
{"error": "unauthorized", "message": "Invalid or missing API key"}
```

---

## `GET /api/v1/ping`

Health check and connection verification. Used by `claudiator-hook test`.

**Response (200):**
```json
{
  "status": "ok",
  "server_version": "1.0.0"
}
```

---

## `POST /api/v1/events`

Receive a hook event from a device.

**Request body:**
```json
{
  "device": {
    "device_id": "550e8400-e29b-41d4-a716-446655440000",
    "device_name": "shahads-macbook",
    "platform": "mac"
  },
  "event": {
    "session_id": "abc123-def456",
    "hook_event_name": "Notification",
    "cwd": "/Users/shahad/project",
    "transcript_path": "/Users/shahad/.claude/sessions/abc123.jsonl",
    "permission_mode": "default",
    "notification_type": "permission_prompt",
    "message": "Claude wants to run a bash command"
  },
  "timestamp": "2026-02-02T15:30:00.000Z"
}
```

**Event field presence by hook type:**

| Field | SessionStart | SessionEnd | Stop | Notification | UserPromptSubmit |
|---|---|---|---|---|---|
| session_id | always | always | always | always | always |
| hook_event_name | always | always | always | always | always |
| cwd | yes | yes | yes | yes | yes |
| transcript_path | yes | yes | yes | yes | yes |
| permission_mode | yes | yes | yes | yes | yes |
| source | yes (startup/resume) | — | — | — | — |
| reason | — | yes | yes | — | — |
| notification_type | — | — | — | yes | — |
| message | — | — | — | yes | — |
| prompt | — | — | — | — | yes |
| tool_name | — | — | — | — | — |

Note: `extra` fields (from `#[serde(flatten)]`) may include additional fields added by future Claude Code versions.

**Response (200):**
```json
{
  "status": "ok"
}
```

**Response (422):**
```json
{
  "error": "validation_error",
  "message": "missing required field: session_id"
}
```

---

## Headers Sent by Client

```
Content-Type: application/json
Authorization: Bearer {api_key}
User-Agent: claudiator-hook/0.1.0
```
