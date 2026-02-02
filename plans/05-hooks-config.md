# Claude Code Hooks Configuration

How claudiator-hook integrates with Claude Code's hook system.

---

## Hook Events We Register

| Event | Why |
|---|---|
| `SessionStart` | Track when sessions begin/resume across devices |
| `SessionEnd` | Track when sessions terminate |
| `Stop` | Detect when agent is idle, waiting for next user prompt |
| `Notification` | Catch permission prompts, idle prompts, elicitation dialogs — primary trigger for mobile push |
| `UserPromptSubmit` | Track user activity (know a session is active) |

## Events We Skip

| Event | Why Skip |
|---|---|
| `PreToolUse` | Too noisy — fires on every tool call. Not useful for notifications. |
| `PostToolUse` | Same — too frequent. |
| `PostToolUseFailure` | Tool failures don't need mobile notification. |
| `PreCompact` | Internal event, no user-facing value. |
| `SubagentStart/Stop` | Internal orchestration, not actionable. |
| `PermissionRequest` | Already covered by `Notification` with `permission_prompt` type. |

## Settings.json Structure

Added to `~/.claude/settings.json` (merged with existing content):

```json
{
  "hooks": {
    "SessionStart": [
      {
        "matcher": "",
        "hooks": [
          {
            "type": "command",
            "command": "~/.claude/claudiator/claudiator-hook send"
          }
        ]
      }
    ],
    "SessionEnd": [
      {
        "matcher": "",
        "hooks": [
          {
            "type": "command",
            "command": "~/.claude/claudiator/claudiator-hook send"
          }
        ]
      }
    ],
    "Stop": [
      {
        "matcher": "",
        "hooks": [
          {
            "type": "command",
            "command": "~/.claude/claudiator/claudiator-hook send"
          }
        ]
      }
    ],
    "Notification": [
      {
        "matcher": "",
        "hooks": [
          {
            "type": "command",
            "command": "~/.claude/claudiator/claudiator-hook send"
          }
        ]
      }
    ],
    "UserPromptSubmit": [
      {
        "matcher": "",
        "hooks": [
          {
            "type": "command",
            "command": "~/.claude/claudiator/claudiator-hook send"
          }
        ]
      }
    ]
  }
}
```

## How Hooks Are Invoked

1. Claude Code fires the hook event
2. Runs `~/.claude/claudiator/claudiator-hook send` as a shell command
3. Pipes the event JSON to stdin
4. The binary reads stdin, enriches with device metadata, POSTs to server
5. Binary exits 0 (always)

## Important Behaviors

- **Empty matcher (`""`)** catches all sub-events for that hook type. This means:
  - `SessionStart` fires for both `startup` and `resume`
  - `Notification` fires for `permission_prompt`, `idle_prompt`, `elicitation_dialog`, and `auth_success`
  - The server can filter by `notification_type` or `source` from the event payload
- **Exit code 0** means Claude Code treats the hook as successful and continues normally
- **Exit code 2** would block Claude Code — we never do this
- **Stderr output** is captured by Claude Code in verbose mode — we avoid writing to stderr in `send`
- **Hook timeout**: Claude Code has a default hook timeout. Our 3-second HTTP timeout ensures we finish well within it.

## Merge Strategy (Install Script)

When auto-configuring hooks, the install script:
1. Reads existing `~/.claude/settings.json`
2. Creates `hooks` key if absent
3. For each event key: checks if an entry with `claudiator-hook send` already exists in the command
4. If not found, appends a new matcher group to the array
5. Preserves all existing hooks for other events and for the same events (from other tools)
6. Writes back with proper JSON formatting

This is non-destructive — it only adds, never removes or modifies existing hook entries.
