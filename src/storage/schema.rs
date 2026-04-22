//! Storage schema constants and helpers.

/// Current on-disk workspace schema version.
pub const SCHEMA_VERSION: &str = "1.0.0";

/// Legacy single-workspace filename (kept for migration).
pub const WORKSPACE_FILE_NAME: &str = "workspace.json";

/// Directory that holds all per-workspace JSON files.
pub const WORKSPACES_DIR_NAME: &str = "workspaces";

/// File that remembers which workspace was last opened.
pub const ACTIVE_SLUG_FILE: &str = "active_workspace";
