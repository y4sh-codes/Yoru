//! Validation utilities for persistent workspace structures.
//!
//! Doctag:validation

use std::collections::HashSet;

use crate::core::models::Workspace;
use crate::{YoruError, YoruResult};

/// Validates workspace and returns user-actionable errors.
pub fn validate_workspace(workspace: &Workspace) -> YoruResult<()> {
    if workspace.name.trim().is_empty() {
        return Err(YoruError::Validation(
            "workspace name cannot be empty".to_string(),
        ));
    }

    let mut collection_ids = HashSet::new();
    for collection in &workspace.collections {
        if !collection_ids.insert(collection.id) {
            return Err(YoruError::Validation(format!(
                "duplicate collection id found: {}",
                collection.id
            )));
        }

        if collection.name.trim().is_empty() {
            return Err(YoruError::Validation(
                "collection name cannot be empty".to_string(),
            ));
        }

        let mut request_ids = HashSet::new();
        for request in &collection.requests {
            if !request_ids.insert(request.id) {
                return Err(YoruError::Validation(format!(
                    "duplicate request id in collection '{}'",
                    collection.name
                )));
            }

            if request.name.trim().is_empty() {
                return Err(YoruError::Validation(
                    "request name cannot be empty".to_string(),
                ));
            }

            if request.url.trim().is_empty() {
                return Err(YoruError::Validation(format!(
                    "request '{}' must have a URL",
                    request.name
                )));
            }
        }
    }

    if let Some(active_environment_id) = workspace.active_environment_id {
        if !workspace
            .environments
            .iter()
            .any(|env| env.id == active_environment_id)
        {
            return Err(YoruError::Validation(
                "active environment id does not exist".to_string(),
            ));
        }
    }

    Ok(())
}
