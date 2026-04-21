//! Workspace manipulation utilities.
//!
//! Doctag:workspace-service

use chrono::Utc;
use uuid::Uuid;

use crate::core::models::{
    Collection, Environment, ExecutedResponse, HistoryEntry, HttpMethod, RequestTemplate, Workspace,
};
use crate::{YoruError, YoruResult};

impl Workspace {
    /// Creates a sample workspace with one request and one environment.
    pub fn sample() -> Self {
        let mut collection = Collection::new("Quickstart");
        collection.description = Some("Starter requests to verify the setup.".to_string());

        let mut request = RequestTemplate::new(
            "Health Check",
            HttpMethod::Get,
            "https://httpbin.org/get?service={{service_name}}",
        );
        request.tags = vec!["health".to_string(), "starter".to_string()];
        collection.requests.push(request);

        let mut environment = Environment::new("local");
        environment.variables.push(crate::core::models::EnvVar {
            key: "service_name".to_string(),
            value: "yoru".to_string(),
            secret: false,
        });

        Self {
            id: Uuid::new_v4(),
            name: "Default Workspace".to_string(),
            schema_version: crate::storage::schema::SCHEMA_VERSION.to_string(),
            collections: vec![collection],
            active_environment_id: Some(environment.id),
            environments: vec![environment],
            history: Vec::new(),
            updated_at: Utc::now(),
        }
    }

    /// Returns the currently active environment.
    pub fn active_environment(&self) -> Option<&Environment> {
        self.active_environment_id
            .and_then(|id| self.environments.iter().find(|env| env.id == id))
    }

    /// Rotates active environment to the next available one.
    pub fn cycle_environment(&mut self) {
        if self.environments.is_empty() {
            self.active_environment_id = None;
            return;
        }

        match self.active_environment_id {
            None => self.active_environment_id = Some(self.environments[0].id),
            Some(active) => {
                let idx = self
                    .environments
                    .iter()
                    .position(|env| env.id == active)
                    .unwrap_or(0);
                let next = (idx + 1) % self.environments.len();
                self.active_environment_id = Some(self.environments[next].id);
            }
        }
    }

    /// Sets the active environment by id.
    pub fn set_active_environment(&mut self, id: Uuid) -> YoruResult<()> {
        if self.environments.iter().any(|env| env.id == id) {
            self.active_environment_id = Some(id);
            self.updated_at = Utc::now();
            Ok(())
        } else {
            Err(YoruError::Validation(format!(
                "environment {id} does not exist"
            )))
        }
    }

    /// Fetches a request by collection and request index.
    pub fn request_at(
        &self,
        collection_idx: usize,
        request_idx: usize,
    ) -> Option<&RequestTemplate> {
        self.collections
            .get(collection_idx)
            .and_then(|collection| collection.requests.get(request_idx))
    }

    /// Appends a history entry and keeps only the newest records.
    pub fn append_history(
        &mut self,
        request: &RequestTemplate,
        response: &ExecutedResponse,
        environment_name: Option<String>,
    ) {
        self.history.push(HistoryEntry {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            request_name: request.name.clone(),
            method: request.method,
            url: request.url.clone(),
            status: response.status,
            latency_ms: response.duration_ms,
            response_size: response.size_bytes,
            environment_name,
        });

        // Bound history for lightweight and fast loads.
        const MAX_HISTORY: usize = 500;
        if self.history.len() > MAX_HISTORY {
            let truncate = self.history.len() - MAX_HISTORY;
            self.history.drain(0..truncate);
        }

        self.updated_at = Utc::now();
    }

    /// Ensures there is at least one collection and request to operate on.
    pub fn ensure_seed_data(&mut self) {
        if self.collections.is_empty() {
            self.collections.push(Collection::new("Quickstart"));
        }

        if self.collections[0].requests.is_empty() {
            self.collections[0].requests.push(RequestTemplate::new(
                "Example GET",
                HttpMethod::Get,
                "https://httpbin.org/get",
            ));
        }

        if self.environments.is_empty() {
            let env = Environment::new("local");
            self.active_environment_id = Some(env.id);
            self.environments.push(env);
        } else if self.active_environment_id.is_none() {
            self.active_environment_id = Some(self.environments[0].id);
        }
    }
}
