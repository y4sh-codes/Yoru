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
    delete_selected_request, duplicate_selected_request, run_selected_request, set_request_bearer,
    set_request_name, set_request_raw_body, set_request_url,
};
use crate::app::state::{AppState, InputMode, ResponseTab};
use crate::core::models::{AuthStrategy, HttpMethod, RequestBody, RequestTemplate};
use crate::http::executor::HttpExecutor;
use crate::storage::fs_store::WorkspaceStore;
use crate::tui::events::{AppEvent, EventHandler};
use crate::tui::theme::Theme;
use crate::tui::ui::draw;
use crate::{YoruError, YoruResult};

pub mod events;
pub mod theme;
pub mod ui;

/// Runs the interactive terminal application.
pub async fn run_tui<S: WorkspaceStore>(
    mut state: AppState,
    executor: HttpExecutor,
    store: &S,
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

async fn handle_key_event<S: WorkspaceStore>(
    state: &mut AppState,
    executor: &HttpExecutor,
    store: &S,
    key_code: KeyCode,
) -> YoruResult<()> {
    if state.input_mode != InputMode::None {
        return handle_inline_input_key(state, store, key_code);
    }

    match key_code {
        KeyCode::Char('q') => state.should_quit = true,
        KeyCode::Char('?') => state.show_help = !state.show_help,
        KeyCode::Char('c') => state.last_error = None,
        KeyCode::Char('r') | KeyCode::Enter => {
            run_selected_request(state, executor, store).await?;
            state.response_tab = ResponseTab::Body;
        }
        KeyCode::Char('e') => {
            cycle_environment(state, store)?;
        }
        KeyCode::Char('n') => {
            add_quick_request(state);
            store.save_workspace(&state.workspace)?;
        }
        KeyCode::Char('d') => {
            duplicate_selected_request(state, store)?;
        }
        KeyCode::Char('x') => {
            delete_selected_request(state, store)?;
        }
        KeyCode::Char('m') => {
            cycle_selected_method(state, store)?;
        }
        KeyCode::Char('/') => {
            let initial = state.request_filter.clone();
            state.begin_input(InputMode::Search, initial);
        }
        KeyCode::Char('i') => {
            let initial = state
                .selected_request()
                .map(|request| request.name.clone())
                .unwrap_or_default();
            state.begin_input(InputMode::EditRequestName, initial);
        }
        KeyCode::Char('u') => {
            let initial = state
                .selected_request()
                .map(|request| request.url.clone())
                .unwrap_or_default();
            state.begin_input(InputMode::EditUrl, initial);
        }
        KeyCode::Char('h') => state.begin_input(InputMode::AddHeader, ""),
        KeyCode::Char('p') => state.begin_input(InputMode::AddQuery, ""),
        KeyCode::Char('b') => {
            let initial = match state.selected_request().map(|request| &request.body) {
                Some(RequestBody::Raw { content, .. }) => content.clone(),
                Some(RequestBody::Json { value }) => value.to_string(),
                _ => String::new(),
            };
            state.begin_input(InputMode::EditBody, initial);
        }
        KeyCode::Char('t') => {
            let initial = match state.selected_request().map(|request| &request.auth) {
                Some(AuthStrategy::Bearer { token }) => token.clone(),
                _ => String::new(),
            };
            state.begin_input(InputMode::SetBearer, initial);
        }
        KeyCode::Char('1') => state.response_tab = ResponseTab::Body,
        KeyCode::Char('2') => state.response_tab = ResponseTab::Headers,
        KeyCode::Char('3') => state.response_tab = ResponseTab::Logs,
        KeyCode::Char('4') => state.response_tab = ResponseTab::History,
        KeyCode::Tab => state.response_tab = state.response_tab.next(),
        KeyCode::Up => state.previous_request(),
        KeyCode::Down => state.next_request(),
        KeyCode::Left => state.previous_collection(),
        KeyCode::Right => state.next_collection(),
        _ => {}
    }

    Ok(())
}

fn handle_inline_input_key<S: WorkspaceStore>(
    state: &mut AppState,
    store: &S,
    key_code: KeyCode,
) -> YoruResult<()> {
    match key_code {
        KeyCode::Esc => {
            state.end_input();
            state.status_line = "Input cancelled".to_string();
        }
        KeyCode::Enter => {
            commit_inline_input(state, store)?;
        }
        KeyCode::Backspace => {
            state.input_buffer.pop();
            if state.input_mode == InputMode::Search {
                state.request_filter = state.input_buffer.clone();
                align_filtered_selection(state);
            }
        }
        KeyCode::Char(character) => {
            state.input_buffer.push(character);
            if state.input_mode == InputMode::Search {
                state.request_filter = state.input_buffer.clone();
                align_filtered_selection(state);
            }
        }
        _ => {}
    }

    Ok(())
}

fn commit_inline_input<S: WorkspaceStore>(state: &mut AppState, store: &S) -> YoruResult<()> {
    let value = state.input_buffer.clone();

    match state.input_mode {
        InputMode::None => {}
        InputMode::Search => {
            state.request_filter = value.trim().to_string();
            align_filtered_selection(state);
            state.status_line = if state.request_filter.is_empty() {
                "Filter cleared".to_string()
            } else {
                format!("Filter applied: {}", state.request_filter)
            };
        }
        InputMode::EditRequestName => {
            set_request_name(state, value, store)?;
        }
        InputMode::EditUrl => {
            set_request_url(state, value, store)?;
        }
        InputMode::AddHeader => {
            add_request_header(state, value, store)?;
        }
        InputMode::AddQuery => {
            add_request_query(state, value, store)?;
        }
        InputMode::EditBody => {
            set_request_raw_body(state, value, store)?;
        }
        InputMode::SetBearer => {
            set_request_bearer(state, value, store)?;
        }
    }

    state.end_input();
    Ok(())
}

fn align_filtered_selection(state: &mut AppState) {
    let filtered = state.filtered_request_indices();
    if filtered.is_empty() {
        return;
    }

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
        let next_index = collection.requests.len() + 1;
        let request = RequestTemplate::new(
            format!("New Request {next_index}"),
            HttpMethod::Get,
            "https://httpbin.org/get",
        );

        collection.requests.push(request);
        state.selected_request_idx = collection.requests.len().saturating_sub(1);
        state.status_line = "Added quick request".to_string();
    }
}

struct TerminalGuard;

impl TerminalGuard {
    fn enter() -> YoruResult<Self> {
        enable_raw_mode().map_err(|err| YoruError::Runtime(format!("raw mode failed: {err}")))?;

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
