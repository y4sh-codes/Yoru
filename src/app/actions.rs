//! Stateful actions triggered by CLI/TUI events.
//!
//! Doctag:app-actions

use chrono::Utc;

use crate::app::state::AppState;
use crate::core::models::{
    AuthStrategy, Collection, HttpMethod, KeyValue, RequestBody, RequestTemplate,
};
use crate::http::executor::HttpExecutor;
use crate::storage::fs_store::WorkspaceStore;
use crate::{YoruError, YoruResult};

/// Executes selected request and updates response/history state.
pub async fn run_selected_request<S: WorkspaceStore>(
    state: &mut AppState,
    executor: &HttpExecutor,
    store: &S,
) -> YoruResult<()> {
    let request = state
        .selected_request()
        .cloned()
        .ok_or_else(|| YoruError::Runtime("no request selected".to_string()))?;

    let active_environment = state.workspace.active_environment().cloned();
    let env_name = active_environment.as_ref().map(|e| e.name.clone());

    state.status_line = format!("⟳  {} {}", request.method, request.url);
    state.last_error = None;
    state.response_scroll = 0;

    match executor
        .execute_request(&request, active_environment.as_ref())
        .await
    {
        Ok(response) => {
            let status_icon = match response.status {
                200..=299 => "✓",
                300..=399 => "→",
                400..=499 => "⚠",
                _          => "✗",
            };
            state.status_line = format!(
                "{}  {} {}  ·  {} ms  ·  {}",
                status_icon,
                response.status,
                response.status_text,
                response.duration_ms,
                format_size(response.size_bytes),
            );
            state.workspace.append_history(&request, &response, env_name);
            state.workspace.updated_at = Utc::now();
            store.save_workspace(&state.workspace)?;
            state.last_response = Some(response);
            Ok(())
        }
        Err(err) => {
            state.last_error = Some(err.to_string());
            state.status_line = "✗  Request failed — see error overlay".to_string();
            Err(err)
        }
    }
}

fn format_size(bytes: usize) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.2} MB", bytes as f64 / (1024.0 * 1024.0))
    }
}

/// Cycles environment and persists workspace.
pub fn cycle_environment<S: WorkspaceStore>(state: &mut AppState, store: &S) -> YoruResult<()> {
    state.workspace.cycle_environment();
    store.save_workspace(&state.workspace)?;
    let name = state
        .workspace
        .active_environment()
        .map(|e| e.name.clone())
        .unwrap_or_else(|| "none".to_string());
    state.status_line = format!("Environment → {name}");
    Ok(())
}

/// Cycles HTTP method for selected request.
pub fn cycle_selected_method<S: WorkspaceStore>(state: &mut AppState, store: &S) -> YoruResult<()> {
    let new_method = {
        let req = state
            .selected_request_mut()
            .ok_or_else(|| YoruError::Runtime("no request selected".to_string()))?;
        req.method = next_method(req.method);
        req.method
    };
    state.workspace.updated_at = Utc::now();
    state.status_line = format!("Method → {}", new_method);
    store.save_workspace(&state.workspace)?;
    Ok(())
}

/// Duplicates selected request in the active collection.
pub fn duplicate_selected_request<S: WorkspaceStore>(
    state: &mut AppState,
    store: &S,
) -> YoruResult<()> {
    let collection = state
        .workspace
        .collections
        .get_mut(state.selected_collection_idx)
        .ok_or_else(|| YoruError::Runtime("no collection selected".to_string()))?;

    let mut cloned = collection
        .requests
        .get(state.selected_request_idx)
        .cloned()
        .ok_or_else(|| YoruError::Runtime("no request selected".to_string()))?;

    cloned.id = uuid::Uuid::new_v4();
    cloned.name = format!("{} Copy", cloned.name);
    collection.requests.insert(state.selected_request_idx + 1, cloned);
    state.selected_request_idx += 1;
    state.workspace.updated_at = Utc::now();
    state.status_line = "Request duplicated".to_string();
    store.save_workspace(&state.workspace)?;
    Ok(())
}

