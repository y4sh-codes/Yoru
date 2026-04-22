//! Filesystem-backed workspace store.
//!
//! Layout:
//!   <data_dir>/
//!     workspaces/
//!       default.json          ← one file per workspace
//!       my-project.json
//!     active_workspace        ← plain-text slug of last-opened workspace
//!
//! Migration: if the legacy `workspace.json` exists in <data_dir> and the
//! `workspaces/` subdirectory does not, the file is automatically moved to
//! `workspaces/default.json` on first access.
//!
//! Doctag:storage-fs

use std::fs;
use std::path::{Path, PathBuf};

use directories::ProjectDirs;

use crate::core::models::Workspace;
use crate::core::validation::validate_workspace;
use crate::storage::schema::{ACTIVE_SLUG_FILE, WORKSPACES_DIR_NAME, WORKSPACE_FILE_NAME};
use crate::{YoruError, YoruResult};

// ─── Contracts ───────────────────────────────────────────────────────────────

/// Minimal contract for single-workspace persistence (used by TUI actions).
pub trait WorkspaceStore {
    fn root_dir(&self) -> &Path;
    fn workspace_file(&self) -> &Path;
    fn load_workspace(&self) -> YoruResult<Workspace>;
    fn save_workspace(&self, workspace: &Workspace) -> YoruResult<()>;
}

/// Lightweight summary shown on the splash workspace picker.
#[derive(Debug, Clone)]
pub struct WorkspaceEntry {
    pub display_name: String,
    pub slug: String,
    pub collections: usize,
    pub requests: usize,
    pub environments: usize,
}

/// Multi-workspace registry operations, implemented by `FsWorkspaceStore`.
pub trait WorkspaceRegistry {
    /// Returns all known workspaces sorted alphabetically by display name.
    fn list_workspaces(&self) -> YoruResult<Vec<WorkspaceEntry>>;

    /// Loads a workspace by its slug (filename stem).
    fn load_workspace_by_slug(&self, slug: &str) -> YoruResult<Workspace>;

    /// Saves a workspace to its slug file (atomic write).
    fn save_workspace_with_slug(&self, workspace: &Workspace, slug: &str) -> YoruResult<()>;

    /// Creates a new workspace with the given display name.
    /// Returns (workspace, slug).
    fn create_workspace(&self, name: &str) -> YoruResult<(Workspace, String)>;

    /// Deletes the workspace file for the given slug.
    fn delete_workspace_by_slug(&self, slug: &str) -> YoruResult<()>;

    /// Renames the workspace file (changes slug + in-file name).
    fn rename_workspace(&self, slug: &str, new_name: &str) -> YoruResult<String>;

    /// Returns the slug of the last-opened workspace, if any.
    fn get_active_slug(&self) -> YoruResult<Option<String>>;

    /// Persists the active workspace slug.
    fn set_active_slug(&self, slug: &str) -> YoruResult<()>;

    /// Returns the `workspaces/` subdirectory path.
    fn workspaces_dir(&self) -> PathBuf;
}

// ─── Implementation ───────────────────────────────────────────────────────────

/// Default filesystem implementation.
#[derive(Debug, Clone)]
pub struct FsWorkspaceStore {
    root_dir: PathBuf,
    /// Active slug – determined at startup and updated when the user switches.
    active_slug: std::sync::Arc<std::sync::Mutex<String>>,
    /// Cached path for the workspace_file() trait method.
    workspace_file_cache: std::sync::Arc<std::sync::Mutex<PathBuf>>,
}

impl FsWorkspaceStore {
    /// Creates a filesystem store from an optional explicit data directory.
    pub fn new(data_dir: Option<PathBuf>) -> YoruResult<Self> {
        let root_dir = if let Some(path) = data_dir {
            path
        } else {
            let project_dirs = ProjectDirs::from("dev", "yoru", "yoru").ok_or_else(|| {
                YoruError::Config("unable to resolve platform data directory".to_string())
            })?;
            project_dirs.data_dir().to_path_buf()
        };

        fs::create_dir_all(&root_dir)?;
        fs::create_dir_all(root_dir.join(WORKSPACES_DIR_NAME))?;

        let workspaces_root = root_dir.join(WORKSPACES_DIR_NAME);
        let initial_cache = workspaces_root.join("default.json");
        let store = Self {
            root_dir,
            active_slug: std::sync::Arc::new(std::sync::Mutex::new(String::new())),
            workspace_file_cache: std::sync::Arc::new(std::sync::Mutex::new(initial_cache)),
        };

        // One-time migration from legacy workspace.json → workspaces/default.json
        store.migrate_legacy()?;

        // Determine starting active slug
        let slug = store
            .get_active_slug()?
            .unwrap_or_else(|| "default".to_string());
        *store.active_slug.lock().unwrap() = slug;

        Ok(store)
    }

    // ── Private helpers ───────────────────────────────────────────────────────

