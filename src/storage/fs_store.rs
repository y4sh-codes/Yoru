//! Filesystem-backed workspace store.
//!
//! Doctag:storage-fs

use std::fs;
use std::path::{Path, PathBuf};

use directories::ProjectDirs;

use crate::core::models::Workspace;
use crate::core::validation::validate_workspace;
use crate::{YoruError, YoruResult};

/// Contract for workspace persistence backends.
pub trait WorkspaceStore {
    fn root_dir(&self) -> &Path;
    fn workspace_file(&self) -> &Path;
    fn load_workspace(&self) -> YoruResult<Workspace>;
    fn save_workspace(&self, workspace: &Workspace) -> YoruResult<()>;
}

/// Default filesystem implementation.
#[derive(Debug, Clone)]
pub struct FsWorkspaceStore {
    root_dir: PathBuf,
    workspace_file: PathBuf,
}

impl FsWorkspaceStore {
    /// Creates a filesystem store from optional explicit data directory.
    pub fn new(data_dir: Option<PathBuf>) -> YoruResult<Self> {
        let root_dir = if let Some(path) = data_dir {
            path
        } else {
            let project_dirs = ProjectDirs::from("dev", "yoru", "yoru").ok_or_else(|| {
                YoruError::Config("unable to resolve platform data directory".to_string())
            })?;
            project_dirs.data_dir().to_path_buf()
        };

        let workspace_file = root_dir.join(crate::storage::schema::WORKSPACE_FILE_NAME);

        Ok(Self {
            root_dir,
            workspace_file,
        })
    }

    /// Imports a workspace from JSON or YAML.
    pub fn import_workspace(&self, source: &Path) -> YoruResult<Workspace> {
        let payload = fs::read_to_string(source)?;

        let extension = source
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or_default()
            .to_ascii_lowercase();

        let mut workspace = match extension.as_str() {
            "yaml" | "yml" => serde_yaml::from_str::<Workspace>(&payload)?,
            _ => serde_json::from_str::<Workspace>(&payload)?,
        };

        workspace.ensure_seed_data();
        validate_workspace(&workspace)?;

        Ok(workspace)
    }

    /// Exports the current workspace to JSON or YAML.
    pub fn export_workspace(&self, workspace: &Workspace, destination: &Path) -> YoruResult<()> {
        validate_workspace(workspace)?;

        let extension = destination
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or_default()
            .to_ascii_lowercase();

        let serialized = match extension.as_str() {
            "yaml" | "yml" => serde_yaml::to_string(workspace)?,
            _ => serde_json::to_string_pretty(workspace)?,
        };

        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(destination, serialized)?;
        Ok(())
    }

    fn ensure_dirs(&self) -> YoruResult<()> {
        fs::create_dir_all(&self.root_dir)?;
        Ok(())
    }
}

impl WorkspaceStore for FsWorkspaceStore {
    fn root_dir(&self) -> &Path {
        &self.root_dir
    }

    fn workspace_file(&self) -> &Path {
        &self.workspace_file
    }

    fn load_workspace(&self) -> YoruResult<Workspace> {
        self.ensure_dirs()?;

        if !self.workspace_file.exists() {
            let workspace = Workspace::sample();
            self.save_workspace(&workspace)?;
            return Ok(workspace);
        }

        let payload = fs::read_to_string(&self.workspace_file)?;
        let mut workspace = serde_json::from_str::<Workspace>(&payload)?;
        workspace.ensure_seed_data();
        validate_workspace(&workspace)?;
        Ok(workspace)
    }

    fn save_workspace(&self, workspace: &Workspace) -> YoruResult<()> {
        self.ensure_dirs()?;
        validate_workspace(workspace)?;

        let tmp_file = self.workspace_file.with_extension("json.tmp");
        let payload = serde_json::to_string_pretty(workspace)?;

        fs::write(&tmp_file, payload)?;
        fs::rename(tmp_file, &self.workspace_file)?;

        Ok(())
    }
}
