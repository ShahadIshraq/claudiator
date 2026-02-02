# Phase 1 — Foundation (Parallel)

5 independent work streams, no cross-dependencies.

---

## Agent A: Cargo Workspace + Core Types

**Files:** `Cargo.toml`, `src/error.rs`, `src/config.rs`, `src/logger.rs`

### `Cargo.toml` (workspace root)

```toml
[workspace]
members = [".", "test-server"]

[package]
name = "claudiator-hook"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "claudiator-hook"
path = "src/main.rs"

[dependencies]
clap = { version = "4", features = ["derive"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
toml = "0.8"
ureq = { version = "2", features = ["json"] }
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1", features = ["v4"] }
dirs = "5"

[profile.release]
opt-level = "z"
lto = true
codegen-units = 1
strip = true
```

### `src/error.rs`

Three error enums:
- `ConfigError` — `NoHomeDir`, `ReadFailed(PathBuf, io::Error)`, `ParseFailed(PathBuf, toml::de::Error)`
- `EventError` — `ParseFailed(serde_json::Error)`
- `SendError` — `Serialize(serde_json::Error)`, `Network(String)`, `ServerError(u16, String)`

All implement `Display`. No `std::error::Error` impl needed (we only format for logging).

### `src/config.rs`

```rust
#[derive(Debug, Deserialize)]
pub struct Config {
    pub server_url: String,
    pub api_key: String,
    pub device_name: String,
    pub device_id: String,
    pub platform: String,
}
```

`Config::load()` → reads `~/.claude/claudiator/config.toml` via `dirs::home_dir()`.

### `src/logger.rs`

`log_error(message: &str)` — appends `[ISO8601] message\n` to `~/.claude/claudiator/error.log`. Silently ignores all IO failures (logging must never crash the hook).

---

## Agent B: Data Model Structs

**Files:** `src/event.rs`, `src/payload.rs`

### `src/event.rs`

```rust
#[derive(Debug, Deserialize, Serialize)]
pub struct HookEvent {
    pub session_id: String,
    pub hook_event_name: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub cwd: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transcript_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permission_mode: Option<String>,

    // Tool-related
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_input: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_output: Option<serde_json::Value>,

    // Notification
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notification_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,

    // UserPromptSubmit / SessionStart
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,

    // SessionEnd / Stop
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,

    // Subagent
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subagent_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subagent_type: Option<String>,

    // Forward-compat: capture unknown fields
    #[serde(flatten)]
    pub extra: std::collections::HashMap<String, serde_json::Value>,
}
```

`HookEvent::from_stdin()` — `serde_json::from_reader(stdin.lock())`

### `src/payload.rs`

```rust
#[derive(Debug, Serialize)]
pub struct DeviceInfo {
    pub device_id: String,
    pub device_name: String,
    pub platform: String,
}

#[derive(Debug, Serialize)]
pub struct EventPayload {
    pub device: DeviceInfo,
    pub event: HookEvent,
    pub timestamp: String,
}
```

`EventPayload::new(config, event)` — builds payload with `chrono::Utc::now().to_rfc3339()`.

---

## Agent C: Installation Scripts

**Files:** `scripts/install.sh`, `scripts/install.ps1`

### `install.sh` Flow

1. Detect OS: `uname -s` → map `Darwin`→`apple-darwin`, `Linux`→`unknown-linux-gnu`
2. Detect arch: `uname -m` → map `arm64`→`aarch64`
3. Build target string: `{arch}-{os}`
4. Download URL: `https://github.com/{OWNER}/{REPO}/releases/latest/download/claudiator-hook-{target}.tar.gz`
5. `mkdir -p ~/.claude/claudiator`
6. Download with `curl -fsSL` → `tar xz -C ~/.claude/claudiator/`
7. `chmod +x ~/.claude/claudiator/claudiator-hook`
8. Prompt: `Server URL:` (read), `API Key:` (read -s for hidden input)
9. `DEVICE_NAME=$(hostname)`, `DEVICE_ID=$(uuidgen 2>/dev/null || cat /proc/sys/kernel/random/uuid)`
10. Map platform: `Darwin`→`mac`, `Linux`→`linux`
11. Write `config.toml` via heredoc
12. Run `~/.claude/claudiator/claudiator-hook test` — if fails, warn but continue
13. Ask: `Auto-configure Claude Code hooks in ~/.claude/settings.json? [Y/n]:`
14. If yes:
    - Check if `jq` is available, else try `python3`, else print manual instructions
    - Read settings.json, merge hooks object (add claudiator entries without removing existing hooks)
    - For each event (SessionStart, SessionEnd, Stop, Notification, UserPromptSubmit): check if claudiator command already exists, skip if so
    - Write back settings.json
