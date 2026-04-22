//! In-memory UI/application state.
//!
//! Doctag:app-state

use crate::core::models::{ExecutedResponse, RequestTemplate, Workspace};
use crate::storage::fs_store::WorkspaceEntry;

// ─── Screens ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Screen {
    /// Workspace picker shown on launch.
    Splash,
    /// Main request editor.
    Main,
}

// ─── Splash-specific input ────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SplashInputMode {
    None,
    NewWorkspace,
    RenameWorkspace,
}

impl SplashInputMode {
    pub fn prompt(self) -> &'static str {
        match self {
            Self::None            => "",
            Self::NewWorkspace    => "New workspace name",
            Self::RenameWorkspace => "Rename workspace",
        }
    }
}

// ─── Main-screen input ────────────────────────────────────────────────────────

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
    SetBasicAuth,
    SetApiKey,
    SetTimeout,
    EditCollectionName,
    NewCollection,
}

impl InputMode {
    pub fn prompt(self) -> &'static str {
        match self {
            Self::None               => "",
            Self::Search             => "Filter requests",
            Self::EditRequestName    => "Edit request name",
            Self::EditUrl            => "Edit URL",
            Self::AddHeader          => "Add header  Key:Value",
            Self::AddQuery           => "Add query param  key=value",
            Self::EditBody           => "Edit body  (Tab to toggle Raw/JSON)",
            Self::SetBearer          => "Set bearer token  (empty clears auth)",
            Self::SetBasicAuth       => "Set basic auth  username:password  (empty clears auth)",
            Self::SetApiKey          => "Set API key  name:value  or  name:value:h/q",
            Self::SetTimeout         => "Set timeout in ms  (empty = default)",
            Self::EditCollectionName => "Rename collection",
            Self::NewCollection      => "New collection name",
        }
    }
}

// ─── Response tabs ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResponseTab {
    Body,
    Headers,
    Logs,
    History,
}

impl ResponseTab {
    pub fn next(self) -> Self {
        match self {
            Self::Body    => Self::Headers,
            Self::Headers => Self::Logs,
            Self::Logs    => Self::History,
            Self::History => Self::Body,
        }
    }
}

// ─── Confirm dialog ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfirmAction {
    DeleteWorkspace(String), // slug
}

// ─── AppState ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct AppState {
    // ── Active workspace ─────────────────────────────────────────────────────
    pub workspace: Workspace,
    pub active_slug: String,

    // ── Screen ───────────────────────────────────────────────────────────────
    pub screen: Screen,

    // ── Splash state ─────────────────────────────────────────────────────────
    pub available_workspaces: Vec<WorkspaceEntry>,
    pub splash_selected_idx: usize,
    pub splash_input_mode: SplashInputMode,
    pub splash_input_buffer: String,
    pub splash_confirm: Option<ConfirmAction>,

    // ── Main UI state ─────────────────────────────────────────────────────────
    pub selected_collection_idx: usize,
    pub selected_request_idx: usize,
    pub status_line: String,
    pub last_error: Option<String>,
    pub last_response: Option<ExecutedResponse>,
    pub input_mode: InputMode,
    pub input_buffer: String,
    pub input_cursor: usize,   // byte-index cursor for left/right navigation
    pub body_is_json: bool,    // toggle: true = Json body, false = Raw body
    pub request_filter: String,
    pub response_tab: ResponseTab,
    pub response_scroll: u16,
    pub show_help: bool,
    pub should_quit: bool,
}

impl AppState {
    /// Creates the initial state with a workspace list and active workspace.
    pub fn new(
        workspace: Workspace,
        slug: String,
        available_workspaces: Vec<WorkspaceEntry>,
    ) -> Self {
        // Find the index in the list that matches our active slug
        let splash_selected_idx = available_workspaces
            .iter()
            .position(|e| e.slug == slug)
            .unwrap_or(0);

        Self {
            workspace,
            active_slug: slug,
            screen: Screen::Splash,
            available_workspaces,
            splash_selected_idx,
            splash_input_mode: SplashInputMode::None,
            splash_input_buffer: String::new(),
            splash_confirm: None,
            selected_collection_idx: 0,
            selected_request_idx: 0,
            status_line: "Ready".to_string(),
            last_error: None,
            last_response: None,
            input_mode: InputMode::None,
            input_buffer: String::new(),
            input_cursor: 0,
            body_is_json: true,
            request_filter: String::new(),
            response_tab: ResponseTab::Body,
            response_scroll: 0,
            show_help: false,
            should_quit: false,
        }
    }

    // ── Splash helpers ────────────────────────────────────────────────────────

    pub fn splash_selected_entry(&self) -> Option<&WorkspaceEntry> {
        self.available_workspaces.get(self.splash_selected_idx)
    }

    pub fn splash_next(&mut self) {
        if !self.available_workspaces.is_empty() {
            self.splash_selected_idx =
                (self.splash_selected_idx + 1) % self.available_workspaces.len();
        }
    }

