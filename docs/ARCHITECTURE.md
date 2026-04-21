# Architecture

## Overview

Yoru is built with a layered architecture that keeps business logic independent from interface and storage details.

Flow:
1. CLI/TUI captures user intent
2. App actions transform state and invoke execution
3. HTTP executor interpolates variables, applies auth, runs scripts, sends request
4. Workspace state and history are persisted
5. UI renders updated state

## Module Breakdown

### `src/core`
- `models.rs`: canonical data model for workspace, collections, requests, auth, and responses
- `workspace.rs`: mutation helpers and seed data lifecycle
- `validation.rs`: safety checks for schema consistency

### `src/storage`
- `fs_store.rs`: filesystem persistence, import/export, atomic writes
- `schema.rs`: schema constants

### `src/http`
- `client.rs`: reqwest client builder
- `templating.rs`: `{{variable}}` interpolation
- `auth.rs`: auth application on request builder
- `scripting.rs`: Rhai hook runtime
- `executor.rs`: end-to-end request execution pipeline

### `src/app`
- `state.rs`: UI/runtime state container
- `actions.rs`: user-triggered commands (run request, cycle env)

### `src/tui`
- `events.rs`: input + tick events
- `theme.rs`: visual tokens
- `ui.rs`: render-only widgets and layout
- `mod.rs`: event loop and keybindings

### `src/cli`
- `args.rs`: clap command schema
- `entry.rs`: command dispatch and operational flow

### `src/util`
- `error.rs`: typed application errors
- `logging.rs`: tracing setup
- `time.rs`: timestamp helpers

## Scalability Decisions

- Domain model is serialized once and reused across all interfaces
- Storage interface (`WorkspaceStore`) allows swapping backend (SQLite, remote API) later
- Rendering is stateless and receives immutable state
- Request execution is encapsulated in `HttpExecutor`
- Script hooks are optional and isolated from transport concerns

## Data Path

Default workspace file path:
- Linux: `~/.local/share/yoru/workspace.json`

Override with:
- `--data-dir <path>`
- `YORU_DATA_DIR=<path>`

## Error Handling

- Internal modules return `YoruResult<T>` with typed `YoruError`
- CLI bootstrap uses `color-eyre` for rich terminal diagnostics