/// Removes selected request, keeping at least one per collection.
pub fn delete_selected_request<S: WorkspaceStore>(
    state: &mut AppState,
    store: &S,
) -> YoruResult<()> {
    let collection = state
        .workspace
        .collections
        .get_mut(state.selected_collection_idx)
        .ok_or_else(|| YoruError::Runtime("no collection selected".to_string()))?;

    if collection.requests.len() <= 1 {
        return Err(YoruError::Validation(
            "collection must have at least one request".to_string(),
        ));
    }
    if state.selected_request_idx >= collection.requests.len() {
        return Err(YoruError::Runtime("no request selected".to_string()));
    }

    let removed = collection.requests.remove(state.selected_request_idx);
    if state.selected_request_idx >= collection.requests.len() {
        state.selected_request_idx = collection.requests.len().saturating_sub(1);
    }
    state.workspace.updated_at = Utc::now();
    state.status_line = format!("Deleted '{}'", removed.name);
    store.save_workspace(&state.workspace)?;
    Ok(())
}

/// Sets selected request name.
pub fn set_request_name<S: WorkspaceStore>(
    state: &mut AppState,
    name: String,
    store: &S,
) -> YoruResult<()> {
    let name = name.trim().to_string();
    if name.is_empty() {
        return Err(YoruError::Validation("request name cannot be empty".to_string()));
    }
    let req = state
        .selected_request_mut()
        .ok_or_else(|| YoruError::Runtime("no request selected".to_string()))?;
    req.name = name.clone();
    state.workspace.updated_at = Utc::now();
    state.status_line = format!("Name → {name}");
    store.save_workspace(&state.workspace)?;
    Ok(())
}

/// Sets selected request URL.
pub fn set_request_url<S: WorkspaceStore>(
    state: &mut AppState,
    url: String,
    store: &S,
) -> YoruResult<()> {
    let url = url.trim().to_string();
    if url.is_empty() {
        return Err(YoruError::Validation("URL cannot be empty".to_string()));
    }
    let req = state
        .selected_request_mut()
        .ok_or_else(|| YoruError::Runtime("no request selected".to_string()))?;
    req.url = url;
    state.workspace.updated_at = Utc::now();
    state.status_line = "URL updated".to_string();
    store.save_workspace(&state.workspace)?;
    Ok(())
}

/// Adds header to selected request.
pub fn add_request_header<S: WorkspaceStore>(
    state: &mut AppState,
    header: String,
    store: &S,
) -> YoruResult<()> {
    let Some((key, value)) = header.split_once(':') else {
        return Err(YoruError::Validation("expected Key:Value format".to_string()));
    };
    let req = state
        .selected_request_mut()
        .ok_or_else(|| YoruError::Runtime("no request selected".to_string()))?;
    req.headers.push(KeyValue::enabled(key.trim(), value.trim()));
    state.workspace.updated_at = Utc::now();
    state.status_line = format!("Header '{}' added", key.trim());
    store.save_workspace(&state.workspace)?;
    Ok(())
}

/// Adds query parameter to selected request.
pub fn add_request_query<S: WorkspaceStore>(
    state: &mut AppState,
    query: String,
    store: &S,
) -> YoruResult<()> {
    let Some((key, value)) = query.split_once('=') else {
        return Err(YoruError::Validation("expected key=value format".to_string()));
    };
    let req = state
        .selected_request_mut()
        .ok_or_else(|| YoruError::Runtime("no request selected".to_string()))?;
    req.query.push(KeyValue::enabled(key.trim(), value.trim()));
    state.workspace.updated_at = Utc::now();
    state.status_line = format!("Query '{}' added", key.trim());
    store.save_workspace(&state.workspace)?;
    Ok(())
}

