<div align="center">

**Terminal API Client · Postman for the Shell**

![Rust](https://img.shields.io/badge/Rust-1.85%2B-orange?logo=rust)
![License](https://img.shields.io/badge/license-MIT-blue)
![TUI](https://img.shields.io/badge/TUI-ratatui-cyan)

</div>

---

![Yoru homescreen](assets/Screenshot%20from%202026-04-22%2007-27-29.png)

Yoru is a full-featured, keyboard-driven HTTP API client that lives entirely in your terminal. It brings the core power of Postman — collections, environments, auth, body editing, history, scripting — to developers who live in the shell.

## Features

| Feature | Details |
|---|---|
| **Interactive TUI** | Built with `ratatui` + `crossterm` — renders in any modern terminal |
| **Request Collections** | Organise requests into named collections; create and rename on the fly |
| **Method Badges** | Colour-coded `GET` / `POST` / `PUT` / `PATCH` / `DELETE` labels in the navigator |
| **Authentication** | None · Bearer token · HTTP Basic · API Key (header or query) |
| **Body Types** | Raw, JSON, Form URL-encoded |
| **Environment Variables** | Named environments with `{{var}}` interpolation; cycle with `e` |
| **Response Viewer** | Status (coloured by code range) · Headers · Body · Script logs · History |
| **Response Scrolling** | `PgUp` / `PgDn` to scroll long response bodies |
| **Request History** | Last 500 executions with latency, size, status — filterable per collection |
| **Live Filter** | Press `/` to fuzzy-filter requests by name, URL, or method |
| **Inline Scripts** | Per-request pre-request and test scripts via `Rhai` |
| **CLI One-Shot** | `yoru send` for scripts and CI pipelines |
| **Import / Export** | JSON and YAML workspace snapshots |
| **Atomic Saves** | Workspace writes use `.tmp` + rename — never corrupt on crash |

---

## Requirements

- **Rust** (stable) ≥ 1.85
- A terminal with **ANSI / 24-bit colour** support (Kitty, Alacritty, iTerm2, Windows Terminal, etc.)

---

## Building & Running

```bash
# Build release binary
cargo build --release

# Launch the TUI (default command)
cargo run

# Or after build:
./target/release/yoru
```

---

## TUI Keybindings

### Navigation
| Key | Action |
|-----|--------|
| `↑` / `↓` | Move between requests |
| `←` / `→` | Switch collection |
| `/` | Open live request filter |

### Requests
| Key | Action |
|-----|--------|
| `r` / `Enter` | Run selected request |
| `n` | Add quick request to current collection |
| `d` | Duplicate selected request |
| `x` | Delete selected request |
| `m` | Cycle HTTP method `GET→POST→PUT→PATCH→DELETE→HEAD→OPTIONS` |

### Editing
| Key | Action |
|-----|--------|
| `i` | Edit request name |
| `u` | Edit URL |
| `h` | Add header — `Key:Value` |
| `p` | Add query parameter — `key=value` |
| `b` | Edit raw request body |
| `T` | Set timeout in ms (empty = default) |

### Authentication
| Key | Action |
|-----|--------|
| `t` | Set bearer token (empty clears auth) |
| `a` | Set Basic auth — `username:password` |
| `k` | Set API key — `name:value` or `name:value:h` (header) / `name:value:q` (query param) |

### Collections
| Key | Action |
|-----|--------|
| `N` | Create new collection (prompts for name) |
| `C` | Rename current collection |
| `e` | Cycle active environment |

### Response
| Key | Action |
|-----|--------|
| `1` / `2` / `3` / `4` | Switch to Body / Headers / Logs / History tab |
| `Tab` | Cycle response tabs |
| `PgDn` / `PgUp` | Scroll response body down / up |

### Other
| Key | Action |
|-----|--------|
| `?` | Toggle help overlay (also closed by `Esc`) |
| `c` / `Esc` | Clear error / close overlays |
| `q` | Quit |

---

## CLI Commands

### One-shot request
```bash
# Simple GET
yoru send --method GET --url "https://httpbin.org/get"

# POST with JSON body and bearer auth
yoru send \
  --method POST \
  --url "https://api.example.com/users" \
  --json '{"name":"Alice","role":"admin"}' \
  --bearer "YOUR_TOKEN_HERE"

# Using environment variables for interpolation
yoru send \
  --method GET \
  --url "https://httpbin.org/get?svc={{service}}" \
  --env service=yoru \
  --header "Accept:application/json"

# Basic auth
yoru send --method GET --url "https://httpbin.org/basic-auth/user/pass" \
  --basic-user user --basic-password pass

# API key in header
yoru send --method GET --url "https://api.example.com/data" \
  --api-key "X-API-Key=abc123"
```

### Workspace management
```bash
# Initialise a fresh workspace
yoru init --name "My Project"

# Export to JSON / YAML
yoru export --file ./backup/workspace.json
yoru export --file ./backup/workspace.yaml

# Import from JSON / YAML
yoru import --file ./backup/workspace.json
```

---

## Inline Scripts (Rhai)

Each request supports optional `pre_request_script` and `test_script` fields in the workspace YAML.

### Available context
- `vars` — map containing all environment + request variables
- `log("message")` — emits a line visible in the **Logs** response tab
- `status` — HTTP status code string (test script only)
- `response_body` — raw response body string (test script only)

### Example
```rhai
// pre_request_script
if vars["env"] == "prod" {
    log("⚠ Targeting production!");
}

// test_script
if status != "200" {
    log("Unexpected status: " + status);
}
```

---

## Workspace File Format

Yoru stores everything in a single YAML (or JSON) workspace file, typically at:

```
~/.local/share/yoru/workspace.yaml   # Linux / macOS
%APPDATA%\yoru\workspace.yaml        # Windows
```

### Sample structure
```yaml
name: My Workspace
collections:
  - name: Auth API
    requests:
      - name: Login
        method: POST
        url: "{{base_url}}/auth/login"
        body:
          kind: json
          value:
            email: "{{user_email}}"
            password: "{{user_password}}"
        auth:
          kind: none
environments:
  - name: local
    variables:
      - key: base_url
        value: http://localhost:3000
      - key: user_email
        value: dev@example.com
```

---

## Architecture

```
src/
├── core/          Domain models & validation (Workspace, Collection, Request, Environment…)
├── storage/       Persistence contracts + filesystem backend (atomic writes, import/export)
├── http/          Transport layer: executor, auth, templating, Rhai scripting
├── app/           State machine + all stateful actions (run, edit, auth, collection CRUD…)
├── tui/           Ratatui rendering, theme, event loop
│   ├── theme.rs   Colour palette with method & status code styles
│   ├── ui.rs      All widget draw functions (splash, navigator, inspector, overlays)
│   ├── mod.rs     Key event handling & TUI runtime loop
│   └── events.rs  Crossterm event polling
├── cli/           Clap argument parsing & `send` command
└── util/          Error types, logging, time helpers
```

---

## Roadmap

- [ ] OAuth 2 token flows (client credentials, auth code)
- [ ] GraphQL request type with query/variables editor
- [ ] gRPC request type
- [ ] Multi-select request runner (run all in collection)
- [ ] Test suite reporting with pass/fail counts
- [ ] Response body syntax highlighting
- [ ] Team sync + encrypted secret store
- [ ] TUI environment variable editor
- [ ] Import from Postman / Insomnia collection JSON

---

## License

MIT — see [LICENSE](LICENSE).