    fn workspaces_dir_path(&self) -> PathBuf {
        self.root_dir.join(WORKSPACES_DIR_NAME)
    }

    fn slug_path(&self, slug: &str) -> PathBuf {
        self.workspaces_dir_path().join(format!("{}.json", slug))
    }

    fn active_slug_file(&self) -> PathBuf {
        self.root_dir.join(ACTIVE_SLUG_FILE)
    }

    fn current_slug(&self) -> String {
        self.active_slug.lock().unwrap().clone()
    }

    fn update_cache(&self, slug: &str) {
        let path = self.workspaces_dir_path().join(format!("{}.json", slug));
        *self.workspace_file_cache.lock().unwrap() = path;
    }

    /// Migrate `workspace.json` → `workspaces/default.json` if needed.
    fn migrate_legacy(&self) -> YoruResult<()> {
        let legacy = self.root_dir.join(WORKSPACE_FILE_NAME);
        let target = self.slug_path("default");

        if legacy.exists() && !target.exists() {
            fs::copy(&legacy, &target)?;
            // Keep the old file; remove it only after successful copy.
            let _ = fs::remove_file(&legacy);
        }

        Ok(())
    }

    /// Converts a display name to a unique slug (lowercase, spaces → dashes).
    fn name_to_unique_slug(&self, name: &str) -> YoruResult<String> {
        let base: String = name
            .to_ascii_lowercase()
            .chars()
            .map(|c| if c.is_alphanumeric() { c } else { '-' })
            .collect::<String>()
            .split('-')
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join("-");

        let base = if base.is_empty() { "workspace".to_string() } else { base };

        // Ensure uniqueness
        if !self.slug_path(&base).exists() {
            return Ok(base);
        }

        for n in 2u32..=999 {
            let candidate = format!("{}-{}", base, n);
            if !self.slug_path(&candidate).exists() {
                return Ok(candidate);
            }
        }

        Err(YoruError::Runtime(
            "could not generate a unique workspace slug".to_string(),
        ))
    }

    // ── Public helper: import/export ─────────────────────────────────────────

    /// Imports a workspace from JSON or YAML.
    pub fn import_workspace(&self, source: &Path) -> YoruResult<Workspace> {
        let payload = fs::read_to_string(source)?;
        let ext = source
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or_default()
            .to_ascii_lowercase();

        let mut workspace = match ext.as_str() {
            "yaml" | "yml" => serde_yaml::from_str::<Workspace>(&payload)?,
            _ => serde_json::from_str::<Workspace>(&payload)?,
        };

        workspace.ensure_seed_data();
        validate_workspace(&workspace)?;
        Ok(workspace)
    }

    /// Exports a workspace to JSON or YAML.
    pub fn export_workspace(&self, workspace: &Workspace, destination: &Path) -> YoruResult<()> {
        validate_workspace(workspace)?;
        let ext = destination
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or_default()
            .to_ascii_lowercase();

        let serialized = match ext.as_str() {
            "yaml" | "yml" => serde_yaml::to_string(workspace)?,
            _ => serde_json::to_string_pretty(workspace)?,
        };

        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(destination, serialized)?;
        Ok(())
    }
}

// ─── WorkspaceStore impl (single-workspace interface used by TUI actions) ────

impl WorkspaceStore for FsWorkspaceStore {
    fn root_dir(&self) -> &Path {
        &self.root_dir
    }

    fn workspace_file(&self) -> &Path {
        // SAFETY: We return a reference to the cached PathBuf inside the Arc<Mutex>.
        // The cache is updated every time active_slug changes so it stays consistent.
        // The PathBuf lives as long as the store, so the lifetime is valid.
        let guard = self.workspace_file_cache.lock().unwrap();
        // SAFETY: The PathBuf is heap-allocated inside the Arc and outlives self.
        unsafe { &*(guard.as_path() as *const Path) }
    }

    fn load_workspace(&self) -> YoruResult<Workspace> {
        let slug = self.current_slug();
        let path = self.slug_path(&slug);

        if !path.exists() {
            // First-ever run: create the default workspace
            let workspace = Workspace::sample();
            self.save_workspace(&workspace)?;
            return Ok(workspace);
        }

        let payload = fs::read_to_string(&path)?;
        let mut workspace = serde_json::from_str::<Workspace>(&payload)?;
        workspace.ensure_seed_data();
        validate_workspace(&workspace)?;
        Ok(workspace)
    }

    fn save_workspace(&self, workspace: &Workspace) -> YoruResult<()> {
        let slug = self.current_slug();
        self.save_workspace_with_slug(workspace, &slug)
    }
}

// ─── WorkspaceRegistry impl ───────────────────────────────────────────────────

impl WorkspaceRegistry for FsWorkspaceStore {
    fn workspaces_dir(&self) -> PathBuf {
        self.workspaces_dir_path()
    }

