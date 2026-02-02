# Claudiator — Project Summary

## What Is This

A system to push Claude Code session events to a central server, enabling mobile notifications when sessions need input or finish. This repo contains:

1. **`claudiator-hook`** — A Rust CLI binary invoked by Claude Code hooks. Reads hook event JSON from stdin, enriches it with device metadata, and POSTs it to a server.
2. **`test-server`** — A local Rust HTTP server for development. Validates API keys, logs incoming events, always returns 200.
3. **Installation scripts** — `install.sh` (macOS/Linux) and `install.ps1` (Windows) that download the binary, configure credentials, and optionally wire up hooks in Claude Code settings.
4. **GitHub Actions CI** — Cross-compiles for 5 targets and publishes GitHub Releases.

## Project Structure

```
claudiator/
├── Cargo.toml                  # Workspace root
├── src/
│   ├── main.rs                 # CLI entry point (send, test, version)
│   ├── cli.rs                  # Clap subcommand definitions
│   ├── config.rs               # Load ~/.claude/claudiator/config.toml
│   ├── event.rs                # Hook event stdin JSON structs
│   ├── payload.rs              # Outbound API payload (device + event + timestamp)
│   ├── sender.rs               # HTTP POST via ureq (3s timeout)
│   ├── error.rs                # Error types
│   └── logger.rs               # Append-only error log
├── test-server/
│   ├── Cargo.toml
│   └── src/
│       └── main.rs             # Axum server: /api/v1/ping + /api/v1/events
├── scripts/
│   ├── install.sh
│   └── install.ps1
├── .github/workflows/
│   └── release.yml
└── plans/                      # This directory (planning docs)
```

## Installed Layout (on user machines)

```
~/.claude/claudiator/
├── claudiator-hook             # Binary
├── config.toml                 # Server URL, API key, device metadata
└── error.log                   # Created on first error
```

## Hook Events Captured

| Event | Meaning | Mobile Notification? |
|---|---|---|
| SessionStart | Session begins or resumes | No (tracking only) |
| SessionEnd | Session terminates | Optional |
| Stop | Agent finished, waiting for next prompt | Yes |
| Notification | Permission prompt, idle, dialog | Yes |
| UserPromptSubmit | User submitted a prompt | No (activity tracking) |

## Implementation Phases

| Phase | What | Parallel? |
|---|---|---|
| [Phase 1](./01-phase1-foundation.md) | Cargo workspace, data types, error handling, config, logger, test server, install scripts, CI | 5 agents in parallel |
| [Phase 2](./02-phase2-integration.md) | HTTP sender, CLI dispatch + main.rs | 2 agents sequential |
| [Phase 3](./03-phase3-verification.md) | End-to-end testing with test server | Manual |

## Key Design Decisions

- **ureq over reqwest**: No async runtime needed. Binary stays ~2MB vs ~8MB. Startup is instant.
- **Always exit 0 in `send`**: Non-zero exits disrupt Claude Code (exit 2 blocks tool calls).
- **3-second HTTP timeout**: Prevents blocking Claude Code if server is unreachable.
- **`#[serde(flatten)]` for unknown fields**: Forward-compatible with future Claude Code hook changes.
- **Errors to log file only**: Never to stderr during `send` (Claude Code captures stderr).
