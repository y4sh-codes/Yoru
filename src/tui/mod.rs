//! Ratatui-powered interactive UI runtime.
//!
//! Doctag:tui-runtime

use std::io::{self, Stdout};

use crossterm::event::{KeyCode, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;

use crate::app::actions::{
    add_request_header, add_request_query, cycle_environment, cycle_selected_method,
    delete_selected_request, duplicate_selected_request, new_collection, rename_collection,
    run_selected_request, set_api_key, set_basic_auth, set_request_bearer,
    set_request_name, set_request_raw_body, set_request_timeout, set_request_url,
};
use crate::app::state::{
    AppState, ConfirmAction, InputMode, ResponseTab, Screen, SplashInputMode,
};
use crate::core::models::{AuthStrategy, HttpMethod, RequestBody, RequestTemplate};
use crate::http::executor::HttpExecutor;
use crate::storage::fs_store::{FsWorkspaceStore, WorkspaceRegistry, WorkspaceStore};
use crate::tui::events::{AppEvent, EventHandler};
use crate::tui::theme::Theme;
use crate::tui::ui::draw;
use crate::{YoruError, YoruResult};

pub mod events;
pub mod theme;
pub mod ui;

/// Runs the interactive terminal application.
pub async fn run_tui(
    mut state: AppState,
    executor: HttpExecutor,
    store: &FsWorkspaceStore,
) -> YoruResult<()> {
    let _guard = TerminalGuard::enter()?;
    let mut terminal = create_terminal()?;
    let events = EventHandler::default();
    let theme = Theme::default();

    while !state.should_quit {
        terminal
            .draw(|frame| draw(frame, &state, &theme))
            .map_err(|err| YoruError::Runtime(format!("terminal draw failure: {err}")))?;

        match events.next()? {
            AppEvent::Tick => {}
            AppEvent::Input(key) if key.kind == KeyEventKind::Press => {
                if let Err(err) = handle_key_event(&mut state, &executor, store, key.code).await {
                    state.last_error = Some(err.to_string());
                }
            }
            AppEvent::Input(_) => {}
        }
    }

    Ok(())
}

async fn handle_key_event(
    state: &mut AppState,
    executor: &HttpExecutor,
    store: &FsWorkspaceStore,
    key_code: KeyCode,
) -> YoruResult<()> {
    match state.screen {
        Screen::Splash => handle_splash_key(state, store, key_code),
        Screen::Main   => handle_main_key(state, executor, store, key_code).await,
    }
}

// ─── Splash key handling ──────────────────────────────────────────────────────

fn handle_splash_key(
    state: &mut AppState,
    store: &FsWorkspaceStore,
    key_code: KeyCode,
) -> YoruResult<()> {
    // ── Confirm-delete dialog is open ─────────────────────────────────────────
    if let Some(ConfirmAction::DeleteWorkspace(slug)) = state.splash_confirm.clone() {
        match key_code {
            KeyCode::Char('y') | KeyCode::Enter => {
                // Must keep at least one workspace
                if state.available_workspaces.len() <= 1 {
                    state.splash_confirm = None;
                    return Err(YoruError::Validation(
                        "cannot delete the only workspace".to_string(),
                    ));
                }
                store.delete_workspace_by_slug(&slug)?;
                // Refresh list
                state.available_workspaces = store.list_workspaces()?;
                state.splash_selected_idx =
                    state.splash_selected_idx.min(state.available_workspaces.len().saturating_sub(1));
                // If we deleted the active one, switch to first available
                if state.active_slug == slug {
                    if let Some(first) = state.available_workspaces.first() {
                        let ws = store.load_workspace_by_slug(&first.slug.clone())?;
                        state.workspace    = ws;
                        state.active_slug  = first.slug.clone();
                    }
                }
                state.splash_confirm = None;
            }
            KeyCode::Char('n') | KeyCode::Esc => {
                state.splash_confirm = None;
            }
            _ => {}
        }
        return Ok(());
    }

    // ── Splash text input (new / rename) ──────────────────────────────────────
    if state.splash_input_mode != SplashInputMode::None {
        match key_code {
            KeyCode::Esc => {
                state.end_splash_input();
            }
            KeyCode::Enter => {
                let value = state.splash_input_buffer.trim().to_string();
                match state.splash_input_mode {
                    SplashInputMode::NewWorkspace => {
                        let (ws, slug) = store.create_workspace(&value)?;
                        state.workspace    = ws;
                        state.active_slug  = slug;
                        state.available_workspaces = store.list_workspaces()?;
                        state.splash_selected_idx  =
                            state.available_workspaces.iter().position(|e| e.slug == state.active_slug)
                                .unwrap_or(0);
                    }
                    SplashInputMode::RenameWorkspace => {
                        if let Some(entry) = state.available_workspaces.get(state.splash_selected_idx).cloned() {
                            let new_slug = store.rename_workspace(&entry.slug, &value)?;
                            if state.active_slug == entry.slug {
                                state.active_slug = new_slug.clone();
                            }
                            state.available_workspaces = store.list_workspaces()?;
                            state.splash_selected_idx  =
                                state.available_workspaces.iter().position(|e| e.slug == new_slug)
                                    .unwrap_or(0);
                        }
                    }
                    SplashInputMode::None => {}
                }
                state.end_splash_input();
            }
            KeyCode::Backspace => { state.splash_input_buffer.pop(); }
            KeyCode::Char(ch)  => { state.splash_input_buffer.push(ch); }
            _ => {}
        }
        return Ok(());
    }

    // ── Normal splash navigation ──────────────────────────────────────────────
    match key_code {
        KeyCode::Char('q') => state.should_quit = true,

        KeyCode::Up   => state.splash_prev(),
        KeyCode::Down => state.splash_next(),

        // Open selected workspace
        KeyCode::Enter => {
            if let Some(entry) = state.splash_selected_entry().cloned() {
                let ws = store.load_workspace_by_slug(&entry.slug)?;
                state.workspace   = ws;
                state.active_slug = entry.slug;
                state.selected_collection_idx = 0;
                state.selected_request_idx    = 0;
                state.normalize_selection();
                state.screen = Screen::Main;
            }
        }

        // New workspace
        KeyCode::Char('n') => {
            state.begin_splash_input(SplashInputMode::NewWorkspace, "");
        }

        // Rename selected workspace
        KeyCode::Char('r') => {
            if let Some(entry) = state.splash_selected_entry() {
                let current = entry.display_name.clone();
                state.begin_splash_input(SplashInputMode::RenameWorkspace, current);
            }
        }

        // Delete selected workspace (with confirm)
        KeyCode::Char('x') | KeyCode::Char('d') => {
            if let Some(entry) = state.splash_selected_entry() {
                if state.available_workspaces.len() <= 1 {
                    return Err(YoruError::Validation(
                        "cannot delete the only workspace".to_string(),
                    ));
                }
                state.splash_confirm = Some(ConfirmAction::DeleteWorkspace(entry.slug.clone()));
            }
        }

        // Duplicate selected workspace
        KeyCode::Char('D') => {
            if let Some(entry) = state.splash_selected_entry().cloned() {
                let ws = store.load_workspace_by_slug(&entry.slug)?;
                let new_name = format!("{} Copy", ws.name);
                let (_, new_slug) = store.create_workspace(&new_name)?;
                // Save the duplicated workspace content under new slug
                store.save_workspace_with_slug(&ws, &new_slug)?;
                state.available_workspaces = store.list_workspaces()?;
                state.splash_selected_idx  =
                    state.available_workspaces.iter().position(|e| e.slug == new_slug)
                        .unwrap_or(state.splash_selected_idx);
            }
        }

        // Help (on splash just show a brief hint via status; full help only on main screen)
        KeyCode::Char('?') => {}

        _ => {}
    }

    Ok(())
}

// ─── Main key handling ────────────────────────────────────────────────────────

async fn handle_main_key(
    state: &mut AppState,
    executor: &HttpExecutor,
    store: &FsWorkspaceStore,
    key_code: KeyCode,
) -> YoruResult<()> {
    // Error overlay
    if state.last_error.is_some() && key_code == KeyCode::Esc {
        state.last_error = None;
        return Ok(());
    }

    // Help overlay — absorbs all keys; only Esc / ? closes it
    if state.show_help {
        match key_code {
            KeyCode::Esc | KeyCode::Char('?') | KeyCode::Char('q') => {
                state.show_help = false;
            }
            _ => {}
        }
        return Ok(());
    }

    // Inline input mode
    if state.input_mode != InputMode::None {
        return handle_inline_input_key(state, store, key_code);
    }

    match key_code {
        KeyCode::Char('q') => state.should_quit = true,
        KeyCode::Char('?') => state.show_help = true,
        KeyCode::Char('c') | KeyCode::Esc => state.last_error = None,

        // Go back to workspace picker
        KeyCode::Char('W') => {
            state.available_workspaces = store.list_workspaces()?;
            state.splash_selected_idx  =
                state.available_workspaces.iter().position(|e| e.slug == state.active_slug)
                    .unwrap_or(0);
            state.screen = Screen::Splash;
        }

        // Run request
        KeyCode::Char('r') | KeyCode::Enter => {
            run_selected_request(state, executor, store).await?;
            state.response_tab = ResponseTab::Body;
        }

        // Collection management
        KeyCode::Char('N') => state.begin_input(InputMode::NewCollection, ""),
        KeyCode::Char('C') => {
            let initial = state.selected_collection().map(|c| c.name.clone()).unwrap_or_default();
            state.begin_input(InputMode::EditCollectionName, initial);
        }

        // Environment
        KeyCode::Char('e') => cycle_environment(state, store)?,

        // Request management
        KeyCode::Char('n') => {
            add_quick_request(state);
            store.save_workspace(&state.workspace)?;
        }
        KeyCode::Char('d') => duplicate_selected_request(state, store)?,
        KeyCode::Char('x') => delete_selected_request(state, store)?,
        KeyCode::Char('m') => cycle_selected_method(state, store)?,

        // Search
        KeyCode::Char('/') => {
            let initial = state.request_filter.clone();
            state.begin_input(InputMode::Search, initial);
        }

        // Request editing
        KeyCode::Char('i') => {
            let initial = state.selected_request().map(|r| r.name.clone()).unwrap_or_default();
            state.begin_input(InputMode::EditRequestName, initial);
        }
        KeyCode::Char('u') => {
            let initial = state.selected_request().map(|r| r.url.clone()).unwrap_or_default();
            state.begin_input(InputMode::EditUrl, initial);
        }
        KeyCode::Char('h') => state.begin_input(InputMode::AddHeader, ""),
        KeyCode::Char('p') => state.begin_input(InputMode::AddQuery, ""),
        KeyCode::Char('b') => {
            let initial = match state.selected_request().map(|r| &r.body) {
                Some(RequestBody::Raw { content, .. }) => content.clone(),
                Some(RequestBody::Json { value })      => value.to_string(),
                _                                       => String::new(),
            };
            state.begin_input(InputMode::EditBody, initial);
        }
        KeyCode::Char('T') => {
            let initial = state.selected_request()
                .and_then(|r| r.timeout_ms)
                .map(|v| v.to_string())
                .unwrap_or_default();
            state.begin_input(InputMode::SetTimeout, initial);
        }

        // Auth
        KeyCode::Char('t') => {
            let initial = match state.selected_request().map(|r| &r.auth) {
                Some(AuthStrategy::Bearer { token }) => token.clone(),
                _ => String::new(),
            };
            state.begin_input(InputMode::SetBearer, initial);
        }
        KeyCode::Char('a') => {
            let initial = match state.selected_request().map(|r| &r.auth) {
                Some(AuthStrategy::Basic { username, password }) => format!("{}:{}", username, password),
                _ => String::new(),
            };
            state.begin_input(InputMode::SetBasicAuth, initial);
        }
        KeyCode::Char('k') => {
            let initial = match state.selected_request().map(|r| &r.auth) {
                Some(AuthStrategy::ApiKey { key, value, in_header }) => {
                    format!("{}:{}:{}", key, value, if *in_header { "h" } else { "q" })
                }
                _ => String::new(),
            };
            state.begin_input(InputMode::SetApiKey, initial);
        }

        // Response tabs
        KeyCode::Char('1') => { state.response_tab = ResponseTab::Body;    state.response_scroll = 0; }
        KeyCode::Char('2') => { state.response_tab = ResponseTab::Headers; state.response_scroll = 0; }
        KeyCode::Char('3') => { state.response_tab = ResponseTab::Logs;    state.response_scroll = 0; }
        KeyCode::Char('4') => { state.response_tab = ResponseTab::History; state.response_scroll = 0; }
        KeyCode::Tab       => { state.response_tab = state.response_tab.next(); state.response_scroll = 0; }

        // Response scroll
        KeyCode::PageDown => { state.response_scroll = state.response_scroll.saturating_add(8); }
        KeyCode::PageUp   => { state.response_scroll = state.response_scroll.saturating_sub(8); }

        // Navigation
        KeyCode::Up    => state.previous_request(),
        KeyCode::Down  => state.next_request(),
        KeyCode::Left  => state.previous_collection(),
        KeyCode::Right => state.next_collection(),

        _ => {}
    }

    Ok(())
}

fn handle_inline_input_key(
    state: &mut AppState,
    store: &FsWorkspaceStore,
    key_code: KeyCode,
) -> YoruResult<()> {
    match key_code {
        KeyCode::Esc => {
            state.end_input();
            state.status_line = "Cancelled".to_string();
        }
        KeyCode::Enter    => commit_inline_input(state, store)?,
        KeyCode::Backspace => {
            state.input_buffer.pop();
            if state.input_mode == InputMode::Search {
                state.request_filter = state.input_buffer.clone();
                align_filtered_selection(state);
            }
        }
        KeyCode::Char(ch) => {
            state.input_buffer.push(ch);
            if state.input_mode == InputMode::Search {
                state.request_filter = state.input_buffer.clone();
                align_filtered_selection(state);
            }
        }
        _ => {}
    }
    Ok(())
}

fn commit_inline_input(state: &mut AppState, store: &FsWorkspaceStore) -> YoruResult<()> {
    let value = state.input_buffer.clone();
    match state.input_mode {
        InputMode::None => {}
        InputMode::Search => {
            state.request_filter = value.trim().to_string();
            align_filtered_selection(state);
            state.status_line = if state.request_filter.is_empty() {
                "Filter cleared".to_string()
            } else {
                format!("Filter: {}", state.request_filter)
            };
        }
        InputMode::EditRequestName    => set_request_name(state, value, store)?,
        InputMode::EditUrl            => set_request_url(state, value, store)?,
        InputMode::AddHeader          => add_request_header(state, value, store)?,
        InputMode::AddQuery           => add_request_query(state, value, store)?,
        InputMode::EditBody           => set_request_raw_body(state, value, store)?,
        InputMode::SetBearer          => set_request_bearer(state, value, store)?,
        InputMode::SetBasicAuth       => set_basic_auth(state, value, store)?,
        InputMode::SetApiKey          => set_api_key(state, value, store)?,
        InputMode::SetTimeout         => set_request_timeout(state, value, store)?,
        InputMode::EditCollectionName => rename_collection(state, value, store)?,
        InputMode::NewCollection      => new_collection(state, value, store)?,
    }
    state.end_input();
    Ok(())
}

fn align_filtered_selection(state: &mut AppState) {
    let filtered = state.filtered_request_indices();
    if filtered.is_empty() { return; }
    if !filtered.contains(&state.selected_request_idx) {
        state.selected_request_idx = filtered[0];
    }
}

fn create_terminal() -> YoruResult<Terminal<CrosstermBackend<Stdout>>> {
    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    Terminal::new(backend)
        .map_err(|err| YoruError::Runtime(format!("failed to initialize terminal: {err}")))
}

fn add_quick_request(state: &mut AppState) {
    if let Some(collection) = state
        .workspace
        .collections
        .get_mut(state.selected_collection_idx)
    {
        let next = collection.requests.len() + 1;
        let req  = RequestTemplate::new(
            format!("New Request {next}"),
            HttpMethod::Get,
            "https://httpbin.org/get",
        );
        collection.requests.push(req);
        state.selected_request_idx = collection.requests.len().saturating_sub(1);
        state.status_line = "Request added".to_string();
    }
}

struct TerminalGuard;

impl TerminalGuard {
    fn enter() -> YoruResult<Self> {
        enable_raw_mode()
            .map_err(|err| YoruError::Runtime(format!("raw mode failed: {err}")))?;
        execute!(io::stdout(), EnterAlternateScreen)
            .map_err(|err| YoruError::Runtime(format!("alternate screen failed: {err}")))?;
        Ok(Self)
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen);
    }
}