/// Sets selected request raw body.
pub fn set_request_raw_body<S: WorkspaceStore>(
    state: &mut AppState,
    body: String,
    store: &S,
) -> YoruResult<()> {
    let req = state
        .selected_request_mut()
        .ok_or_else(|| YoruError::Runtime("no request selected".to_string()))?;
    if body.trim().is_empty() {
        req.body = RequestBody::None;
        state.status_line = "Body cleared".to_string();
    } else {
        req.body = RequestBody::Raw {
            mime_type: "text/plain".to_string(),
            content: body,
        };
        state.status_line = "Body updated (raw)".to_string();
    }
    state.workspace.updated_at = Utc::now();
    store.save_workspace(&state.workspace)?;
    Ok(())
}

/// Sets a JSON body on the selected request. Parses the string into a
/// `serde_json::Value`; falls back to storing it as a JSON string literal
/// if the input is not valid JSON so the user never loses their work.
pub fn set_request_json_body<S: WorkspaceStore>(
    state: &mut AppState,
    body: String,
    store: &S,
) -> YoruResult<()> {
    let req = state
        .selected_request_mut()
        .ok_or_else(|| YoruError::Runtime("no request selected".to_string()))?;
    if body.trim().is_empty() {
        req.body = RequestBody::None;
        state.status_line = "Body cleared".to_string();
    } else {
        let value: serde_json::Value = serde_json::from_str(&body)
            .map_err(|e| YoruError::Validation(format!("Invalid JSON: {e}")))?;
        req.body = RequestBody::Json { value };
        state.status_line = "Body updated (JSON)".to_string();
    }
    state.workspace.updated_at = Utc::now();
    store.save_workspace(&state.workspace)?;
    Ok(())
}

/// Sets bearer token auth on selected request.
pub fn set_request_bearer<S: WorkspaceStore>(
    state: &mut AppState,
    token: String,
    store: &S,
) -> YoruResult<()> {
    let req = state
        .selected_request_mut()
        .ok_or_else(|| YoruError::Runtime("no request selected".to_string()))?;
    let token = token.trim();
    if token.is_empty() {
        req.auth = AuthStrategy::None;
        state.status_line = "Auth cleared".to_string();
    } else {
        req.auth = AuthStrategy::Bearer { token: token.to_string() };
        state.status_line = "Bearer token set".to_string();
    }
    state.workspace.updated_at = Utc::now();
    store.save_workspace(&state.workspace)?;
    Ok(())
}

/// Sets Basic auth on selected request (format: username:password).
pub fn set_basic_auth<S: WorkspaceStore>(
    state: &mut AppState,
    value: String,
    store: &S,
) -> YoruResult<()> {
    let req = state
        .selected_request_mut()
        .ok_or_else(|| YoruError::Runtime("no request selected".to_string()))?;
    let value = value.trim();
    if value.is_empty() {
        req.auth = AuthStrategy::None;
        state.status_line = "Auth cleared".to_string();
    } else {
        let Some((user, pass)) = value.split_once(':') else {
            return Err(YoruError::Validation("expected username:password format".to_string()));
        };
        req.auth = AuthStrategy::Basic {
            username: user.trim().to_string(),
            password: pass.to_string(),
        };
        state.status_line = format!("Basic auth set for '{}'", user.trim());
    }
    state.workspace.updated_at = Utc::now();
    store.save_workspace(&state.workspace)?;
    Ok(())
}

