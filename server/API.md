# Claudiator Server API Contract

Base URL: `{server_url}/api/v1`

All endpoints require authentication via Bearer token in the `Authorization` header.

## Authentication

Every request must include:

```
Authorization: Bearer {key}
```

Two key types are accepted:

- **Master key** (`CLAUDIATOR_API_KEY`): Always has full read+write access.
- **Scoped keys**: Created via the admin API. Each key has one or more scopes (`read`, `write`). Using a key for an endpoint that requires a different scope returns `403 Forbidden`.

Requests with a missing or invalid token receive `401 Unauthorized`. A valid key used on an endpoint requiring a different scope receives `403 Forbidden`.

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
  "server_version": "string",
  "data_version": 0,
  "notification_version": 0
}
```

**Field Details**

| Field | Type | Description |
|---|---|---|
| `status` | string | Health check status |
| `server_version` | string | Server version identifier |
| `data_version` | number | Incremented on each event ingestion. Clients can poll this to detect new data. |
| `notification_version` | number | Incremented when a new notification is created. Clients can poll this to detect new notifications. |

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

The server stores only the 7 declared fields (`session_id`, `hook_event_name`, `cwd`, `prompt`, `notification_type`, `tool_name`, `message`). All other fields are silently dropped.

`timestamp` — RFC 3339 timestamp with millisecond precision, e.g. `"2025-01-15T10:30:00.123Z"`.

**Hook Event Names**

| Event Name          | Description                              |
|---------------------|------------------------------------------|
| `SessionStart`      | A new Claude Code session began          |
| `SessionEnd`        | A session ended                          |
| `Stop`              | Execution was stopped                    |
| `Notification`      | A notification was generated             |
| `UserPromptSubmit`  | The user submitted a prompt              |
| `SubagentStart`     | A subagent was started                   |
| `SubagentStop`      | A subagent was stopped                   |
| `PermissionRequest` | A tool requested user permission         |
| `TeammateIdle`      | A teammate went idle                     |
| `TaskCompleted`     | A task was completed                     |

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
      "cwd": "string | null",
      "title": "string | null",
      "device_name": "string | null",
      "platform": "string | null"
    }
  ]
}
```

Sessions are ordered by `last_event` descending. Returns an empty array if the device has no sessions.

**Field Details**

| Field        | Type          | Description                                                                 |
|--------------|---------------|-----------------------------------------------------------------------------|
| `title`      | string / null | Session title derived from the first user prompt. Null if no prompt has been submitted yet. |
| `device_name` | string / null | Device name (included when listing all sessions) |
| `platform` | string / null | Device platform (included when listing all sessions) |

---

### GET /api/v1/sessions

List all sessions across all devices.

**Query Parameters**

| Parameter | Type   | Default | Description                                           |
|-----------|--------|---------|-------------------------------------------------------|
| `status`  | string | —       | Filter by session status |
| `limit`   | int    | 200     | Maximum number of sessions to return                  |

**Response: 200 OK**

Same response shape as `GET /api/v1/devices/:device_id/sessions`.

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
  "push_token": "string",
  "sandbox": false
}
```

| Field        | Type   | Required | Description                          |
|--------------|--------|----------|--------------------------------------|
| `platform`   | string | yes      | `"ios"` or `"android"`               |
| `push_token` | string | yes      | APNs or FCM device token             |
| `sandbox`    | boolean | no      | Whether the token uses the APNs sandbox environment (default: false) |

If the token already exists, it is updated (upsert). Tokens are associated with the API key used for authentication.

**Response: 200 OK**

```json
{
  "status": "ok"
}
```

---

### GET /api/v1/notifications

List notification records. Notifications are auto-cleaned after 24 hours.

**Query Parameters**

| Parameter | Type | Default | Description |
|---|---|---|---|
| `after` | string (UUID) | — | Return only notifications created after this notification ID |
| `limit` | int | 50 | Maximum number of notifications to return (max 200) |

**Response: 200 OK**

```json
{
  "notifications": [
    {
      "id": "string (UUID)",
      "event_id": 0,
      "session_id": "string",
      "device_id": "string",
      "title": "string",
      "body": "string",
      "notification_type": "string",
      "payload_json": "string | null",
      "created_at": "string (RFC 3339)",
      "acknowledged": false
    }
  ]
}
```

Notifications are ordered by `created_at` ascending. Use the `after` parameter with the last received notification `id` to poll for new notifications incrementally.

**Notification Types**

| notification_type | Triggered by | Title |
|---|---|---|
| `stop` | `Stop` hook event | "Session Stopped" |
| `permission_prompt` | `Notification` event with `notification_type: "permission_prompt"` | "Permission Required" |
| `idle_prompt` | `Notification` event with `notification_type: "idle_prompt"` | "Session Idle" |
| `permission_prompt` | `PermissionRequest` hook event | "Permission Required" |

---

### POST /api/v1/notifications/ack

Bulk acknowledge notifications.

**Request Body**

```json
{
  "ids": ["string (UUID)", "..."]
}
```

| Field | Type | Required | Description |
|---|---|---|---|
| `ids` | array of strings | yes | Notification UUIDs to acknowledge |

**Response: 200 OK**

```json
{
  "status": "ok"
}
```

## Admin Endpoints

Admin endpoints manage API keys. They require:
- A connection from localhost (`127.0.0.1` or `::1`)
- The master key in the `Authorization` header

Base path: `/admin`

### POST /admin/api-keys

Create a new API key.

**Request Body**

```json
{
  "name": "string",
  "scopes": ["read", "write"],
  "rate_limit": 1000
}
```

| Field | Type | Required | Description |
|---|---|---|---|
| `rate_limit` | number | no | Maximum requests per minute for this key (default: 1000) |

**Response: 201 Created**

```json
{
  "id": "string (UUID)",
  "name": "string",
  "key": "string",
  "scopes": ["string"],
  "created_at": "string (RFC 3339)",
  "rate_limit": null
}
```

The `key` field contains the full API key (`claud_<32-hex-chars>`). Store it securely — it is not retrievable after creation.

---

### GET /admin/api-keys

List all API keys.

**Response: 200 OK**

```json
{
  "keys": [
    {
      "id": "string (UUID)",
      "name": "string",
      "key_prefix": "string",
      "scopes": ["string"],
      "created_at": "string (RFC 3339)",
      "last_used": "string (RFC 3339) | null",
      "rate_limit": null
    }
  ]
}
```

`key_prefix` is the first 12 characters of the key (e.g. `claud_a1b2c3`).

---

### DELETE /admin/api-keys/:id

Delete an API key by its UUID.

**Response: 200 OK**

```json
{
  "status": "ok"
}
```

---

## Error Responses

| Status | Meaning                                      |
|--------|----------------------------------------------|
| 401    | Missing or invalid `Authorization` token     |
| 403    | Valid key but insufficient scope; or non-localhost request to admin endpoint |
| 429    | Too many failed auth attempts (rate-limited) |
| 4xx    | Client error (malformed request, etc.)       |
| 5xx    | Server error                                 |

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
