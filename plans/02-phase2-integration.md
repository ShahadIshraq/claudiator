# Phase 2 — Integration (Sequential)

Depends on Phase 1 completion. Two agents, run sequentially since `main.rs` depends on `sender.rs`.

---

## Agent A: HTTP Sender

**Files:** `src/sender.rs`

**Depends on:** `config.rs` (Config struct), `payload.rs` (EventPayload struct), `error.rs` (SendError enum)

### `send_event(config: &Config, payload: &EventPayload) -> Result<(), SendError>`

- Build URL: `{config.server_url}/api/v1/events` (trim trailing slash)
- `ureq::post(&url)` with:
  - `.timeout(Duration::from_secs(3))`
  - `.set("Authorization", &format!("Bearer {}", config.api_key))`
  - `.set("Content-Type", "application/json")`
  - `.set("User-Agent", &format!("claudiator-hook/{}", env!("CARGO_PKG_VERSION")))`
  - `.send_json(serde_json::to_value(payload)?)`
- Match response:
  - 2xx → `Ok(())`
  - Other status → `Err(SendError::ServerError(status, body))`
  - Network error → `Err(SendError::Network(e.to_string()))`

### `test_connection(config: &Config) -> Result<String, SendError>`

- Build URL: `{config.server_url}/api/v1/ping`
- `ureq::get(&url)` with:
  - `.timeout(Duration::from_secs(5))` (slightly longer for interactive use)
  - Same auth and user-agent headers
- Match response:
  - 200 → `Ok(body_string)`
  - Other → `Err(SendError::ServerError(...))`
  - Network error → `Err(SendError::Network(...))`

---

## Agent B: CLI Dispatch + Main

**Files:** `src/cli.rs`, `src/main.rs`

**Depends on:** All other `src/` modules

### `src/cli.rs`

```rust
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "claudiator-hook", about = "Claude Code hook event forwarder")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Read hook event from stdin and send to server
    Send,
    /// Test connection to the configured server
    Test,
    /// Print version information
    Version,
}
```

### `src/main.rs`

Module declarations for all modules, then:

#### `fn main()`
- `Cli::parse()` → match on subcommand → dispatch

#### `fn cmd_send()`
1. `Config::load()` — on error: `log_error(...)`, return (exit 0)
2. `HookEvent::from_stdin()` — on error: `log_error(...)`, return (exit 0)
3. `EventPayload::new(&config, event)`
4. `send_event(&config, &payload)` — on error: `log_error(...)`, return (exit 0)
5. Always returns normally (exit 0)

**Critical: this function must NEVER panic or exit non-zero.**

#### `fn cmd_test()`
1. `Config::load()` — on error: `eprintln!(...)`, `process::exit(1)`
2. Print "Testing connection to {url}..."
3. `test_connection(&config)` — on error: `eprintln!(...)`, `process::exit(1)`
4. Print "Connection successful!" + server response body if non-empty

#### `fn cmd_version()`
- `println!("claudiator-hook v{}", env!("CARGO_PKG_VERSION"))`