15. Print summary box

### `install.ps1` Flow

Same logic, PowerShell equivalents:
- `Invoke-WebRequest -Uri $url -OutFile $zipPath`
- `Expand-Archive`
- `[System.Guid]::NewGuid().ToString()`
- `$env:COMPUTERNAME`
- `ConvertFrom-Json` / `ConvertTo-Json -Depth 10` for settings.json

---

## Agent D: GitHub Actions CI

**Files:** `.github/workflows/release.yml`

### Trigger
```yaml
on:
  push:
    tags: ['v*']
```

### Build Matrix
| target | runner |
|---|---|
| `x86_64-apple-darwin` | `macos-latest` |
| `aarch64-apple-darwin` | `macos-latest` |
| `x86_64-unknown-linux-gnu` | `ubuntu-latest` |
| `aarch64-unknown-linux-gnu` | `ubuntu-latest` |
| `x86_64-pc-windows-msvc` | `windows-latest` |

### Steps (per matrix entry)
1. `actions/checkout@v4`
2. `dtolnay/rust-toolchain@stable` with target
3. For `aarch64-unknown-linux-gnu`: install `gcc-aarch64-linux-gnu`, set `CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER`
4. `cargo build --release --target {target}` (only the `claudiator-hook` package, not test-server)
5. Package: `.tar.gz` (unix) / `.zip` (windows)
6. `actions/upload-artifact@v4`

### Release Job
- `actions/download-artifact@v4` with `merge-multiple: true`
- `softprops/action-gh-release@v2` with all archives

---

## Agent E: Test Server

**Files:** `test-server/Cargo.toml`, `test-server/src/main.rs`

### `test-server/Cargo.toml`

```toml
[package]
name = "test-server"
version = "0.1.0"
edition = "2021"

[dependencies]
axum = "0.7"
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
clap = { version = "4", features = ["derive"] }
chrono = "0.4"
colored = "2"
```

### CLI Args
- `--port` (default: 3000)
- `--api-key` (default: "test-key")

### Endpoints

**`GET /api/v1/ping`**
- Extract `Authorization` header, verify `Bearer {key}`
- 401 → `{"error": "unauthorized", "message": "Invalid or missing API key"}`
- 200 → `{"status": "ok", "server_version": "test-0.1.0"}`

**`POST /api/v1/events`**
- Verify auth (same as ping)
- Parse body as `serde_json::Value`
- Extract and log: device info, session_id, hook_event_name, notification_type (if present), cwd, message
- Log format:
  ```
  [2026-02-02T15:30:00Z] EVENT received
    Device:  shahads-macbook (mac) [550e8400-...]
    Session: abc123-def456
    Event:   Notification (permission_prompt)
    CWD:     /Users/shahad/project
    Message: Claude wants to run a bash command
  ---
  ```
- 200 → `{"status": "ok"}`

### Code Structure

Single file (~120 lines):
- `struct AppState { api_key: String }`
- `async fn check_auth(headers, state) -> Result<(), StatusCode>`
- `async fn ping_handler(headers, state) -> impl IntoResponse`
- `async fn events_handler(headers, state, body: Json<Value>) -> impl IntoResponse`
- `#[tokio::main] async fn main()` — clap parse, build router, `axum::serve`

Shared state via `axum::extract::State<Arc<AppState>>`.