    fn list_workspaces(&self) -> YoruResult<Vec<WorkspaceEntry>> {
        let dir = self.workspaces_dir_path();
        fs::create_dir_all(&dir)?;

        let mut entries = Vec::new();

        for entry in fs::read_dir(&dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|e| e.to_str()) != Some("json") {
                continue;
            }

            let slug = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown")
                .to_string();

            // Read only enough to extract the summary
            let summary = match fs::read_to_string(&path)
                .ok()
                .and_then(|s| serde_json::from_str::<Workspace>(&s).ok())
            {
                Some(ws) => {
                    let requests: usize = ws.collections.iter().map(|c| c.requests.len()).sum();
                    WorkspaceEntry {
                        display_name: ws.name,
                        slug: slug.clone(),
                        collections: ws.collections.len(),
                        requests,
                        environments: ws.environments.len(),
                    }
                }
                None => WorkspaceEntry {
                    display_name: slug.clone(),
                    slug: slug.clone(),
                    collections: 0,
                    requests: 0,
                    environments: 0,
                },
            };

            entries.push(summary);
        }

        // Ensure at least a "default" entry always exists
        if entries.is_empty() {
            let ws = Workspace::sample();
            self.save_workspace_with_slug(&ws, "default")?;
            entries.push(WorkspaceEntry {
                display_name: ws.name,
                slug: "default".to_string(),
                collections: ws.collections.len(),
                requests: ws.collections.iter().map(|c| c.requests.len()).sum(),
                environments: ws.environments.len(),
            });
        }

        entries.sort_by(|a, b| a.display_name.to_ascii_lowercase().cmp(&b.display_name.to_ascii_lowercase()));
        Ok(entries)
    }

    fn load_workspace_by_slug(&self, slug: &str) -> YoruResult<Workspace> {
        // Switch active slug then delegate to single-workspace interface
        *self.active_slug.lock().unwrap() = slug.to_string();
        self.update_cache(slug);
        self.set_active_slug(slug)?;
        self.load_workspace()
    }

    fn save_workspace_with_slug(&self, workspace: &Workspace, slug: &str) -> YoruResult<()> {
        let dir = self.workspaces_dir_path();
        fs::create_dir_all(&dir)?;
        validate_workspace(workspace)?;

        let path = self.slug_path(slug);
        let tmp = path.with_extension("json.tmp");
        let payload = serde_json::to_string_pretty(workspace)?;

        fs::write(&tmp, payload)?;
        fs::rename(tmp, &path)?;
        Ok(())
    }

    fn create_workspace(&self, name: &str) -> YoruResult<(Workspace, String)> {
        let name = name.trim();
        if name.is_empty() {
            return Err(YoruError::Validation("workspace name cannot be empty".to_string()));
        }

        let slug = self.name_to_unique_slug(name)?;
        let mut workspace = Workspace::sample();
        workspace.name = name.to_string();
        workspace.id = uuid::Uuid::new_v4();

        self.save_workspace_with_slug(&workspace, &slug)?;
        *self.active_slug.lock().unwrap() = slug.clone();
        self.set_active_slug(&slug)?;

        Ok((workspace, slug))
    }

    fn delete_workspace_by_slug(&self, slug: &str) -> YoruResult<()> {
        let path = self.slug_path(slug);
        if !path.exists() {
            return Err(YoruError::Runtime(format!(
                "workspace '{}' not found",
                slug
            )));
        }
        fs::remove_file(&path)?;
        Ok(())
    }

    fn rename_workspace(&self, slug: &str, new_name: &str) -> YoruResult<String> {
        let new_name = new_name.trim();
        if new_name.is_empty() {
            return Err(YoruError::Validation("name cannot be empty".to_string()));
        }

        // Load, update name, compute new slug
        let mut workspace = self.load_workspace_by_slug(slug)?;
        workspace.name = new_name.to_string();

        let new_slug = self.name_to_unique_slug(new_name)?;
        self.save_workspace_with_slug(&workspace, &new_slug)?;

        // Remove old file if slug changed
        if new_slug != slug {
            let old_path = self.slug_path(slug);
            if old_path.exists() {
                fs::remove_file(old_path)?;
            }
        }

        // Update active slug if we just renamed the current workspace
        if self.current_slug() == slug {
            *self.active_slug.lock().unwrap() = new_slug.clone();
            self.set_active_slug(&new_slug)?;
        }

        Ok(new_slug)
    }

    fn get_active_slug(&self) -> YoruResult<Option<String>> {
        let path = self.active_slug_file();
        if path.exists() {
            let content = fs::read_to_string(&path)?;
            let slug = content.trim().to_string();
            if !slug.is_empty() {
                return Ok(Some(slug));
            }
        }
        Ok(None)
    }

    fn set_active_slug(&self, slug: &str) -> YoruResult<()> {
        fs::write(self.active_slug_file(), slug)?;
        Ok(())
    }
}