    pub fn splash_prev(&mut self) {
        if !self.available_workspaces.is_empty() {
            self.splash_selected_idx = if self.splash_selected_idx == 0 {
                self.available_workspaces.len() - 1
            } else {
                self.splash_selected_idx - 1
            };
        }
    }

    pub fn begin_splash_input(&mut self, mode: SplashInputMode, initial: impl Into<String>) {
        self.splash_input_mode = mode;
        self.splash_input_buffer = initial.into();
    }

    pub fn end_splash_input(&mut self) {
        self.splash_input_mode = SplashInputMode::None;
        self.splash_input_buffer.clear();
    }

    // ── Main screen helpers ───────────────────────────────────────────────────

    pub fn selected_collection(&self) -> Option<&crate::core::models::Collection> {
        self.workspace.collections.get(self.selected_collection_idx)
    }

    pub fn selected_request(&self) -> Option<&RequestTemplate> {
        self.workspace
            .request_at(self.selected_collection_idx, self.selected_request_idx)
    }

    pub fn selected_request_mut(&mut self) -> Option<&mut RequestTemplate> {
        self.workspace
            .collections
            .get_mut(self.selected_collection_idx)
            .and_then(|c| c.requests.get_mut(self.selected_request_idx))
    }

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
            .filter_map(|(idx, req)| {
                let haystack =
                    format!("{} {} {}", req.name, req.url, req.method).to_ascii_lowercase();
                if haystack.contains(&needle) { Some(idx) } else { None }
            })
            .collect()
    }

    pub fn normalize_selection(&mut self) {
        if let Some(col) = self.selected_collection() {
            if col.requests.is_empty() {
                self.selected_request_idx = 0;
            } else if self.selected_request_idx >= col.requests.len() {
                self.selected_request_idx = col.requests.len() - 1;
            }
        } else {
            self.selected_collection_idx = 0;
            self.selected_request_idx = 0;
        }
    }

    pub fn next_request(&mut self) {
        let filtered = self.filtered_request_indices();
        if filtered.is_empty() { return; }
        let pos = filtered.iter().position(|i| *i == self.selected_request_idx).unwrap_or(0);
        self.selected_request_idx = filtered[(pos + 1) % filtered.len()];
    }

    pub fn previous_request(&mut self) {
        let filtered = self.filtered_request_indices();
        if filtered.is_empty() { return; }
        let pos = filtered.iter().position(|i| *i == self.selected_request_idx).unwrap_or(0);
        let prev = if pos == 0 { filtered.len() - 1 } else { pos - 1 };
        self.selected_request_idx = filtered[prev];
    }

    pub fn next_collection(&mut self) {
        if !self.workspace.collections.is_empty() {
            self.selected_collection_idx =
                (self.selected_collection_idx + 1) % self.workspace.collections.len();
            self.selected_request_idx = 0;
            self.normalize_selection();
        }
    }

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

    pub fn begin_input(&mut self, mode: InputMode, initial: impl Into<String>) {
        self.input_mode = mode;
        self.input_buffer = initial.into();
        self.input_cursor = self.input_buffer.len();
        self.status_line = format!("{}: Enter to confirm, Esc to cancel", mode.prompt());
    }

    pub fn end_input(&mut self) {
        self.input_mode = InputMode::None;
        self.input_buffer.clear();
        self.input_cursor = 0;
    }

    /// Insert a char at the cursor position.
    pub fn input_insert(&mut self, ch: char) {
        self.input_buffer.insert(self.input_cursor, ch);
        self.input_cursor += ch.len_utf8();
        if self.input_cursor > self.input_buffer.len() {
            self.input_cursor = self.input_buffer.len();
        }
    }

    /// Delete char before cursor (backspace).
    pub fn input_backspace(&mut self) {
        if self.input_cursor == 0 { return; }
        // Find the previous character boundary
        let mut prev = self.input_cursor - 1;
        while prev > 0 && !self.input_buffer.is_char_boundary(prev) {
            prev -= 1;
        }
        self.input_buffer.drain(prev..self.input_cursor);
        self.input_cursor = prev;
    }

    /// Move cursor left by one char.
    pub fn input_move_left(&mut self) {
        if self.input_cursor == 0 { return; }
        let mut pos = self.input_cursor - 1;
        while pos > 0 && !self.input_buffer.is_char_boundary(pos) {
            pos -= 1;
        }
        self.input_cursor = pos;
    }

    /// Move cursor right by one char.
    pub fn input_move_right(&mut self) {
        if self.input_cursor >= self.input_buffer.len() { return; }
        let mut pos = self.input_cursor + 1;
        while pos < self.input_buffer.len() && !self.input_buffer.is_char_boundary(pos) {
            pos += 1;
        }
        self.input_cursor = pos;
    }
}
