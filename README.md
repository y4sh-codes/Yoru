# Yoru

Yoru is a lightweight Postman-like API client for terminal-first developers.

It ships with:
- Interactive TUI built with `ratatui` + `crossterm`
- Inline request editing and live filtering
- Response tabs for body, headers, script logs, and history
- One-shot request command for CI/scripts
- Workspace/collection/request persistence
- Environment variables with `{{var}}` interpolation
- Auth support: None, Basic, Bearer, API Key
- Request hooks using Rhai scripts (pre-request and tests)
- History tracking with latency and payload metrics
- JSON/YAML import-export

## Why This Architecture

The codebase is modular and scalable:
- `src/core`: domain entities and validation
- `src/storage`: persistence contracts and filesystem backend
- `src/http`: transport, templating, auth, scripting
- `src/app`: state and actions
- `src/tui`: rendering/theme/event loop
- `src/cli`: command parsing and dispatch
- `src/util`: logging/errors/time helpers

## Requirements

- Rust (stable) >= 1.85
- Linux/macOS/Windows terminal with ANSI support

## Run Commands

### 1) Build

```bash
cargo build
```

### 2) Launch TUI (default)

```bash
cargo run
```

or

```bash
cargo run -- tui
```

### 3) Initialize workspace

```bash
cargo run -- init --name "My Workspace"
```

### 4) Send request from CLI

```bash
cargo run -- send \
  --method GET \
  --url "https://httpbin.org/get?svc={{service}}" \
  --env service=yoru \
  --header "Accept:application/json"
```

### 5) Send POST JSON with bearer token

```bash
cargo run -- send \
  --method POST \
  --url "https://httpbin.org/post" \
  --json '{"project":"yoru"}' \
  --bearer "YOUR_TOKEN"
```

### 6) Import/Export workspace

```bash
cargo run -- export --file ./examples/workspace.export.json
cargo run -- import --file ./examples/workspace.export.json
```

## TUI Keybindings

- `Up/Down`: select request
- `Left/Right`: switch collection
- `r` or `Enter`: run selected request
- `e`: cycle active environment
- `n`: quick-add request to selected collection
- `d`: duplicate selected request
- `x`: delete selected request
- `m`: cycle HTTP method
- `/`: open live request filter
- `i`: edit request name
- `u`: edit request URL
- `h`: add header (`Key:Value`)
- `p`: add query param (`key=value`)
- `b`: edit raw request body
- `t`: set/clear bearer token
- `1/2/3/4`: switch response tab (body/headers/logs/history)
- `Tab`: cycle response tabs
- `?`: toggle help dialog
- `c`: clear current error banner
- `q`: quit

## Scripts

Each request supports optional inline scripts:
- `pre_request_script`
- `test_script`

Available script values:
- `vars` map with environment/request variables
- `log("message")` function

Example:

```rhai
if vars["service"] == "yoru" {
  log("Service variable is configured");
}
```

## Lightweight + Production Notes

- Atomic workspace writes (`.tmp` + rename)
- Strict model validation on load/save
- Bounded history for quick startup
- Shared HTTP client with connection pooling
- Clear separation of state/render/transport layers

## Next Enterprise Extensions

- OAuth2 token flows
- GraphQL/gRPC request types
- Advanced editors for request bodies
- Test suites and runner pipelines
- Team sync + encrypted secret store

See [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) and [docs/DOCTAGS.md](docs/DOCTAGS.md).
