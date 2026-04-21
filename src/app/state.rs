//! In-memory UI/application state.
//!
//! Doctag:app-state

use crate::core::models::{ExecutedResponse, RequestTemplate, Workspace};

/// Input mode used by inline TUI editors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    None,
    Search,
    EditRequestName,
    EditUrl,
    AddHeader,
    AddQuery,
    EditBody,
    SetBearer,
}

impl InputMode {
    /// Human-readable prompt for each mode.
    pub fn prompt(self) -> &'static str {
        match self {
            Self::None => "",
            Self::Search => "Filter requests",
            Self::EditRequestName => "Edit request name",
            Self::EditUrl => "Edit URL",
            Self::AddHeader => "Add header (Key:Value)",
            Self::AddQuery => "Add query (key=value)",
            Self::EditBody => "Edit raw body",
            Self::SetBearer => "Set bearer token (empty clears auth)",
        }
    }
}

/// Tabs used in the response panel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResponseTab {
    Body,
    Headers,
    Logs,
    History,
}

impl ResponseTab {
    /// Cycles tab in order.
    pub fn next(self) -> Self {
        match self {
            Self::Body => Self::Headers,
            Self::Headers => Self::Logs,
            Self::Logs => Self::History,
            Self::History => Self::Body,
        }
    }
}

/// Top-level state shared across UI render and actions.
#[derive(Debug, Clone)]
pub struct AppState {
    pub workspace: Workspace,
    pub selected_collection_idx: usize,
    pub selected_request_idx: usize,
    pub status_line: String,
    pub last_error: Option<String>,
    pub last_response: Option<ExecutedResponse>,
    pub input_mode: InputMode,
    pub input_buffer: String,
    pub request_filter: String,
    pub response_tab: ResponseTab,
    pub show_help: bool,
    pub should_quit: bool,
}

impl AppState {
    /// Builds initial state from workspace.
    pub fn new(mut workspace: Workspace) -> Self {
        workspace.ensure_seed_data();

        Self {
            workspace,
            selected_collection_idx: 0,
            selected_request_idx: 0,
            status_line: "Ready".to_string(),
            last_error: None,
            last_response: None,
            input_mode: InputMode::None,
            input_buffer: String::new(),
            request_filter: String::new(),
            response_tab: ResponseTab::Body,
            show_help: false,
            should_quit: false,
        }
    }

    /// Returns currently selected collection if present.
    pub fn selected_collection(&self) -> Option<&crate::core::models::Collection> {
        self.workspace.collections.get(self.selected_collection_idx)
    }

    /// Returns currently selected request if present.
    pub fn selected_request(&self) -> Option<&RequestTemplate> {
        self.workspace
            .request_at(self.selected_collection_idx, self.selected_request_idx)
    }

    /// Returns currently selected request mutably.
    pub fn selected_request_mut(&mut self) -> Option<&mut RequestTemplate> {
        self.workspace
            .collections
            .get_mut(self.selected_collection_idx)
            .and_then(|collection| collection.requests.get_mut(self.selected_request_idx))
    }

    /// Returns request indices filtered by current request filter.
    pub fn filtered_request_indices(&self) -> Vec<usize> {
        let Some(collection) = self.selected_collection() else {
            return Vec::new();
        };

        if self.request_filter.trim().is_empty() {
            return (0..collection.requests.len()).collect();
        }

        let needle = self.request_filter.to_ascii_lowercase();
        collection
            .requests
            .iter()
            .enumerate()
            .filter_map(|(idx, request)| {
                let haystack = format!(
                    "{} {} {}",
                    request.name,
                    request.url,
                    request.method
                )
                .to_ascii_lowercase();

                if haystack.contains(&needle) {
                    Some(idx)
                } else {
                    None
                }
            })
            .collect()
    }

    /// Ensures selected request index is valid for current collection.
    pub fn normalize_selection(&mut self) {
        if let Some(collection) = self.selected_collection() {
            if collection.requests.is_empty() {
                self.selected_request_idx = 0;
            } else if self.selected_request_idx >= collection.requests.len() {
                self.selected_request_idx = collection.requests.len() - 1;
            }
        } else {
            self.selected_collection_idx = 0;
            self.selected_request_idx = 0;
        }
    }

    /// Selects the next request in current collection.
    pub fn next_request(&mut self) {
        let filtered = self.filtered_request_indices();
        if filtered.is_empty() {
            return;
        }

        let current_pos = filtered
            .iter()
            .position(|idx| *idx == self.selected_request_idx)
            .unwrap_or(0);
        let next_pos = (current_pos + 1) % filtered.len();
        self.selected_request_idx = filtered[next_pos];
    }

    /// Selects the previous request in current collection.
    pub fn previous_request(&mut self) {
        let filtered = self.filtered_request_indices();
        if filtered.is_empty() {
            return;
        }

        let current_pos = filtered
            .iter()
            .position(|idx| *idx == self.selected_request_idx)
            .unwrap_or(0);
        let previous_pos = if current_pos == 0 {
            filtered.len() - 1
        } else {
            current_pos - 1
        };
        self.selected_request_idx = filtered[previous_pos];
    }

    /// Moves to next collection and resets request selection.
    pub fn next_collection(&mut self) {
        if !self.workspace.collections.is_empty() {
            self.selected_collection_idx =
                (self.selected_collection_idx + 1) % self.workspace.collections.len();
            self.selected_request_idx = 0;
            self.normalize_selection();
        }
    }

    /// Moves to previous collection and resets request selection.
    pub fn previous_collection(&mut self) {
        if !self.workspace.collections.is_empty() {
            self.selected_collection_idx = if self.selected_collection_idx == 0 {
                self.workspace.collections.len() - 1
            } else {
                self.selected_collection_idx - 1
            };
            self.selected_request_idx = 0;
            self.normalize_selection();
        }
    }

    /// Starts an inline input session.
    pub fn begin_input(&mut self, mode: InputMode, initial: impl Into<String>) {
        self.input_mode = mode;
        self.input_buffer = initial.into();
        self.status_line = format!("{}: enter to confirm, esc to cancel", mode.prompt());
    }

    /// Ends inline input session.
    pub fn end_input(&mut self) {
        self.input_mode = InputMode::None;
        self.input_buffer.clear();
    }
}
