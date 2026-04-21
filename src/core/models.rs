//! Domain models used by CLI, TUI, storage and HTTP execution.
//!
//! Doctag:domain-models

use std::fmt::{Display, Formatter};
use std::str::FromStr;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

/// Supported HTTP methods.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "UPPERCASE")]
pub enum HttpMethod {
    #[default]
    Get,
    Post,
    Put,
    Patch,
    Delete,
    Head,
    Options,
}

impl HttpMethod {
    /// Converts to reqwest's HTTP method type.
    pub fn as_reqwest_method(self) -> reqwest::Method {
        match self {
            Self::Get => reqwest::Method::GET,
            Self::Post => reqwest::Method::POST,
            Self::Put => reqwest::Method::PUT,
            Self::Patch => reqwest::Method::PATCH,
            Self::Delete => reqwest::Method::DELETE,
            Self::Head => reqwest::Method::HEAD,
            Self::Options => reqwest::Method::OPTIONS,
        }
    }
}

impl Display for HttpMethod {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let value = match self {
            Self::Get => "GET",
            Self::Post => "POST",
            Self::Put => "PUT",
            Self::Patch => "PATCH",
            Self::Delete => "DELETE",
            Self::Head => "HEAD",
            Self::Options => "OPTIONS",
        };

        write!(f, "{value}")
    }
}

impl FromStr for HttpMethod {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_uppercase().as_str() {
            "GET" => Ok(Self::Get),
            "POST" => Ok(Self::Post),
            "PUT" => Ok(Self::Put),
            "PATCH" => Ok(Self::Patch),
            "DELETE" => Ok(Self::Delete),
            "HEAD" => Ok(Self::Head),
            "OPTIONS" => Ok(Self::Options),
            other => Err(format!("unsupported method: {other}")),
        }
    }
}

/// A key-value pair used for headers, query items and form fields.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct KeyValue {
    pub key: String,
    pub value: String,
    pub enabled: bool,
}

impl KeyValue {
    /// Creates an enabled key-value pair.
    pub fn enabled(key: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
            enabled: true,
        }
    }
}

/// HTTP request body strategies.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum RequestBody {
    #[default]
    None,
    Raw {
        mime_type: String,
        content: String,
    },
    Json {
        value: Value,
    },
    FormUrlEncoded {
        fields: Vec<KeyValue>,
    },
}

/// Request authentication strategy.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AuthStrategy {
    #[default]
    None,
    Basic {
        username: String,
        password: String,
    },
    Bearer {
        token: String,
    },
    ApiKey {
        key: String,
        value: String,
        in_header: bool,
    },
}

/// A reusable request definition.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RequestTemplate {
    pub id: Uuid,
    pub name: String,
    pub method: HttpMethod,
    pub url: String,
    pub headers: Vec<KeyValue>,
    pub query: Vec<KeyValue>,
    pub body: RequestBody,
    pub auth: AuthStrategy,
    pub pre_request_script: Option<String>,
    pub test_script: Option<String>,
    pub tags: Vec<String>,
    pub timeout_ms: Option<u64>,
}

impl RequestTemplate {
    /// Creates a minimal request template.
    pub fn new(name: impl Into<String>, method: HttpMethod, url: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            method,
            url: url.into(),
            headers: Vec::new(),
            query: Vec::new(),
            body: RequestBody::None,
            auth: AuthStrategy::None,
            pre_request_script: None,
            test_script: None,
            tags: Vec::new(),
            timeout_ms: None,
        }
    }
}

/// A collection of related requests.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Collection {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub requests: Vec<RequestTemplate>,
}

impl Collection {
    /// Creates an empty collection with generated id.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            description: None,
            requests: Vec::new(),
        }
    }
}

/// Environment variable value.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EnvVar {
    pub key: String,
    pub value: String,
    pub secret: bool,
}

/// Runtime environment, similar to Postman environments.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Environment {
    pub id: Uuid,
    pub name: String,
    pub variables: Vec<EnvVar>,
}

impl Environment {
    /// Creates an empty environment with generated id.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            variables: Vec::new(),
        }
    }

    /// Returns variable lookup table for templating.
    pub fn as_context(&self) -> std::collections::HashMap<String, String> {
        self.variables
            .iter()
            .map(|item| (item.key.clone(), item.value.clone()))
            .collect()
    }
}

/// A concise history record for each request execution.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HistoryEntry {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub request_name: String,
    pub method: HttpMethod,
    pub url: String,
    pub status: u16,
    pub latency_ms: u128,
    pub response_size: usize,
    pub environment_name: Option<String>,
}

/// Captures execution result details shown in TUI and CLI output.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExecutedResponse {
    pub status: u16,
    pub status_text: String,
    pub headers: Vec<(String, String)>,
    pub body: String,
    pub duration_ms: u128,
    pub size_bytes: usize,
    pub script_logs: Vec<String>,
}

/// Workspace root object persisted to disk.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Workspace {
    pub id: Uuid,
    pub name: String,
    pub schema_version: String,
    pub collections: Vec<Collection>,
    pub environments: Vec<Environment>,
    pub active_environment_id: Option<Uuid>,
    pub history: Vec<HistoryEntry>,
    pub updated_at: DateTime<Utc>,
}
