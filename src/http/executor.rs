//! Request execution pipeline.
//!
//! Doctag:executor

use std::collections::HashMap;
use std::time::{Duration, Instant};

use reqwest::Client;

use crate::core::models::{
    AuthStrategy, Environment, ExecutedResponse, RequestBody, RequestTemplate,
};
use crate::http::auth::apply_auth;
use crate::http::scripting::{run_pre_request_script, run_test_script};
use crate::http::templating::{interpolate, interpolate_enabled_pairs};
use crate::{YoruError, YoruResult};

/// Orchestrates request interpolation, auth and transport.
#[derive(Debug, Clone)]
pub struct HttpExecutor {
    client: Client,
}

impl HttpExecutor {
    /// Constructs executor from a configured reqwest client.
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    /// Executes a request with optional environment context.
    pub async fn execute_request(
        &self,
        template: &RequestTemplate,
        environment: Option<&Environment>,
    ) -> YoruResult<ExecutedResponse> {
        let mut context: HashMap<String, String> =
            environment.map(Environment::as_context).unwrap_or_default();
        context.insert("request_name".to_string(), template.name.clone());

        let mut script_logs = Vec::new();
        let pre_report = run_pre_request_script(template.pre_request_script.as_deref(), &context);
        script_logs.extend(pre_report.logs);
        if let Some(err) = pre_report.error {
            return Err(YoruError::Script(format!(
                "pre-request script failed: {err}"
            )));
        }

        let method = template.method.as_reqwest_method();
        let url = interpolate(&template.url, &context);

        let mut builder = self.client.request(method, &url);

        let query_items = interpolate_enabled_pairs(&template.query, &context);
        if !query_items.is_empty() {
            builder = builder.query(&query_items);
        }

        for (key, value) in interpolate_enabled_pairs(&template.headers, &context) {
            builder = builder.header(key, value);
        }

        if let Some(timeout_ms) = template.timeout_ms {
            builder = builder.timeout(Duration::from_millis(timeout_ms));
        }

        builder = match &template.body {
            RequestBody::None => builder,
            RequestBody::Raw { mime_type, content } => builder
                .header("content-type", interpolate(mime_type, &context))
                .body(interpolate(content, &context)),
            RequestBody::Json { value } => {
                let interpolated = interpolate(&value.to_string(), &context);
                let parsed =
                    serde_json::from_str::<serde_json::Value>(&interpolated).map_err(|err| {
                        YoruError::Validation(format!("invalid interpolated JSON body: {err}"))
                    })?;
                builder.json(&parsed)
            }
            RequestBody::FormUrlEncoded { fields } => {
                let form = interpolate_enabled_pairs(fields, &context);
                builder.form(&form)
            }
        };

        let rendered_auth = render_auth(&template.auth, &context);
        builder = apply_auth(builder, &rendered_auth);

        let start = Instant::now();
        let response = builder.send().await?;
        let elapsed = start.elapsed().as_millis();

        let status = response.status();
        let status_code = status.as_u16();
        let status_text = status
            .canonical_reason()
            .map(ToString::to_string)
            .unwrap_or_else(|| "Unknown".to_string());

        let headers = response
            .headers()
            .iter()
            .map(|(name, value)| {
                (
                    name.as_str().to_string(),
                    value.to_str().unwrap_or("<binary>").to_string(),
                )
            })
            .collect::<Vec<_>>();

        let bytes = response.bytes().await?;
        let size_bytes = bytes.len();
        let body = String::from_utf8_lossy(&bytes).to_string();

        let mut test_context = context.clone();
        test_context.insert("status".to_string(), status_code.to_string());
        test_context.insert("response_body".to_string(), body.clone());

        let test_report = run_test_script(template.test_script.as_deref(), &test_context);
        script_logs.extend(test_report.logs);
        if let Some(err) = test_report.error {
            script_logs.push(format!("test script failed: {err}"));
        }

        Ok(ExecutedResponse {
            status: status_code,
            status_text,
            headers,
            body,
            duration_ms: elapsed,
            size_bytes,
            script_logs,
        })
    }
}

fn render_auth(auth: &AuthStrategy, context: &HashMap<String, String>) -> AuthStrategy {
    match auth {
        AuthStrategy::None => AuthStrategy::None,
        AuthStrategy::Basic { username, password } => AuthStrategy::Basic {
            username: interpolate(username, context),
            password: interpolate(password, context),
        },
        AuthStrategy::Bearer { token } => AuthStrategy::Bearer {
            token: interpolate(token, context),
        },
        AuthStrategy::ApiKey {
            key,
            value,
            in_header,
        } => AuthStrategy::ApiKey {
            key: interpolate(key, context),
            value: interpolate(value, context),
            in_header: *in_header,
        },
    }
}
