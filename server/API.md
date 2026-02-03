# Claudiator Server API Contract

Base URL: `{server_url}/api/v1`

All endpoints require authentication via Bearer token in the `Authorization` header.

## Authentication

Every request must include:

```
Authorization: Bearer {api_key}
```

Requests with a missing or invalid token receive a `401 Unauthorized` response:

```json
{
  "error": "unauthorized",
  "message": "Invalid or missing API key"
}
```

## Common Headers

The hook client sends the following headers on every request:

| Header          | Value                                  |
|-----------------|----------------------------------------|
| `Authorization` | `Bearer {api_key}`                     |
| `Content-Type`  | `application/json` (POST requests)     |
| `User-Agent`    | `claudiator-hook/{version}`            |

## Endpoints

### GET /api/v1/ping

Health check and connectivity test.

**Response: 200 OK**

```json
{
  "status": "ok",
  "server_version": "string"
}
```

---

### POST /api/v1/events

Ingest a hook event from a device.

**Request Body**

```json
{
  "device": {
    "device_id": "string",
    "device_name": "string",
    "platform": "string"
  },
  "event": {
    "session_id": "string",
    "hook_event_name": "string",
    "cwd": "string | null",
    "transcript_path": "string | null",
    "permission_mode": "string | null",
    "tool_name": "string | null",
    "tool_input": "object | null",
    "tool_output": "object | null",
    "notification_type": "string | null",
    "message": "string | null",
    "prompt": "string | null",
    "source": "string | null",
    "reason": "string | null",
    "subagent_id": "string | null",
    "subagent_type": "string | null"
  },
  "timestamp": "string (RFC 3339, millisecond precision)"
}
```

**Field Details**

`device` — Identifies the machine sending the event.

| Field         | Type   | Required | Description                                       |
|---------------|--------|----------|---------------------------------------------------|
| `device_id`   | string | yes      | Unique device identifier (UUID)                   |
| `device_name` | string | yes      | Human-readable device name (e.g. hostname)        |
| `platform`    | string | yes      | OS platform: `"mac"`, `"linux"`, or `"windows"`   |

`event` — The Claude Code hook event. Optional fields are omitted from the payload when null (not sent as explicit nulls).

| Field              | Type           | Required | Description                                          |
|--------------------|----------------|----------|------------------------------------------------------|
| `session_id`       | string         | yes      | Claude Code session identifier                       |
| `hook_event_name`  | string         | yes      | One of the hook event names (see below)              |
| `cwd`              | string         | no       | Working directory of the session                     |
| `transcript_path`  | string         | no       | Path to the session transcript file                  |
| `permission_mode`  | string         | no       | Permission mode of the session                       |
| `tool_name`        | string         | no       | Name of the tool being invoked                       |
| `tool_input`       | object         | no       | Tool input parameters                                |
| `tool_output`      | object         | no       | Tool output result                                   |
| `notification_type`| string         | no       | Type of notification                                 |
| `message`          | string         | no       | Notification or event message                        |
| `prompt`           | string         | no       | User prompt text                                     |
| `source`           | string         | no       | Event source                                         |
| `reason`           | string         | no       | Reason for the event (e.g. stop reason)              |
| `subagent_id`      | string         | no       | Sub-agent identifier                                 |
| `subagent_type`    | string         | no       | Sub-agent type                                       |

Unknown fields in the event object are preserved as-is (the hook uses a catch-all for forward compatibility).

`timestamp` — RFC 3339 timestamp with millisecond precision, e.g. `"2025-01-15T10:30:00.123Z"`.

**Hook Event Names**

| Event Name          | Description                              |
|---------------------|------------------------------------------|
| `SessionStart`      | A new Claude Code session began          |
| `SessionEnd`        | A session ended                          |
| `Stop`              | Execution was stopped                    |
| `Notification`      | A notification was generated             |
| `UserPromptSubmit`  | The user submitted a prompt              |

**Response: 200 OK**

```json
{
  "status": "ok"
}
```

---

### GET /api/v1/devices

List all known devices with active session counts.

**Response: 200 OK**

```json
{
  "devices": [
    {
      "device_id": "string",
      "device_name": "string",
      "platform": "string",
      "first_seen": "string (RFC 3339)",
      "last_seen": "string (RFC 3339)",
      "active_sessions": 0
    }
  ]
}
```

Devices are ordered by `last_seen` descending. `active_sessions` counts sessions with `status != 'ended'`.

---

### GET /api/v1/devices/:device_id/sessions

List sessions for a specific device.

**Query Parameters**

| Parameter | Type   | Default | Description                                           |
|-----------|--------|---------|-------------------------------------------------------|
| `status`  | string | —       | Filter by session status (e.g. `active`, `waiting_for_input`, `ended`) |
| `limit`   | int    | 50      | Maximum number of sessions to return                  |

**Response: 200 OK**

```json
{
  "sessions": [
    {
      "session_id": "string",
      "device_id": "string",
      "started_at": "string (RFC 3339)",
      "last_event": "string (RFC 3339)",
      "status": "string",
      "cwd": "string | null"
    }
  ]
}
```

Sessions are ordered by `last_event` descending. Returns an empty array if the device has no sessions.

---

### GET /api/v1/sessions/:session_id/events

List events for a specific session.

**Query Parameters**

| Parameter | Type | Default | Description                        |
|-----------|------|---------|------------------------------------|
| `limit`   | int  | 100     | Maximum number of events to return |

**Response: 200 OK**

```json
{
  "events": [
    {
      "id": 0,
      "hook_event_name": "string",
      "timestamp": "string (RFC 3339)",
      "tool_name": "string | null",
      "notification_type": "string | null",
      "message": "string | null"
    }
  ]
}
```

Events are ordered by `timestamp` descending. Returns selected fields only (not the full event JSON blob).

---

### POST /api/v1/push/register

Register a mobile device's push notification token.

**Request Body**

```json
{
  "platform": "string",
  "push_token": "string"
}
```

| Field        | Type   | Required | Description                          |
|--------------|--------|----------|--------------------------------------|
| `platform`   | string | yes      | `"ios"` or `"android"`               |
| `push_token` | string | yes      | APNs or FCM device token             |

If the token already exists, it is updated (upsert). Tokens are associated with the API key used for authentication.

**Response: 200 OK**

```json
{
  "status": "ok"
}
```

## Error Responses

| Status | Meaning                                  |
|--------|------------------------------------------|
| 401    | Missing or invalid `Authorization` token |
| 4xx    | Client error (malformed request, etc.)   |
| 5xx    | Server error                             |

The hook client treats any non-200 response as an error and logs the status code and response body.

## Example

```bash
curl -X POST https://your-server.com/api/v1/events \
  -H "Authorization: Bearer your-api-key" \
  -H "Content-Type: application/json" \
  -H "User-Agent: claudiator-hook/0.1.0" \
  -d '{
    "device": {
      "device_id": "550e8400-e29b-41d4-a716-446655440000",
      "device_name": "MacBook Pro",
      "platform": "mac"
    },
    "event": {
      "session_id": "sess-abc123",
      "hook_event_name": "SessionStart",
      "cwd": "/Users/dev/project"
    },
    "timestamp": "2025-01-15T10:30:00.123Z"
  }'
```

## Timeouts

The hook client enforces a 3-second timeout on all requests. The server should respond within that window.
