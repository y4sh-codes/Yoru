//! Stateful actions triggered by CLI/TUI events.
//!
//! Doctag:app-actions

use chrono::Utc;

use crate::app::state::AppState;
use crate::core::models::{AuthStrategy, HttpMethod, KeyValue, RequestBody};
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
    let env_name = active_environment.as_ref().map(|env| env.name.clone());

    state.status_line = format!("Running {} {}", request.method, request.url);
    state.last_error = None;

    match executor
        .execute_request(&request, active_environment.as_ref())
        .await
    {
        Ok(response) => {
            state.status_line = format!(
                "{} {} in {} ms ({} bytes)",
                response.status, response.status_text, response.duration_ms, response.size_bytes
            );
            state
                .workspace
                .append_history(&request, &response, env_name);
            state.workspace.updated_at = Utc::now();
            store.save_workspace(&state.workspace)?;
            state.last_response = Some(response);
            Ok(())
        }
        Err(err) => {
            state.last_error = Some(err.to_string());
            state.status_line = "Request failed".to_string();
            Err(err)
        }
    }
}

/// Cycles environment and persists workspace.
pub fn cycle_environment<S: WorkspaceStore>(state: &mut AppState, store: &S) -> YoruResult<()> {
    state.workspace.cycle_environment();
    store.save_workspace(&state.workspace)?;

    let env_name = state
        .workspace
        .active_environment()
        .map(|env| env.name.clone())
        .unwrap_or_else(|| "none".to_string());

    state.status_line = format!("Active environment: {env_name}");
    Ok(())
}

/// Cycles HTTP method for selected request.
pub fn cycle_selected_method<S: WorkspaceStore>(state: &mut AppState, store: &S) -> YoruResult<()> {
    let new_method = {
        let request = state
            .selected_request_mut()
            .ok_or_else(|| YoruError::Runtime("no request selected".to_string()))?;
        request.method = next_method(request.method);
        request.method
    };

    state.workspace.updated_at = Utc::now();
    state.status_line = format!("Method changed to {}", new_method);
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

    let selected = collection
        .requests
        .get(state.selected_request_idx)
        .cloned()
        .ok_or_else(|| YoruError::Runtime("no request selected".to_string()))?;

    let mut cloned = selected;
    cloned.id = uuid::Uuid::new_v4();
    cloned.name = format!("{} Copy", cloned.name);

    collection.requests.insert(state.selected_request_idx + 1, cloned);
    state.selected_request_idx += 1;
    state.workspace.updated_at = Utc::now();
    state.status_line = "Request duplicated".to_string();
    store.save_workspace(&state.workspace)?;

    Ok(())
}

/// Removes selected request while preserving at least one request in workspace.
pub fn delete_selected_request<S: WorkspaceStore>(state: &mut AppState, store: &S) -> YoruResult<()> {
    let collection = state
        .workspace
        .collections
        .get_mut(state.selected_collection_idx)
        .ok_or_else(|| YoruError::Runtime("no collection selected".to_string()))?;

    if collection.requests.len() <= 1 {
        return Err(YoruError::Validation(
            "collection must contain at least one request".to_string(),
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
    state.status_line = format!("Deleted request '{}'", removed.name);
    store.save_workspace(&state.workspace)?;

    Ok(())
}

/// Sets selected request name.
pub fn set_request_name<S: WorkspaceStore>(
    state: &mut AppState,
    name: String,
    store: &S,
) -> YoruResult<()> {
    let request = state
        .selected_request_mut()
        .ok_or_else(|| YoruError::Runtime("no request selected".to_string()))?;

    let name = name.trim();
    if name.is_empty() {
        return Err(YoruError::Validation("request name cannot be empty".to_string()));
    }

    request.name = name.to_string();
    state.workspace.updated_at = Utc::now();
    state.status_line = "Request name updated".to_string();
    store.save_workspace(&state.workspace)?;
    Ok(())
}

/// Sets selected request URL.
pub fn set_request_url<S: WorkspaceStore>(state: &mut AppState, url: String, store: &S) -> YoruResult<()> {
    let request = state
        .selected_request_mut()
        .ok_or_else(|| YoruError::Runtime("no request selected".to_string()))?;

    let url = url.trim();
    if url.is_empty() {
        return Err(YoruError::Validation("URL cannot be empty".to_string()));
    }

    request.url = url.to_string();
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
        return Err(YoruError::Validation(
            "invalid header format, expected Key:Value".to_string(),
        ));
    };

    let request = state
        .selected_request_mut()
        .ok_or_else(|| YoruError::Runtime("no request selected".to_string()))?;

    request.headers.push(KeyValue::enabled(key.trim(), value.trim()));
    state.workspace.updated_at = Utc::now();
    state.status_line = "Header added".to_string();
    store.save_workspace(&state.workspace)?;
    Ok(())
}

/// Adds query key-value pair to selected request.
pub fn add_request_query<S: WorkspaceStore>(
    state: &mut AppState,
    query: String,
    store: &S,
) -> YoruResult<()> {
    let Some((key, value)) = query.split_once('=') else {
        return Err(YoruError::Validation(
            "invalid query format, expected key=value".to_string(),
        ));
    };

    let request = state
        .selected_request_mut()
        .ok_or_else(|| YoruError::Runtime("no request selected".to_string()))?;

    request.query.push(KeyValue::enabled(key.trim(), value.trim()));
    state.workspace.updated_at = Utc::now();
    state.status_line = "Query parameter added".to_string();
    store.save_workspace(&state.workspace)?;
    Ok(())
}

/// Sets selected request raw text body.
pub fn set_request_raw_body<S: WorkspaceStore>(
    state: &mut AppState,
    body: String,
    store: &S,
) -> YoruResult<()> {
    let request = state
        .selected_request_mut()
        .ok_or_else(|| YoruError::Runtime("no request selected".to_string()))?;

    if body.trim().is_empty() {
        request.body = RequestBody::None;
        state.status_line = "Request body cleared".to_string();
    } else {
        request.body = RequestBody::Raw {
            mime_type: "application/json".to_string(),
            content: body,
        };
        state.status_line = "Raw body updated".to_string();
    }

    state.workspace.updated_at = Utc::now();
    store.save_workspace(&state.workspace)?;
    Ok(())
}

/// Sets selected request bearer auth token, empty value clears auth.
pub fn set_request_bearer<S: WorkspaceStore>(
    state: &mut AppState,
    token: String,
    store: &S,
) -> YoruResult<()> {
    let request = state
        .selected_request_mut()
        .ok_or_else(|| YoruError::Runtime("no request selected".to_string()))?;

    let token = token.trim();
    if token.is_empty() {
        request.auth = AuthStrategy::None;
        state.status_line = "Auth cleared".to_string();
    } else {
        request.auth = AuthStrategy::Bearer {
            token: token.to_string(),
        };
        state.status_line = "Bearer token updated".to_string();
    }

    state.workspace.updated_at = Utc::now();
    store.save_workspace(&state.workspace)?;
    Ok(())
}

fn next_method(method: HttpMethod) -> HttpMethod {
    match method {
        HttpMethod::Get => HttpMethod::Post,
        HttpMethod::Post => HttpMethod::Put,
        HttpMethod::Put => HttpMethod::Patch,
        HttpMethod::Patch => HttpMethod::Delete,
        HttpMethod::Delete => HttpMethod::Head,
        HttpMethod::Head => HttpMethod::Options,
        HttpMethod::Options => HttpMethod::Get,
    }
}