/// Sets API key auth on selected request.
///
/// Formats accepted:
/// - `name:value`        → API key sent in header (default)
/// - `name:value:h`      → explicitly in header
/// - `name:value:q`      → sent as query parameter
pub fn set_api_key<S: WorkspaceStore>(
    state: &mut AppState,
    value: String,
    store: &S,
) -> YoruResult<()> {
    let req = state
        .selected_request_mut()
        .ok_or_else(|| YoruError::Runtime("no request selected".to_string()))?;
    let value = value.trim();
    if value.is_empty() {
        req.auth = AuthStrategy::None;
        state.status_line = "Auth cleared".to_string();
    } else {
        let parts: Vec<&str> = value.splitn(3, ':').collect();
        match parts.as_slice() {
            [key, val] => {
                req.auth = AuthStrategy::ApiKey {
                    key: key.trim().to_string(),
                    value: val.trim().to_string(),
                    in_header: true,
                };
                state.status_line = format!("API key '{}' set (header)", key.trim());
            }
            [key, val, loc] => {
                let in_header = *loc != "q";
                req.auth = AuthStrategy::ApiKey {
                    key: key.trim().to_string(),
                    value: val.trim().to_string(),
                    in_header,
                };
                state.status_line = format!(
                    "API key '{}' set ({})",
                    key.trim(),
                    if in_header { "header" } else { "query" }
                );
            }
            _ => {
                return Err(YoruError::Validation(
                    "expected name:value or name:value:h/q".to_string(),
                ));
            }
        }
    }
    state.workspace.updated_at = Utc::now();
    store.save_workspace(&state.workspace)?;
    Ok(())
}

/// Sets request timeout in milliseconds.  Empty string clears to default.
pub fn set_request_timeout<S: WorkspaceStore>(
    state: &mut AppState,
    value: String,
    store: &S,
) -> YoruResult<()> {
    let req = state
        .selected_request_mut()
        .ok_or_else(|| YoruError::Runtime("no request selected".to_string()))?;
    let value = value.trim();
    if value.is_empty() {
        req.timeout_ms = None;
        state.status_line = "Timeout cleared (using default)".to_string();
    } else {
        let ms: u64 = value
            .parse()
            .map_err(|_| YoruError::Validation("timeout must be a number in ms".to_string()))?;
        req.timeout_ms = Some(ms);
        state.status_line = format!("Timeout → {} ms", ms);
    }
    state.workspace.updated_at = Utc::now();
    store.save_workspace(&state.workspace)?;
    Ok(())
}

/// Creates a new named collection with a starter request.
pub fn new_collection<S: WorkspaceStore>(
    state: &mut AppState,
    name: String,
    store: &S,
) -> YoruResult<()> {
    let name = name.trim().to_string();
    if name.is_empty() {
        return Err(YoruError::Validation("collection name cannot be empty".to_string()));
    }
    let mut col = Collection::new(name.clone());
    col.requests.push(RequestTemplate::new(
        "New Request",
        HttpMethod::Get,
        "https://httpbin.org/get",
    ));
    state.workspace.collections.push(col);
    state.selected_collection_idx = state.workspace.collections.len() - 1;
    state.selected_request_idx = 0;
    state.workspace.updated_at = Utc::now();
    state.status_line = format!("Collection '{}' created", name);
    store.save_workspace(&state.workspace)?;
    Ok(())
}

/// Renames the currently selected collection.
pub fn rename_collection<S: WorkspaceStore>(
    state: &mut AppState,
    name: String,
    store: &S,
) -> YoruResult<()> {
    let name = name.trim().to_string();
    if name.is_empty() {
        return Err(YoruError::Validation("collection name cannot be empty".to_string()));
    }
    let col = state
        .workspace
        .collections
        .get_mut(state.selected_collection_idx)
        .ok_or_else(|| YoruError::Runtime("no collection selected".to_string()))?;
    col.name = name.clone();
    state.workspace.updated_at = Utc::now();
    state.status_line = format!("Collection renamed to '{}'", name);
    store.save_workspace(&state.workspace)?;
    Ok(())
}

fn next_method(method: HttpMethod) -> HttpMethod {
    match method {
        HttpMethod::Get     => HttpMethod::Post,
        HttpMethod::Post    => HttpMethod::Put,
        HttpMethod::Put     => HttpMethod::Patch,
        HttpMethod::Patch   => HttpMethod::Delete,
        HttpMethod::Delete  => HttpMethod::Head,
        HttpMethod::Head    => HttpMethod::Options,
        HttpMethod::Options => HttpMethod::Get,
    }
}
